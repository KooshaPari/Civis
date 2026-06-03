#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Pipes;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Threading;
using DINOForge.Bridge.Protocol;
using DINOForge.Runtime.Diagnostics;
using DINOForge.Runtime.Telemetry;
using DINOForge.Runtime.UI;
using RuntimeDriver = DINOForge.Runtime.RuntimeDriver;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using Unity.Collections;
using Unity.Entities;
using UnityEngine;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Named pipe server implementing JSON-RPC 2.0 over NDJSON for IPC communication.
    /// Runs on a background thread and dispatches Unity-thread-required operations
    /// through <see cref="MainThreadDispatcher"/>.
    /// </summary>
    public sealed partial class GameBridgeServer : IDisposable
    {
        /// <summary>The well-known pipe name used by the DINOForge bridge.</summary>
        public const string PipeName = "dinoforge-game-bridge";

        // CLR's COR_E_THREADABORT HRESULT — Thread.Abort on .NET Core wraps as IOException.
        // See: https://github.com/dotnet/runtime/blob/main/src/libraries/System.Private.CoreLib/src/System/Threading/Thread.cs
        private const int COR_E_THREADABORT = unchecked((int)0x80131623);

        // Bounded waits for MainThreadDispatcher work items (task #535).
        // Background threads MUST NOT call .Result or .Wait() unbounded — the dispatcher
        // pump (KeyInputSystem.OnUpdate) can be torn down at any scene transition, after
        // which queued work never drains and the bridge thread parks indefinitely
        // (IsHungAppWindow=True). Use these constants as the timeout argument.
        // threshold-ok: bounded main-thread wait for ECS handlers
        private const int MainThreadWaitTimeoutMs = 5000;
        // threshold-ok: mid-tier bound for input-injection handlers that scan scene objects
        private const int MainThreadInputWaitTimeoutMs = 8000;
        // threshold-ok: longer bound for heavier reflection-driven UI handlers
        private const int MainThreadHeavyWaitTimeoutMs = 10000;
        // threshold-ok: poll interval for shutdown signal check; short enough to react quickly, long enough not to spin
        private const int ShutdownPollIntervalMs = 200;

        /// <summary>
        /// Resets the current thread abort state on legacy Unity/Mono runtimes.
        /// The runtime still relies on ThreadAbortException handling for scene-transition teardown,
        /// so the call remains, but the SYSLIB0006 warning is intentionally suppressed here rather
        /// than changing the abort recovery behavior.
        /// </summary>
#pragma warning disable SYSLIB0006
        private static void ResetThreadAbort()
        {
            Thread.ResetAbort();
        }
#pragma warning restore SYSLIB0006

        private ModPlatform _platform;
        private readonly DateTimeOffset _startTime;
        private readonly TimeProvider _timeProvider;
        private Thread? _serverThread;
        private volatile bool _running;

        /// <summary>
        /// Diagnostic: true if the server background thread is alive. Used by RuntimeDriver.OnDestroy
        /// to log accurate state instead of asserting "Bridge kept alive" without verification.
        /// (iter-144 #535 re-fix.)
        /// </summary>
        public bool IsServerThreadAlive => _running && _serverThread != null && _serverThread.IsAlive;
        private volatile NamedPipeServerStream? _currentPipe;
        private readonly ManualResetEventSlim _shutdownEvent = new(false);
        private SessionHmac? _session;

        /// <summary>
        /// True while the ModPlatform is alive (not destroyed during a scene transition).
        /// </summary>
        private bool IsPlatformAlive
        {
            get
            {
                try { return _platform != null && _platform.IsInitialized; }
                catch (Exception) { /* safe-swallow: bridge health probe must return false instead of surfacing transient teardown races */ return false; }
            }
        }

        /// <summary>
        /// Creates a new game bridge server.
        /// </summary>
        /// <param name="platform">The ModPlatform instance for accessing subsystems.</param>
        /// <param name="timeProvider">Optional TimeProvider for testing. Defaults to <see cref="TimeProvider.System"/>.</param>
        public GameBridgeServer(ModPlatform platform, TimeProvider? timeProvider = null)
        {
            _platform = platform ?? throw new ArgumentNullException(nameof(platform));
            _timeProvider = timeProvider ?? TimeProvider.System;
            _startTime = _timeProvider.GetUtcNow();
        }

        /// <summary>
        /// Starts the named pipe server on a background thread.
        /// </summary>
        public void Start()
        {
            if (_running) return;
            _running = true;
            StartThread();
        }

        /// <summary>
        /// Starts (or restarts) the server thread. If the thread was aborted by
        /// DINO's scene transitions, this creates a new one.
        /// </summary>
        private void StartThread()
        {
            _serverThread = new Thread(ServerLoopWithAutoRestart)
            {
                Name = "DINOForge-Bridge-Server",
                IsBackground = false
            };
            _serverThread.Start();
            DebugLog.Write("GameBridgeServer", "[GameBridgeServer] Started on pipe: " + PipeName);
        }

        /// <summary>
        /// Wrapper around ServerLoop that catches ThreadAbortException and restarts.
        /// </summary>
        private void ServerLoopWithAutoRestart()
        {
            try
            {
                ServerLoop();
            }
            catch
            {
                // ServerLoop exited — either stopped normally or thread was aborted.
                // Dispose any lingering pipe to free the pipe name for restart.
                try { _currentPipe?.Dispose(); } catch { /* safe-swallow: pipe already disposed or invalid during restart */ }
                _currentPipe = null;

                if (_running)
                {
                    DebugLog.Write("GameBridgeServer", "[GameBridgeServer] Server loop exited — restarting in 2s...");
                    try
                    {
                        new Thread(() =>
                        {
                            Thread.Sleep(2000);
                            if (_running) StartThread();
                        })
                        { IsBackground = false, Name = "DINOForge-Bridge-Restart" }.Start();
                    }
                    catch (Exception ex)
                    {
                        DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Restart failed: {ex.Message}");
                    }
                }
            }
        }

        /// <summary>
        /// Iter-144 #547 gray-freeze ROOT CAUSE fix: signals shutdown and disposes the current
        /// pipe handle synchronously to unblock the kernel-mode ConnectNamedPipe wait that was
        /// parking the bridge thread in mono_jit_cleanup. Safe to call from Unity OnDestroy at
        /// the TOP of teardown, BEFORE mono_threads_set_shutting_down tries to interrupt the
        /// blocked thread. WinDbg analysis (docs/sessions/iter144-windbg-wedge-stack.md)
        /// identified the synchronous ConnectNamedPipe syscall on thread 82 as the wedge.
        /// </summary>
        public void RequestShutdown()
        {
            _running = false;
            try { _shutdownEvent.Set(); } catch { /* safe-swallow: event disposed during shutdown race */ }

            // Dispose the pipe handle synchronously — this unblocks the kernel I/O
            // (BeginWaitForConnection / WaitForConnectionAsync) with ObjectDisposedException.
            try
            {
                NamedPipeServerStream? pipe = _currentPipe;
                _currentPipe = null;
                pipe?.Dispose();
            }
            catch { /* safe-swallow: pipe dispose during shutdown — kernel I/O will unblock */ }

            DebugLog.Write("GameBridgeServer", "[GameBridgeServer] shutdown requested — pipe disposed, accept loop will exit.");
        }

        /// <summary>
        /// Stops the server and releases all resources.
        /// </summary>
        public void Stop()
        {
            RequestShutdown();

            try
            {
                _session?.Dispose();
            }
            catch { /* safe-swallow: session disposal during shutdown can race */ }

            _session = null;
            DebugLog.Write("GameBridgeServer", "[GameBridgeServer] Stopped.");
        }

        /// <summary>
        /// Disposes the server, stopping it if running.
        /// </summary>
        public void Dispose()
        {
            Stop();
        }

        /// <summary>
        /// Updates the ModPlatform reference after resurrection.
        /// Called when a new RuntimeDriver is created and re-initializes ModPlatform.
        /// Also ensures the server thread is alive — restarts it if it died.
        /// </summary>
        public void UpdatePlatform(ModPlatform platform)
        {
            _platform = platform;
            DebugLog.Write("GameBridgeServer", "[GameBridgeServer] Platform reference updated (post-resurrection).");
            EnsureServerAlive();
        }

        /// <summary>
        /// Checks if the server thread is alive and restarts it if dead.
        /// Also triggers RuntimeDriver resurrection if PersistentRoot is null.
        /// Called from KeyInputSystem.OnUpdate() every ~50ms to ensure the bridge
        /// and UI systems survive Unity's scene transitions which may abort threads
        /// and destroy the RuntimeDriver.
        /// </summary>
        public void EnsureServerAlive()
        {
            // If RuntimeDriver was destroyed (scene transition), resurrect it.
            // This creates a new RuntimeDriver which re-registers KeyInputSystem
            // in the current ECS world, ensuring DrainQueue and F9/F10 work.
            if (Plugin.PersistentRoot == null)
            {
                DebugLog.Write("GameBridgeServer", "[GameBridgeServer] PersistentRoot is null — triggering resurrection...");
                Plugin.TryResurrect("(Bridge supervisor)", "EnsureServerAlive");
            }

            if (_running && (_serverThread == null || !_serverThread.IsAlive))
            {
                DebugLog.Write("GameBridgeServer", "[GameBridgeServer] Server thread is dead — restarting...");
                Stop();
                // Create fresh thread — the old thread object is abandoned after abort.
                Start();
            }
        }

        /// <summary>
        /// Main server loop: accepts pipe connections and processes NDJSON messages.
        /// Reconnects automatically after each client disconnects.
        /// </summary>
        private void ServerLoop()
        {
            while (_running)
            {
                NamedPipeServerStream? pipe = null;
                try
                {
                    // Use None (synchronous mode) — this server runs on a dedicated background
                    // thread so async pipe mode is not needed and causes ReadLine() to block
                    // indefinitely on Windows when no data is available.
                    // Allow multiple server instances so that after a ThreadAbortException
                    // + ResetAbort cycle, a new pipe can be created even if the old one
                    // hasn't been fully disposed yet.
                    // Iter-144 #547 gray-freeze ROOT CAUSE fix: use PipeOptions.Asynchronous so
                    // BeginWaitForConnection returns an IAsyncResult whose WaitHandle can be
                    // multiplexed with _shutdownEvent. The previous synchronous WaitForConnection
                    // parked this thread in kernel-mode ConnectNamedPipe; Mono could not interrupt
                    // it, causing mono_jit_cleanup to wait forever on the bridge thread at the
                    // MainMenu transition (WinDbg dump: docs/sessions/iter144-windbg-wedge-stack.md).
                    pipe = new NamedPipeServerStream(
                        PipeName,
                        PipeDirection.InOut,
                        NamedPipeServerStream.MaxAllowedServerInstances,
                        PipeTransmissionMode.Byte,
                        PipeOptions.Asynchronous);

                    _currentPipe = pipe;
                    DebugLog.Write("GameBridgeServer", "[GameBridgeServer] Waiting for connection (async-cancellable)...");

                    IAsyncResult connectAr = pipe.BeginWaitForConnection(null, null);
                    WaitHandle[] handles = { connectAr.AsyncWaitHandle, _shutdownEvent.WaitHandle };
                    int sigIdx = WaitHandle.WaitAny(handles);
                    if (sigIdx == 1 || !_running)
                    {
                        // Shutdown signaled — dispose pipe to unblock the pending I/O cleanly.
                        DebugLog.Write("GameBridgeServer", "[GameBridgeServer] pipe disposed, accept loop exiting (shutdown).");
                        try { pipe.Dispose(); } catch { /* safe-swallow: pipe dispose during shutdown */ }
                        _currentPipe = null;
                        return;
                    }

                    // Connection completed — call EndWaitForConnection to observe any exception.
                    pipe.EndWaitForConnection(connectAr);
                    DebugLog.Write("GameBridgeServer", "[GameBridgeServer] Client connected.");

                    DebugLog.Write("GameBridgeServer", "[GameBridgeServer] Setting up line reader");
                    // Read lines manually byte-by-byte to avoid StreamReader buffering issues
                    // on Mono with synchronous named pipes.
                    while (_running && pipe.IsConnected)
                    {
                        string? line = null;
                        bool responseWritten = false;
                        string? processError = null;
                        try
                        {
                            // Read line from client
                            try
                            {
                                line = ReadLineFromPipe(pipe);
                            }
                            catch (IOException ex)
                            {
                                if (ex.HResult == unchecked((int)0x80131623))
                                    ResetThreadAbort();
                            }
                            catch (ThreadAbortException)
                            {
                                ResetThreadAbort();
                            }
                            catch { /* safe-swallow: read errors on client pipe — loop continues, line remains null */ }

                            if (line == null) continue;
                            if (string.IsNullOrWhiteSpace(line)) continue;

                            // Process message
                            string? response = null;
                            processError = null;
                            try
                            {
                                response = ProcessMessage(line);
                            }
                            catch (ThreadAbortException)
                            {
                                ResetThreadAbort();
                            }
                            catch (Exception ex)
                            {
                                processError = ex.ToString();
                            }

                            if (response == null) continue;

                            // Write response to client
                            try
                            {
                                byte[] bytes = Encoding.UTF8.GetBytes(response + "\n");
                                pipe.Write(bytes, 0, bytes.Length);
                                pipe.Flush();
                                responseWritten = true;
                            }
                            catch (IOException ex)
                            {
                                if (ex.HResult == unchecked((int)0x80131623))
                                    ResetThreadAbort();
                            }
                            catch (ThreadAbortException)
                            {
                                ResetThreadAbort();
                            }
                            catch { /* safe-swallow: write errors trigger fallback writer in finally block */ }
                        }
                        finally
                        {
                            // GUARANTEED fallback: if no response was written (exception occurred),
                            // send a minimal error response so the client unblocks and does not hang.
                            if (!responseWritten)
                            {
                                string fallbackJson = BuildBridgeErrorResponse(processError) + "\n";
                                bool pipeWriteFailed = false;
                                try
                                {
                                    byte[] fallback = Encoding.UTF8.GetBytes(fallbackJson);
                                    pipe.Write(fallback, 0, fallback.Length);
                                    pipe.Flush();
                                }
                                catch
                                {
                                    pipeWriteFailed = true;
                                }

                                if (pipeWriteFailed)
                                {
                                    // Pipe is broken (e.g., Thread.Abort during write).
                                    // Write fallback to a file as a backup. The CLI will check this file.
                                    try
                                    {
                                        string fallbackDir = Path.Combine(Path.GetTempPath(), "DINOForge");
                                        Directory.CreateDirectory(fallbackDir);
                                        string fallbackFile = Path.Combine(fallbackDir, "dinoforge_bridge_fallback.txt");
                                        File.WriteAllText(fallbackFile, fallbackJson.TrimEnd(), Encoding.UTF8);
                                    }
                                    catch { /* safe-swallow: fallback file is best-effort diagnostic; pipe error already handled */ }
                                }
                            }
                        }
                    }

                    DebugLog.Write("GameBridgeServer", "[GameBridgeServer] Exited read loop");

                    DebugLog.Write("GameBridgeServer", "[GameBridgeServer] Client disconnected.");
                }
                catch (ObjectDisposedException)
                {
                    // safe-swallow: server is shutting down — pipe disposal is expected
                }
                catch (System.Threading.ThreadAbortException)
                {
                    // DINO/Unity may abort threads during scene transitions.
                    // Reset the abort and continue the loop — the bridge must survive.
                    ResetThreadAbort();
                    DebugLog.Write("GameBridgeServer", "[GameBridgeServer] [OUTER] ThreadAbortException caught — closing pipe to unblock client.");
                    try { pipe?.Dispose(); } catch { /* safe-swallow: pipe dispose after ThreadAbort recovery */ }
                }
                catch (IOException ex)
                {
                    // Thread.Abort may manifest as IOException with COR_E_THREADABORT HResult
                    // (0x80131623) when the abort interrupts a blocking synchronous I/O call.
                    // See: https://github.com/dotnet/runtime/issues/30675
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] [OUTER-IO] IOException: {ex.Message} (HResult=0x{ex.HResult:X8})");
                    if (ex.HResult == COR_E_THREADABORT)
                    {
                        ResetThreadAbort();
                        DebugLog.Write("GameBridgeServer", "[GameBridgeServer] [OUTER-IO] COR_E_THREADABORT — resetting and restarting.");
                    }
                    else
                    {
                        DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] [OUTER-IO] Non-abort IOException.");
                    }
                    try { pipe?.Dispose(); } catch { /* safe-swallow: pipe dispose after IO error */ }
                }
                catch (Exception ex)
                {
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] [OUTER] Error in server loop: {ex.Message}");
                    try { pipe?.Dispose(); } catch { /* safe-swallow: pipe dispose after outer exception */ }
                }
                finally
                {
                    // Close the pipe handle. Unlike Dispose(), Close() on Windows named pipes
                    // sends ERROR_OPIPE_NOT_CONNECTED to waiting clients without blocking.
                    // This unblocks any client blocked in Read() on this pipe.
                    try { pipe?.Dispose(); } catch { /* safe-swallow: pipe dispose in finally — guaranteed cleanup */ }
                    _currentPipe = null;
                }

                // Pause before re-listening. Longer delay after errors to avoid log spam.
                if (_running)
                {
                    Thread.Sleep(1000);
                }
            }
        }

        private static string BuildBridgeErrorResponse(string? detail)
        {
            string message = string.IsNullOrWhiteSpace(detail) ? "Bridge error" : $"Bridge error: {detail}";
            return JsonConvert.SerializeObject(new
            {
                jsonrpc = "2.0",
                id = (object?)null,
                error = new { code = -32603, message }
            });
        }

        /// <summary>
        /// Parses a single NDJSON line as a JSON-RPC request, dispatches to the
        /// appropriate handler, and returns the serialized response.
        /// </summary>
        private string ProcessMessage(string json)
        {
            JsonRpcRequest? request;
            try
            {
                request = JsonConvert.DeserializeObject<JsonRpcRequest>(json);
            }
            catch (Exception ex)
            {
                return SerializeError(null, -32700, "Parse error: " + ex.Message);
            }

            if (request == null || string.IsNullOrEmpty(request.Method))
            {
                return SerializeError(request?.Id, -32600, "Invalid request");
            }

            try
            {
                JToken result = DispatchMethod(request.Method, request.Params);
                return SerializeSuccess(request.Id, result);
            }
            catch (ThreadAbortException tae)
            {
                // Thread.Abort() was called on the bridge thread (e.g., by Unity/Mono runtime cleanup).
                // Reset the abort so the thread can continue. Return a valid response so the client
                // unblocks — otherwise the pipe breaks without a response and the client hangs forever.
                ResetThreadAbort();
                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] ThreadAbortException during '{request.Method}': {tae.Message}");
                return SerializeError(request.Id, -32603, "Bridge thread abort — retry later");
            }
            catch (Exception ex)
            {
                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Handler error for '{request.Method}': {ex}");
                return SerializeError(request.Id, -32603, "Internal error: " + ex.Message);
            }
        }

        /// <summary>
        /// Routes a method name to the appropriate handler and returns the result as a JToken.
        /// </summary>
        private JToken DispatchMethod(string method, JObject? parameters)
        {
            // Normalize: accept both "game.status" and "status" formats
            string m = method.StartsWith("game.") ? method.Substring(5) : method;
            switch (m)
            {
                case "connect":
                    return HandleConnect();
                case "ping":
                    return HandlePing();
                case "status":
                    return HandleStatus();
                case "getCatalog":
                    return HandleGetCatalog();
                case "getComponentMap":
                    return HandleGetComponentMap(parameters);
                case "discoverTypes":
                    return HandleDiscoverTypes(parameters);
                case "getUiTree":
                    return HandleGetUiTree(parameters);
                case "queryUi":
                    return HandleQueryUi(parameters);
                case "clickUi":
                    return HandleClickUi(parameters);
                case "waitForUi":
                    return HandleWaitForUi(parameters);
                case "expectUi":
                    return HandleExpectUi(parameters);
                case "getStat":
                    return HandleGetStat(parameters);
                case "applyOverride":
                    return HandleApplyOverride(parameters);
                case "queryEntities":
                    return HandleQueryEntities(parameters);
                case "reloadPacks":
                    return HandleReloadPacks(parameters);
                case "getResources":
                    return HandleGetResources();
                case "screenshot":
                    return HandleScreenshot(parameters);
                case "dumpState":
                    return HandleDumpState(parameters);
                case "verifyMod":
                    return HandleVerifyMod(parameters);
                case "waitForWorld":
                    return HandleWaitForWorld(parameters);
                case "loadScene":
                    return HandleLoadScene(parameters);
                case "startGame":
                    return HandleStartGame(parameters);
                case "loadSave":
                    return HandleLoadSave(parameters);
                case "listSaves":
                    return HandleListSaves();
                case "pressKey":
                    return HandlePressKey(parameters);
                case "simulateKey":
                    return HandleSimulateKey(parameters);
                case "pressEscape":
                    return HandleSimulateKey(new JObject { ["key"] = "Escape" });
                case "togglePauseMenu":
                    return HandleTogglePauseMenu(parameters);
                case "dismissLoadScreen":
                    return HandleDismissLoadScreen();
                case "clickButton":
                    return HandleClickButton(parameters);
                case "toggleUi":
                    return HandleToggleUi(parameters);
                case "invokeMethod":
                    return HandleInvokeMethod(parameters);
                case "getMetrics":
                    return HandleGetMetrics();
                default:
                    throw new InvalidOperationException($"Method not found: {method}");
            }
        }

        // ──────────────────────────────────────────────
        //  Handler Implementations
        // ──────────────────────────────────────────────

        /// <summary>
        /// Handles the <c>connect</c> handshake request. Generates a fresh session
        /// with a unique session_id and 32-byte ephemeral key for Phase 4a SessionHmac.
        /// Returns a JSON object with snake_case fields: session_id, session_key_b64.
        /// Per the mock server contract, replaces any prior session (reconnect semantics).
        /// </summary>
        private JToken HandleConnect()
        {
            // Dispose previous session if one exists (reconnect semantics)
            _session?.Dispose();

            // Mint a fresh session
            _session = new SessionHmac();

            // Return the handshake envelope with snake_case fields
            var envelope = new JObject
            {
                ["session_id"] = _session.SessionId,
                ["session_key_b64"] = _session.KeyMaterialB64(),
            };

            DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleConnect: minted session_id={_session.SessionId}");
            return envelope;
        }

        private JToken HandlePing()
        {
            PingResult result = new PingResult
            {
                Pong = true,
                Version = PluginInfo.VERSION,
                UptimeSeconds = (_timeProvider.GetUtcNow() - _startTime).TotalSeconds
            };
            return JToken.FromObject(result);
        }

        private JToken HandleStatus()
        {
            DebugLog.Write("GameBridgeServer", "[GameBridgeServer] HandleStatus ENTER");
            GameStatus status = new GameStatus
            {
                Running = _running && IsPlatformAlive,
                WorldReady = IsPlatformAlive && _platform.IsWorldReady,
                ModPlatformReady = IsPlatformAlive && _platform.IsInitialized,
                Version = PluginInfo.VERSION,
                EntityCount = -1,
                LoadedPacks = new List<string>()
            };

            // Access KeyInputSystem cached values for world name.
            // KeyInputSystem.OnUpdate caches these from the main ECS thread each frame.
            // Reading static strings from a background thread is safe.
            try
            {
                string? cachedName = KeyInputSystem.CachedWorldName;
                if (!string.IsNullOrEmpty(cachedName))
                    status.WorldName = cachedName;
            }
            catch (Exception worldEx)
            {
                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] KeyInputSystem.CachedWorldName failed: {worldEx.Message}");
            }

            // Try entity count from KeyInputSystem cached value (updated each OnUpdate frame).
            // This is a static int read — thread-safe and never triggers ECS main-thread checks.
            try
            {
                int cachedCount = KeyInputSystem.LastEntityCount;
                status.EntityCount = cachedCount;
            }
            catch (Exception ex)
            {
                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] KeyInputSystem.LastEntityCount failed: {ex.Message}");
            }

            // Populate loaded pack names from platform
            if (IsPlatformAlive && _platform.IsInitialized)
            {
                try
                {
                    System.Collections.Generic.IReadOnlyList<string>? packs = _platform.GetLoadedPackIds();
                    if (packs != null)
                    {
                        foreach (string id in packs)
                            status.LoadedPacks.Add(id);
                    }
                }
                catch { /* safe-swallow: pack-status enumeration is best-effort diagnostic */ }
            }

            DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleStatus EXIT: worldName='{status.WorldName}' entityCount={status.EntityCount}");
            try { return JToken.FromObject(status); }
            catch { return JToken.FromObject(new { EntityCount = -1, Running = _running && IsPlatformAlive }); /* safe-swallow: JToken serialization fallback */ }
        }

        private JToken HandleGetCatalog()
        {
            VanillaCatalog? catalog = IsPlatformAlive ? _platform.Catalog : null;
            CatalogSnapshot snapshot = new CatalogSnapshot();

            if (catalog == null || !catalog.IsBuilt)
                return JToken.FromObject(snapshot);

            foreach (VanillaEntityInfo info in catalog.Units)
            {
                snapshot.Units.Add(new DINOForge.Bridge.Protocol.CatalogEntry
                {
                    InferredId = info.InferredId,
                    ComponentCount = info.ComponentTypes.Length,
                    EntityCount = info.EntityCount,
                    Category = info.Category
                });
            }

            foreach (VanillaEntityInfo info in catalog.Buildings)
            {
                snapshot.Buildings.Add(new DINOForge.Bridge.Protocol.CatalogEntry
                {
                    InferredId = info.InferredId,
                    ComponentCount = info.ComponentTypes.Length,
                    EntityCount = info.EntityCount,
                    Category = info.Category
                });
            }

            foreach (VanillaEntityInfo info in catalog.Projectiles)
            {
                snapshot.Projectiles.Add(new DINOForge.Bridge.Protocol.CatalogEntry
                {
                    InferredId = info.InferredId,
                    ComponentCount = info.ComponentTypes.Length,
                    EntityCount = info.EntityCount,
                    Category = info.Category
                });
            }

            foreach (VanillaEntityInfo info in catalog.Other)
            {
                snapshot.Other.Add(new DINOForge.Bridge.Protocol.CatalogEntry
                {
                    InferredId = info.InferredId,
                    ComponentCount = info.ComponentTypes.Length,
                    EntityCount = info.EntityCount,
                    Category = info.Category
                });
            }

            return JToken.FromObject(snapshot);
        }

        private JToken HandleGetComponentMap(JObject? parameters)
        {
            string? sdkPath = parameters?.Value<string>("sdkPath");

            ComponentMapResult result = new ComponentMapResult();

            if (sdkPath != null)
            {
                ComponentMapping? mapping = ComponentMap.Find(sdkPath);
                if (mapping != null)
                {
                    result.Mappings.Add(MappingToEntry(mapping));
                }
            }
            else
            {
                foreach (KeyValuePair<string, ComponentMapping> kvp in ComponentMap.All)
                {
                    result.Mappings.Add(MappingToEntry(kvp.Value));
                }
            }

            return JToken.FromObject(result);
        }

        /// <summary>
        /// Discovers and returns ECS component types from loaded game assemblies.
        /// Useful for identifying correct type names when game version changes.
        /// </summary>
        private JToken HandleDiscoverTypes(JObject? parameters)
        {
            string? pattern = parameters?.Value<string>("pattern");

            EcsTypeDiscovery.DiscoverAndLog();

            var assemblies = EcsTypeDiscovery.GetDiscoveredAssemblies() ?? new List<string>();
            var types = (pattern != null
                ? EcsTypeDiscovery.FindTypes(pattern)
                : EcsTypeDiscovery.GetDiscoveredTypes() ?? Enumerable.Empty<string>())
                .Take(200)
                .ToList();

            return JToken.FromObject(new
            {
                success = true,
                assemblies = assemblies,
                typesFound = types.Count,
                types = types,
                pattern = pattern ?? "(all)",
                logMessage = "Full type list written to dinoforge_debug.log"
            });
        }

        private JToken HandleGetUiTree(JObject? parameters)
        {
            string? selector = parameters?.Value<string>("selector");

            var result = MainThreadDispatcher.RunOnMainThread(() => UiTreeSnapshotBuilder.Capture(selector));
            // sync-over-async-unavoidable: ECS-bound, main-thread-required. MainThreadDispatcher.RunOnMainThread
            // returns a Task that completes on main thread; RPC handler must wait synchronously to return response.
            bool completed = result.Wait(MainThreadWaitTimeoutMs);
            UiTreeResult treeResult;
            if (!completed)
            {
                treeResult = new UiTreeResult
                {
                    Success = false,
                    Message = "Timed out while capturing UI tree.",
                    Selector = selector,
                    GeneratedAtUtc = DateTime.UtcNow.ToString("O"),
                    NodeCount = 0,
                    Root = new UiNode
                    {
                        Id = "root",
                        Path = "root",
                        Name = "root",
                        Label = "Unity UI",
                        Role = "root",
                        ComponentType = "Root",
                        Active = true,
                        Visible = true,
                        Interactable = false,
                        RaycastTarget = false
                    }
                };
            }
            else
            {
                // sync-over-async-unavoidable: ECS-bound, main-thread-required
                treeResult = result.Result;
            }

            UiActionTrace.Record("tree", selector ?? "", treeResult);
            return JToken.FromObject(treeResult);
        }

        private JToken HandleQueryUi(JObject? parameters)
        {
            string selector = parameters?.Value<string>("selector") ?? string.Empty;
            var result = MainThreadDispatcher.RunOnMainThread(() => UiSelectorEngine.Query(selector));
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            bool completed = result.Wait(MainThreadWaitTimeoutMs);
            UiActionResult queryResult;
            if (!completed)
            {
                queryResult = new UiActionResult
                {
                    Success = false,
                    Selector = selector,
                    Message = "Timed out while querying UI."
                };
            }
            else
            {
                // sync-over-async-unavoidable: ECS-bound, main-thread-required
                queryResult = result.Result;
            }

            UiActionTrace.Record("query", selector, queryResult, queryResult.MatchedNode);
            return JToken.FromObject(queryResult);
        }

        private JToken HandleClickUi(JObject? parameters)
        {
            string selector = parameters?.Value<string>("selector") ?? string.Empty;
            var result = MainThreadDispatcher.RunOnMainThread(() => UiSelectorEngine.Click(selector));
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            bool completed = result.Wait(MainThreadWaitTimeoutMs);
            UiActionResult clickResult;
            if (!completed)
            {
                clickResult = new UiActionResult
                {
                    Success = false,
                    Selector = selector,
                    Message = "Timed out while clicking UI."
                };
            }
            else
            {
                // sync-over-async-unavoidable: ECS-bound, main-thread-required
                clickResult = result.Result;
            }

            UiActionTrace.Record("click", selector, clickResult, clickResult.MatchedNode);
            return JToken.FromObject(clickResult);
        }

        private JToken HandleWaitForUi(JObject? parameters)
        {
            string selector = parameters?.Value<string>("selector") ?? string.Empty;
            string? state = parameters?.Value<string>("state");
            int timeoutMs = parameters?.Value<int?>("timeoutMs") ?? 5000;
            DateTimeOffset deadline = _timeProvider.GetUtcNow().AddMilliseconds(Math.Max(1, timeoutMs));
            UiWaitResult? lastResult = null;

            while (_timeProvider.GetUtcNow() <= deadline)
            {
                var evalTask = MainThreadDispatcher.RunOnMainThread(() => UiSelectorEngine.EvaluateState(selector, state));
                // sync-over-async-unavoidable: ECS-bound, main-thread-required
                bool completed = evalTask.Wait(MainThreadWaitTimeoutMs);
                if (!completed)
                {
                    var timeoutResult = new UiWaitResult
                    {
                        Ready = false,
                        Selector = selector,
                        State = string.IsNullOrWhiteSpace(state) ? "visible" : state!,
                        Message = "Timed out while evaluating UI state on the main thread."
                    };
                    UiActionTrace.Record("wait", selector, timeoutResult, timeoutResult.MatchedNode);
                    return JToken.FromObject(timeoutResult);
                }

                // sync-over-async-unavoidable: ECS-bound, main-thread-required
                lastResult = evalTask.Result;
                if (lastResult.Ready)
                {
                    UiActionTrace.Record("wait", selector, lastResult, lastResult.MatchedNode);
                    return JToken.FromObject(lastResult);
                }

                Thread.Sleep(100);
            }

            var finalResult = lastResult ?? new UiWaitResult
            {
                Ready = false,
                Selector = selector,
                State = string.IsNullOrWhiteSpace(state) ? "visible" : state!,
                Message = $"Timed out waiting for selector '{selector}'."
            };
            UiActionTrace.Record("wait", selector, finalResult, finalResult.MatchedNode);
            return JToken.FromObject(finalResult);
        }

        private JToken HandleExpectUi(JObject? parameters)
        {
            string selector = parameters?.Value<string>("selector") ?? string.Empty;
            string condition = parameters?.Value<string>("condition") ?? "visible";

            var result = MainThreadDispatcher.RunOnMainThread(() => UiSelectorEngine.Expect(selector, condition));
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            bool completed = result.Wait(MainThreadWaitTimeoutMs);
            UiExpectationResult expectResult;
            if (!completed)
            {
                expectResult = new UiExpectationResult
                {
                    Success = false,
                    Selector = selector,
                    Condition = condition,
                    Message = "Timed out while evaluating UI expectation."
                };
            }
            else
            {
                // sync-over-async-unavoidable: ECS-bound, main-thread-required
                expectResult = result.Result;
            }

            UiActionTrace.Record("expect", selector, expectResult, expectResult.MatchedNode);
            return JToken.FromObject(expectResult);
        }

        private JToken HandleGetStat(JObject? parameters)
        {
            string sdkPath = parameters?.Value<string>("sdkPath") ?? "";
            int? entityIndex = parameters?.Value<int?>("entityIndex");

            if (string.IsNullOrEmpty(sdkPath))
                throw new ArgumentException("sdkPath is required");

            ComponentMapping? mapping = ComponentMap.Find(sdkPath);
            if (mapping == null)
                throw new ArgumentException($"Unknown SDK path: {sdkPath}");

            // Reading ECS data requires main thread.
            // Task #535: bound the wait to avoid wedging the bridge thread when the
            // dispatcher pump (KeyInputSystem) is dead. On timeout, surface a structured
            // failure with EntityCount=0 so callers can distinguish "no data" from "hang".
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            var statTask = MainThreadDispatcher.RunOnMainThread(() =>
            {
                return ReadStatFromEcs(mapping, entityIndex);
            });
            StatResult statResult;
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            if (!statTask.Wait(MainThreadWaitTimeoutMs))
            {
                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleGetStat timed out ({MainThreadWaitTimeoutMs}ms) — dispatcher pump may be dead");
                statResult = new StatResult
                {
                    SdkPath = mapping.SdkModelPath,
                    ComponentType = mapping.EcsComponentType,
                    FieldName = mapping.TargetFieldName ?? "",
                    EntityCount = 0
                };
            }
            else
            {
                // sync-over-async-unavoidable: ECS-bound, main-thread-required
                statResult = statTask.Result;
            }

            return JToken.FromObject(statResult);
        }

        private JToken HandleApplyOverride(JObject? parameters)
        {
            string sdkPath = parameters?.Value<string>("sdkPath") ?? "";
            float value = parameters?.Value<float>("value") ?? 0f;
            string modeStr = parameters?.Value<string>("mode") ?? "override";
            string? filter = parameters?.Value<string>("filter");

            if (string.IsNullOrEmpty(sdkPath))
                throw new ArgumentException("sdkPath is required");

            ModifierMode mode;
            switch (modeStr.ToLowerInvariant())
            {
                case "add":
                    mode = ModifierMode.Add;
                    break;
                case "multiply":
                    mode = ModifierMode.Multiply;
                    break;
                default:
                    mode = ModifierMode.Override;
                    break;
            }

            StatModification mod = new StatModification(sdkPath, value, mode, filter);

            // Apply immediately on the main thread so callers see the change reflected at once.
            // Also enqueue so the StatModifierSystem re-applies it after scene reloads.
            // Task #535: bounded wait — on timeout, the enqueue won't happen but the bridge
            // thread survives; return Success=false so callers can retry once the pump is alive.
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            var overrideTask = MainThreadDispatcher.RunOnMainThread(() =>
            {
                World? world = GetActiveWorld();
                int modified = 0;
                if (world != null && world.IsCreated)
                {
                    modified = StatModifierSystem.ApplyImmediate(world.EntityManager, mod);
                }

                // Always enqueue for persistence across reloads (runs after MinFrameDelay guard).
                StatModifierSystem.Enqueue(mod);

                return new OverrideResult
                {
                    Success = modified >= 0, // -1 means unknown sdkPath, 0+ means applied
                    SdkPath = sdkPath,
                    Message = modified > 0
                        ? $"Applied {modeStr} override for {sdkPath} = {value} to {modified} entities"
                        : $"Enqueued {modeStr} override for {sdkPath} = {value} (no live entities yet)"
                };
            });
            OverrideResult result;
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            if (!overrideTask.Wait(MainThreadWaitTimeoutMs))
            {
                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleApplyOverride timed out ({MainThreadWaitTimeoutMs}ms) — dispatcher pump may be dead");
                result = new OverrideResult
                {
                    Success = false,
                    SdkPath = sdkPath,
                    Message = $"Timed out applying {modeStr} override for {sdkPath} (main-thread pump unresponsive)"
                };
            }
            else
            {
                // sync-over-async-unavoidable: ECS-bound, main-thread-required
                result = overrideTask.Result;
            }

            return JToken.FromObject(result);
        }

        private JToken HandleQueryEntities(JObject? parameters)
        {
            string? componentType = parameters?.Value<string>("componentType");
            string? category = parameters?.Value<string>("category");

            // Task #535: bounded wait — return empty result on timeout instead of hanging.
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            var queryTask = MainThreadDispatcher.RunOnMainThread(() =>
            {
                return QueryEntitiesOnMainThread(componentType, category);
            });
            QueryResult queryResult;
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            if (!queryTask.Wait(MainThreadWaitTimeoutMs))
            {
                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleQueryEntities timed out ({MainThreadWaitTimeoutMs}ms) — dispatcher pump may be dead (componentType='{componentType}' category='{category}')");
                queryResult = new QueryResult
                {
                    Count = 0
                };
            }
            else
            {
                // sync-over-async-unavoidable: ECS-bound, main-thread-required
                queryResult = queryTask.Result;
            }

            return JToken.FromObject(queryResult);
        }

        private JToken HandleReloadPacks(JObject? parameters)
        {
            ReloadResult reloadResult;
            if (!IsPlatformAlive)
            {
                reloadResult = new ReloadResult
                {
                    Success = false,
                    LoadedPacks = new List<string>(),
                    Errors = new List<string> { "ModPlatform not ready (scene transition in progress)." }
                };
                return JToken.FromObject(reloadResult);
            }
            try
            {
                // Pack loading involves file IO and registry updates.
                // Task #535: bounded wait — heavy I/O so use the heavy timeout.
                // sync-over-async-unavoidable: ECS-bound, main-thread-required
                var loadTask = MainThreadDispatcher.RunOnMainThread(() =>
                {
                    return _platform.LoadPacks();
                });
                // sync-over-async-unavoidable: ECS-bound, main-thread-required
                if (!loadTask.Wait(MainThreadHeavyWaitTimeoutMs))
                {
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleReloadPacks timed out ({MainThreadHeavyWaitTimeoutMs}ms) — dispatcher pump may be dead");
                    reloadResult = new ReloadResult
                    {
                        Success = false,
                        LoadedPacks = new List<string>(),
                        Errors = new List<string> { "Pack reload timed out (main-thread pump unresponsive)." }
                    };
                }
                else
                {
                    // sync-over-async-unavoidable: ECS-bound, main-thread-required
                    SDK.ContentLoadResult loadResult = loadTask.Result;
                    reloadResult = new ReloadResult
                    {
                        Success = loadResult.IsSuccess,
                        LoadedPacks = new List<string>(loadResult.LoadedPacks),
                        Errors = new List<string>(loadResult.Errors)
                    };
                }
            }
            catch (Exception ex)
            {
                reloadResult = new ReloadResult
                {
                    Success = false,
                    Errors = new List<string> { ex.Message }
                };
            }

            return JToken.FromObject(reloadResult);
        }

        private JToken HandleGetResources()
        {
            // Use an explicit timeout (consistent with HandleStatus) to avoid blocking
            // the bridge thread indefinitely if the main thread is busy.
            System.Threading.Tasks.Task<ResourceSnapshot> task = MainThreadDispatcher.RunOnMainThread(() =>
            {
                World? world = GetActiveWorld();
                if (world == null || !world.IsCreated)
                    return new ResourceSnapshot();
                return ResourceReader.ReadResources(world.EntityManager);
            });

            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            bool completed = task.Wait(MainThreadWaitTimeoutMs);
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            ResourceSnapshot snapshot = completed ? task.Result : new ResourceSnapshot();

            if (!completed)
                DebugLog.Write("GameBridgeServer", "[GameBridgeServer] HandleGetResources timed out waiting for main thread");

            return JToken.FromObject(snapshot);
        }

        private JToken HandleLoadScene(JObject? parameters)
        {
            string sceneName = parameters?.Value<string>("scene") ?? "level0";
            // JToken.Value<int> returns 0 when the key is absent — treat missing buildIndex as unset.
            int buildIndex = -1;
            if (parameters?["buildIndex"] is JToken buildIndexToken
                && buildIndexToken.Type != JTokenType.Null)
            {
                buildIndex = buildIndexToken.Value<int>();
            }

            // If scene is purely numeric, treat as build index
            if (buildIndex < 0 && int.TryParse(sceneName, out int parsed))
                buildIndex = parsed;

            var loadResult = MainThreadDispatcher.RunOnMainThread(() =>
            {
                int count = UnityEngine.SceneManagement.SceneManager.sceneCountInBuildSettings;
                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] LoadScene: buildIndex={buildIndex} sceneName={sceneName} totalScenes={count}");
                try
                {
                    if (buildIndex >= 0)
                        UnityEngine.SceneManagement.SceneManager.LoadScene(buildIndex);
                    else
                        UnityEngine.SceneManagement.SceneManager.LoadScene(sceneName);
                    return new { success = true, sceneCount = count };
                }
                catch (Exception ex)
                {
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] LoadScene failed: {ex.Message}");
                    return new { success = false, sceneCount = count };
                }
            });

            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            bool timedOut = !loadResult.Wait(MainThreadWaitTimeoutMs);
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            bool success = !timedOut && loadResult.Result.success;
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            int sceneCount = timedOut ? -1 : loadResult.Result.sceneCount;

            return JToken.FromObject(new { success, scene = sceneName, buildIndex, sceneCount });
        }

        private JToken HandleScreenshot(JObject? parameters)
        {
            string path = parameters?.Value<string>("path") ?? "";
            // Guard: if the path arg looks like a CLI flag (e.g. "--format=json" leaked from
            // the MCP Python wrapper), discard it and fall back to the default path.
            if (string.IsNullOrEmpty(path) || path.StartsWith("--", StringComparison.Ordinal))
            {
                path = Path.Combine(
                    BepInEx.Paths.BepInExRootPath,
                    "screenshots",
                    $"dinoforge_{DateTime.UtcNow:yyyyMMddTHHmmssZ}.png");
            }

            // Task #535: bounded wait. Screenshot can be a couple of frames long on first call.
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            var ssTask = MainThreadDispatcher.RunOnMainThread(() =>
            {
                try
                {
                    string? dir = Path.GetDirectoryName(path);
                    if (dir != null && !Directory.Exists(dir))
                        Directory.CreateDirectory(dir);

                    UnityEngine.ScreenCapture.CaptureScreenshot(path);

                    return new ScreenshotResult
                    {
                        Success = true,
                        Path = path,
                        Width = Screen.width,
                        Height = Screen.height
                    };
                }
                catch (Exception ex)
                {
                    // Pattern #104 (Task #302): structured logging instead of catch-swallow-default.
                    // Surface the failure path via WriteDebug (full Exception.ToString() per Pattern #96).
                    // ScreenshotResult.Success=false is preserved as the wire signal; the DTO is shared
                    // with Bridge.Protocol so we don't add a new field here, but the runtime log
                    // captures the exception type, message, and stack for diagnosis.
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleScreenshot failed for '{path}' ({ex.GetType().Name}): {ex}");
                    return new ScreenshotResult
                    {
                        Success = false,
                        Path = path
                    };
                }
            });
            ScreenshotResult ssResult;
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            if (!ssTask.Wait(MainThreadWaitTimeoutMs))
            {
                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleScreenshot timed out ({MainThreadWaitTimeoutMs}ms) — dispatcher pump may be dead");
                ssResult = new ScreenshotResult
                {
                    Success = false,
                    Path = path
                };
            }
            else
            {
                // sync-over-async-unavoidable: ECS-bound, main-thread-required
                ssResult = ssTask.Result;
            }

            return JToken.FromObject(ssResult);
        }

        private JToken HandleDumpState(JObject? parameters)
        {
            string? category = parameters?.Value<string>("category");

            // Rebuild the catalog for a fresh dump.
            // Task #535: bounded wait — heavy catalog rebuild on a busy ECS world, use heavy timeout.
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            var dumpTask = MainThreadDispatcher.RunOnMainThread(() =>
            {
                World? world = GetActiveWorld();
                if (world == null || !world.IsCreated)
                    return new CatalogSnapshot();

                VanillaCatalog freshCatalog = new VanillaCatalog();
                freshCatalog.Build(world.EntityManager);

                return BuildCatalogSnapshot(freshCatalog, category);
            });
            CatalogSnapshot snapshot;
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            if (!dumpTask.Wait(MainThreadHeavyWaitTimeoutMs))
            {
                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleDumpState timed out ({MainThreadHeavyWaitTimeoutMs}ms) — dispatcher pump may be dead");
                snapshot = new CatalogSnapshot();
            }
            else
            {
                // sync-over-async-unavoidable: ECS-bound, main-thread-required
                snapshot = dumpTask.Result;
            }

            return JToken.FromObject(snapshot);
        }

        private JToken HandleVerifyMod(JObject? parameters)
        {
            string packPath = parameters?.Value<string>("packPath") ?? "";
            VerifyResult verifyResult = new VerifyResult();

            if (string.IsNullOrEmpty(packPath))
            {
                verifyResult.Errors.Add("packPath is required");
                return JToken.FromObject(verifyResult);
            }

            try
            {
                SDK.PackLoader loader = new SDK.PackLoader();
                string manifestPath = packPath;
                if (Directory.Exists(packPath))
                {
                    manifestPath = Path.Combine(packPath, "pack.yaml");
                }

                if (!File.Exists(manifestPath))
                {
                    verifyResult.Errors.Add($"Manifest not found: {manifestPath}");
                    return JToken.FromObject(verifyResult);
                }

                SDK.PackManifest manifest = loader.LoadFromFile(manifestPath);
                verifyResult.PackId = manifest.Id;
                verifyResult.Loaded = true;

                // Report stat changes that would be applied
                verifyResult.StatChanges.Add($"Pack '{manifest.Id}' v{manifest.Version} verified successfully");
            }
            catch (Exception ex)
            {
                verifyResult.Errors.Add($"Verification failed: {ex.Message}");
            }

            return JToken.FromObject(verifyResult);
        }

        private JToken HandleWaitForWorld(JObject? parameters)
        {
            int timeoutMs = parameters?.Value<int?>("timeoutMs") ?? 30000;

            DateTimeOffset deadline = _timeProvider.GetUtcNow().AddMilliseconds(timeoutMs);
            while (_timeProvider.GetUtcNow() < deadline && _running)
            {
                if (IsPlatformAlive && _platform.IsWorldReady)
                {
                    string worldName = "";
                    try
                    {
                        // Task #535: bounded wait. The world is "ready" but the dispatcher pump
                        // can still be dead during scene transition — fall through to empty name.
                        // sync-over-async-unavoidable: ECS-bound, main-thread-required
                        var wnTask = MainThreadDispatcher.RunOnMainThread(() =>
                        {
                            World? world = GetActiveWorld();
                            return world?.Name ?? "";
                        });
                        // sync-over-async-unavoidable: ECS-bound, main-thread-required
                        if (wnTask.Wait(MainThreadWaitTimeoutMs))
                        {
                            // sync-over-async-unavoidable: ECS-bound, main-thread-required
                            worldName = wnTask.Result;
                        }
                        else
                        {
                            DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleWaitForWorld world-name read timed out ({MainThreadWaitTimeoutMs}ms) — dispatcher pump may be dead");
                        }
                    }
                    catch (Exception ex) { DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleWaitForWorld world-name read failed: {ex}"); }

                    WaitResult readyResult = new WaitResult
                    {
                        Ready = true,
                        WorldName = worldName
                    };
                    return JToken.FromObject(readyResult);
                }

                // sync-over-async-unavoidable: ManualResetEventSlim signal wait with ShutdownPollIntervalMs bounded timeout (not a Task)
                if (_shutdownEvent.Wait(ShutdownPollIntervalMs))  // Signaled = shutdown
                    break;
            }

            WaitResult timeoutResult = new WaitResult
            {
                Ready = false,
                WorldName = ""
            };
            return JToken.FromObject(timeoutResult);
        }

        // #923: Telemetry — return current MetricsCollector snapshot as JSON.
        private JToken HandleGetMetrics()
        {
            try
            {
                string json = MetricsCollector.Instance.DumpJson();
                return JToken.Parse(json);
            }
            catch (Exception ex)
            {
                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleGetMetrics failed: {ex.Message}");
                return JToken.FromObject(new { error = ex.Message });
            }
        }

        // ──────────────────────────────────────────────
        //  Helper Methods
        // ──────────────────────────────────────────────

        /// <summary>
        /// Reads stat values from the ECS world for a given component mapping.
        /// Must be called on the main thread.
        /// </summary>
        private StatResult ReadStatFromEcs(ComponentMapping mapping, int? entityIndex)
        {
            StatResult result = new StatResult
            {
                SdkPath = mapping.SdkModelPath,
                ComponentType = mapping.EcsComponentType,
                FieldName = mapping.TargetFieldName ?? ""
            };

            Type? clrType = mapping.ResolvedType;
            if (clrType == null)
            {
                result.EntityCount = 0;
                return result;
            }

            World? world = GetActiveWorld();
            if (world == null || !world.IsCreated)
                return result;

            EntityManager em = world.EntityManager;
            ComponentType? ct = EntityQueries.ResolveComponentType(mapping.EcsComponentType);
            if (ct == null) return result;

            EntityQueryDesc desc = new EntityQueryDesc
            {
                All = new[] { ct.Value }
            };
            EntityQuery query = em.CreateEntityQuery(desc);
            NativeArray<Entity> entities = query.ToEntityArray(Allocator.Temp);

            result.EntityCount = entities.Length;

            if (entities.Length == 0)
            {
                entities.Dispose();
                query.Dispose();
                return result;
            }

            MethodInfo? getMethod = typeof(EntityManager)
                .GetMethod("GetComponentData", new[] { typeof(Entity) });
            if (getMethod == null)
            {
                entities.Dispose();
                query.Dispose();
                return result;
            }

            MethodInfo genericGet = getMethod.MakeGenericMethod(clrType);
            string fieldName = mapping.TargetFieldName ?? "value";
            FieldInfo? field = clrType.GetField(fieldName,
                BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);

            if (field == null)
            {
                entities.Dispose();
                query.Dispose();
                return result;
            }

            result.Values = new List<float>();
            float sum = 0f;

            int start = entityIndex.HasValue ? entityIndex.Value : 0;
            int end = entityIndex.HasValue ? Math.Min(entityIndex.Value + 1, entities.Length) : entities.Length;

            for (int i = start; i < end; i++)
            {
                try
                {
                    object? data = genericGet.Invoke(em, new object[] { entities[i] });
                    if (data == null) continue;

                    object? rawValue = field.GetValue(data);
                    float floatVal = 0f;
                    if (rawValue is float f) floatVal = f;
                    else if (rawValue is int iv) floatVal = iv;

                    result.Values.Add(floatVal);
                    // event-lifecycle-ok: local float accumulator, not an event subscription
                    sum += floatVal;
                }
                catch { /* safe-swallow: per-entity reflection failure skips one entry but continues aggregation */ }
            }

            if (result.Values.Count > 0)
                result.Value = sum / result.Values.Count;

            entities.Dispose();
            query.Dispose();
            return result;
        }

        /// <summary>
        /// Queries entities on the main thread, optionally filtering by component type or category.
        /// </summary>
        private QueryResult QueryEntitiesOnMainThread(string? componentType, string? category)
        {
            QueryResult result = new QueryResult();

            World? world = GetActiveWorld();
            if (world == null || !world.IsCreated)
                return result;

            EntityManager em = world.EntityManager;

            if (!string.IsNullOrEmpty(componentType))
            {
                ComponentType? ct = EntityQueries.ResolveComponentType(componentType!);
                if (ct == null)
                {
                    result.Count = 0;
                    return result;
                }

                EntityQueryDesc desc = new EntityQueryDesc
                {
                    All = new[] { ct.Value }
                };
                EntityQuery query = em.CreateEntityQuery(desc);
                NativeArray<Entity> entities = query.ToEntityArray(Allocator.Temp);

                result.Count = entities.Length;

                // Return up to 100 entity summaries
                int limit = Math.Min(entities.Length, 100);
                for (int i = 0; i < limit; i++)
                {
                    EntityInfo info = new EntityInfo { Index = entities[i].Index };
                    try
                    {
                        NativeArray<ComponentType> types = em.GetComponentTypes(entities[i], Allocator.Temp);
                        for (int j = 0; j < types.Length; j++)
                        {
                            Type? managed = types[j].GetManagedType();
                            info.Components.Add(managed?.FullName ?? $"Unknown({types[j].TypeIndex})");
                        }
                        types.Dispose();
                    }
                    catch { /* safe-swallow: component-type enumeration per-entity is best-effort diagnostic */ }

                    result.Entities.Add(info);
                }

                entities.Dispose();
                query.Dispose();
            }
            else if (!string.IsNullOrEmpty(category))
            {
                // Use VanillaCatalog to filter by category
                VanillaCatalog? catalog = IsPlatformAlive ? _platform.Catalog : null;
                if (catalog != null && catalog.IsBuilt)
                {
                    IReadOnlyList<VanillaEntityInfo> list;
                    // null-forgiveness-ok: guarded by if (!string.IsNullOrEmpty(category)) above
                    switch (category!.ToLowerInvariant())
                    {
                        case "unit":
                            list = catalog.Units;
                            break;
                        case "building":
                            list = catalog.Buildings;
                            break;
                        case "projectile":
                            list = catalog.Projectiles;
                            break;
                        default:
                            list = catalog.Other;
                            break;
                    }

                    int totalCount = 0;
                    foreach (VanillaEntityInfo entry in list)
                    {
                        // event-lifecycle-ok: local int accumulator, not an event subscription
                        totalCount += entry.EntityCount;
                        EntityInfo info = new EntityInfo
                        {
                            Index = -1, // archetype-level, not individual entity
                            Components = new List<string>(entry.ComponentTypes)
                        };
                        result.Entities.Add(info);
                    }
                    result.Count = totalCount;
                }
            }
            else
            {
                // Return total entity count
                NativeArray<Entity> all = em.GetAllEntities(Allocator.Temp);
                result.Count = all.Length;
                all.Dispose();
            }

            return result;
        }

        /// <summary>
        /// Builds a CatalogSnapshot from a VanillaCatalog, optionally filtered by category.
        /// </summary>
        private static CatalogSnapshot BuildCatalogSnapshot(VanillaCatalog catalog, string? category)
        {
            CatalogSnapshot snapshot = new CatalogSnapshot();
            bool all = string.IsNullOrEmpty(category) ||
                        string.Equals(category, "all", StringComparison.OrdinalIgnoreCase);

            if (all || string.Equals(category, "unit", StringComparison.OrdinalIgnoreCase))
            {
                foreach (VanillaEntityInfo info in catalog.Units)
                {
                    snapshot.Units.Add(new DINOForge.Bridge.Protocol.CatalogEntry
                    {
                        InferredId = info.InferredId,
                        ComponentCount = info.ComponentTypes.Length,
                        EntityCount = info.EntityCount,
                        Category = info.Category
                    });
                }
            }

            if (all || string.Equals(category, "building", StringComparison.OrdinalIgnoreCase))
            {
                foreach (VanillaEntityInfo info in catalog.Buildings)
                {
                    snapshot.Buildings.Add(new DINOForge.Bridge.Protocol.CatalogEntry
                    {
                        InferredId = info.InferredId,
                        ComponentCount = info.ComponentTypes.Length,
                        EntityCount = info.EntityCount,
                        Category = info.Category
                    });
                }
            }

            if (all || string.Equals(category, "projectile", StringComparison.OrdinalIgnoreCase))
            {
                foreach (VanillaEntityInfo info in catalog.Projectiles)
                {
                    snapshot.Projectiles.Add(new DINOForge.Bridge.Protocol.CatalogEntry
                    {
                        InferredId = info.InferredId,
                        ComponentCount = info.ComponentTypes.Length,
                        EntityCount = info.EntityCount,
                        Category = info.Category
                    });
                }
            }

            if (all || string.Equals(category, "other", StringComparison.OrdinalIgnoreCase))
            {
                foreach (VanillaEntityInfo info in catalog.Other)
                {
                    snapshot.Other.Add(new DINOForge.Bridge.Protocol.CatalogEntry
                    {
                        InferredId = info.InferredId,
                        ComponentCount = info.ComponentTypes.Length,
                        EntityCount = info.EntityCount,
                        Category = info.Category
                    });
                }
            }

            return snapshot;
        }

        private JToken HandleStartGame(JObject? parameters)
        {
            string saveName = parameters?.Value<string>("saveName") ?? "";

            // Trigger the game's own world-loading system by creating the
            // BeginGameWorldLoadingSingleton ECS entity, which SceneLoadingSystem listens for.
            var result = MainThreadDispatcher.RunOnMainThread(() =>
            {
                try
                {
                    World? world = GetActiveWorld();
                    if (world == null || !world.IsCreated)
                        return new { success = false, message = "No ECS world" };

                    // Resolve BeginGameWorldLoadingSingleton type dynamically
                    Type? singletonType = null;
                    foreach (System.Reflection.Assembly asm in AppDomain.CurrentDomain.GetAssemblies())
                    {
                        singletonType = asm.GetType("Components.SingletonComponents.BeginGameWorldLoadingSingleton");
                        if (singletonType != null) break;
                    }

                    if (singletonType == null)
                        return new { success = false, message = "BeginGameWorldLoadingSingleton type not found" };

                    // Dump the singleton's fields for diagnostics
                    FieldInfo[] fields = singletonType.GetFields(BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);
                    string fieldList = string.Join(", ", System.Array.ConvertAll(fields, f => $"{f.FieldType.Name} {f.Name}"));
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] BeginGameWorldLoadingSingleton fields: [{fieldList}]");

                    ComponentType ct = ComponentType.ReadWrite(singletonType);
                    Entity e = world.EntityManager.CreateEntity(ct);
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Created BeginGameWorldLoadingSingleton entity {e.Index}");

                    // If the singleton has a NameToLoad field, try to set it via reflection
                    // (ECS components are structs so we use SetComponentData via reflection)
                    if (!string.IsNullOrEmpty(saveName))
                    {
                        FieldInfo? nameField = singletonType.GetField("NameToLoad",
                            BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);
                        DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] NameToLoad field: {(nameField == null ? "not found" : nameField.FieldType.Name)}");
                    }

                    return new { success = true, message = $"Created singleton entity {e.Index}, fields=[{fieldList}]" };
                }
                catch (Exception ex)
                {
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleStartGame failed: {ex.Message}");
                    return new { success = false, message = ex.Message };
                }
            });

            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            bool completed = result.Wait(MainThreadWaitTimeoutMs);
            if (!completed) return JToken.FromObject(new { success = false, message = "Timed out" });
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            return JToken.FromObject(result.Result);
        }

        private JToken HandleDismissLoadScreen()
        {
            var result = MainThreadDispatcher.RunOnMainThread(() =>
            {
                try
                {
                    // DINO's loading screen uses UI.LoadingProgressBar which has a _startAction field
                    // (a UnityAction) that gets invoked when the player presses any key.
                    var allMBs = UnityEngine.Object.FindObjectsOfType<MonoBehaviour>();
                    string found = "";
                    foreach (var mb in allMBs)
                    {
                        if (mb == null) continue;
                        string tName = mb.GetType().Name;
                        // event-lifecycle-ok: local string accumulator for diagnostic dump, not an event subscription
                        found += $"[{tName}]";

                        // Target: UI.LoadingProgressBar
                        if (tName == "LoadingProgressBar")
                        {
                            // Try _startAction field (UnityAction)
                            FieldInfo? startField = mb.GetType().GetField("_startAction",
                                BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);
                            if (startField != null)
                            {
                                object? action = startField.GetValue(mb);
                                if (action is UnityEngine.Events.UnityAction ua)
                                {
                                    ua.Invoke();
                                    return new { success = true, message = $"Invoked _startAction on LoadingProgressBar" };
                                }
                                // Try invoking as delegate
                                if (action is System.Delegate del)
                                {
                                    del.DynamicInvoke();
                                    return new { success = true, message = $"DynamicInvoked _startAction on LoadingProgressBar" };
                                }
                                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] _startAction type: {(action?.GetType().Name ?? "null")}");
                            }

                            // Fallback: call Update() to simulate time passing with anyKeyDown
                            // Actually try GetComponent on the progress GameObject
                            FieldInfo? progressField = mb.GetType().GetField("_progress",
                                BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);
                            if (progressField != null)
                            {
                                UnityEngine.GameObject? progressGO = progressField.GetValue(mb) as UnityEngine.GameObject;
                                if (progressGO != null)
                                    progressGO.SetActive(false); // hide progress bar panel
                            }

                            // Try destroying the component to let the scene proceed
                            return new { success = false, message = $"LoadingProgressBar found but _startAction invoke failed. Action type: {startField?.GetValue(mb)?.GetType().Name ?? "null"}" };
                        }
                    }

                    return new { success = false, message = $"No dismiss handler found. MBs: {found}" };
                }
                catch (Exception ex)
                {
                    // Pattern #104 (Task #302): preserve type info in wire message + full stack in log.
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] DismissDialog handler failed: {ex}");
                    return new { success = false, message = $"{ex.GetType().Name}: {ex.Message}" };
                }
            });

            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            bool completed = result.Wait(MainThreadWaitTimeoutMs);
            if (!completed) return JToken.FromObject(new { success = false, message = "Timed out" });
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            return JToken.FromObject(result.Result);
        }

        /// <summary>
        /// Injects a key press via Win32 SendInput (same path as MCP game input tools).
        /// Parameter <c>key</c> defaults to Escape for pause-menu tests.
        /// For Escape, also runs <see cref="PauseMenuBridgeHelper"/> on the main thread when Win32 fails
        /// or the pause UI is still hidden.
        /// </summary>
        private JToken HandleSimulateKey(JObject? parameters)
        {
            string key = parameters?.Value<string>("key") ?? "Escape";
            bool win32Ok = Win32KeyInput.TrySendKey(key, out string win32Message);

            if (!IsEscapeKey(key))
            {
                DebugLog.Write("GameBridgeServer",
                    $"[GameBridgeServer] HandleSimulateKey key='{key}' ok={win32Ok} msg={win32Message}");
                return JToken.FromObject(new { success = win32Ok, message = win32Message });
            }

            return HandleEscapePauseOpen(win32Ok, win32Message);
        }

        private JToken HandleTogglePauseMenu(JObject? _)
        {
            var result = MainThreadDispatcher.RunOnMainThread(() =>
            {
                (bool opened, string message) = PauseMenuBridgeHelper.TryOpenPauseMenu();
                bool visible = PauseMenuBridgeHelper.IsPauseMenuVisible();
                return new
                {
                    success = visible,
                    message,
                    pauseVisible = visible,
                    opened,
                };
            });

            bool completed = result.Wait(MainThreadInputWaitTimeoutMs);
            if (!completed)
            {
                return JToken.FromObject(new { success = false, message = "Timed out" });
            }

            DebugLog.Write("GameBridgeServer",
                $"[GameBridgeServer] HandleTogglePauseMenu result={result.Result}");
            return JToken.FromObject(result.Result);
        }

        private JToken HandleEscapePauseOpen(bool win32Ok, string win32Message)
        {
            var pauseResult = MainThreadDispatcher.RunOnMainThread(() =>
            {
                (bool _, string openMessage) = PauseMenuBridgeHelper.TryOpenPauseMenu();
                bool visible = PauseMenuBridgeHelper.IsPauseMenuVisible();
                return (visible, openMessage);
            });

            bool pauseCompleted = pauseResult.Wait(MainThreadInputWaitTimeoutMs);
            bool pauseVisible = false;
            string pauseMessage = "main-thread pause open timed out";
            if (pauseCompleted)
            {
                (bool visible, string openMessage) = pauseResult.Result;
                pauseVisible = visible;
                pauseMessage = openMessage;
            }

            bool success = pauseVisible || win32Ok;
            string message = pauseVisible
                ? $"pause menu visible; win32={win32Ok} ({win32Message}); {pauseMessage}"
                : $"win32={win32Ok} ({win32Message}); {pauseMessage}";

            DebugLog.Write("GameBridgeServer",
                $"[GameBridgeServer] HandleSimulateKey Escape success={success} pauseVisible={pauseVisible} msg={message}");
            return JToken.FromObject(new { success, message, pauseVisible });
        }

        private static bool IsEscapeKey(string key) =>
            string.Equals(key, "Escape", StringComparison.OrdinalIgnoreCase)
            || string.Equals(key, "Esc", StringComparison.OrdinalIgnoreCase);

        private JToken HandlePressKey(JObject? parameters)
        {
            // scanScene: dump all active MonoBehaviours + their public/private void methods
            // filter: optional substring filter on type name
            string filter = parameters?.Value<string>("filter") ?? "";

            var result = MainThreadDispatcher.RunOnMainThread(() =>
            {
                try
                {
                    var allMBs = Resources.FindObjectsOfTypeAll<MonoBehaviour>();
                    var sb = new System.Text.StringBuilder();
                    foreach (var mb in allMBs)
                    {
                        if (mb == null || !mb.gameObject.activeInHierarchy) continue;
                        string tName = mb.GetType().Name;
                        if (!string.IsNullOrEmpty(filter) &&
                            tName.IndexOf(filter, StringComparison.OrdinalIgnoreCase) < 0 &&
                            mb.gameObject.name.IndexOf(filter, StringComparison.OrdinalIgnoreCase) < 0)
                            continue;

                        // List void methods with 0 params
                        var methods = mb.GetType().GetMethods(
                            BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance)
                            .Where(m => m.ReturnType == typeof(void) && m.GetParameters().Length == 0)
                            .Select(m => m.Name)
                            .Where(n => !n.StartsWith("get_") && !n.StartsWith("set_") && n != "Finalize")
                            .Take(8);
                        sb.AppendLine($"[{mb.gameObject.name}] {tName}: {string.Join(", ", methods)}");
                    }
                    string output = sb.Length > 0 ? sb.ToString().Substring(0, Math.Min(2000, sb.Length)) : "No matches";
                    return new { success = true, message = output };
                }
                catch (Exception ex)
                {
                    // Pattern #104 (Task #302): preserve type info in wire message + full stack in log.
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] PressKey/scanScene failed: {ex}");
                    return new { success = false, message = $"{ex.GetType().Name}: {ex.Message}" };
                }
            });

            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            bool completed = result.Wait(MainThreadInputWaitTimeoutMs);
            if (!completed) return JToken.FromObject(new { success = false, message = "Timed out" });
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            return JToken.FromObject(result.Result);
        }

        /// <summary>
        /// Invokes a named void(0-param) method on any MonoBehaviour whose type name or
        /// gameObject name contains <c>target</c>. Use to call dialog confirm handlers, etc.
        /// </summary>
        private JToken HandleInvokeMethod(JObject? parameters)
        {
            string target = parameters?.Value<string>("target") ?? "";
            string method = parameters?.Value<string>("method") ?? "";

            var result = MainThreadDispatcher.RunOnMainThread(() =>
            {
                try
                {
                    if (string.IsNullOrEmpty(target) || string.IsNullOrEmpty(method))
                        return new { success = false, message = "Provide target (type/go name) and method" };

                    var allMBs = Resources.FindObjectsOfTypeAll<MonoBehaviour>();
                    var invoked = new List<string>();
                    foreach (var mb in allMBs)
                    {
                        if (mb == null) continue;
                        string tName = mb.GetType().Name;
                        string goName = mb.gameObject.name;
                        bool matches = tName.IndexOf(target, StringComparison.OrdinalIgnoreCase) >= 0 ||
                                       goName.IndexOf(target, StringComparison.OrdinalIgnoreCase) >= 0;
                        if (!matches) continue;

                        if (!mb.gameObject.activeInHierarchy)
                        {
                            Transform? current = mb.transform;
                            while (current != null)
                            {
                                if (!current.gameObject.activeSelf)
                                {
                                    current.gameObject.SetActive(true);
                                }

                                current = current.parent;
                            }
                        }

                        MethodInfo? mi = mb.GetType().GetMethod(method,
                            BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance,
                            null, Type.EmptyTypes, null);
                        if (mi == null) continue;

                        mi.Invoke(mb, null);
                        invoked.Add($"{tName}.{method}()");
                        DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] InvokeMethod: {tName}.{method}()");
                    }

                    if (invoked.Count == 0)
                        return new { success = false, message = $"No active MonoBehaviour matching '{target}' with method '{method}' found" };

                    return new { success = true, message = $"Invoked: {string.Join(", ", invoked)}" };
                }
                catch (Exception ex)
                {
                    // Pattern #104 (Task #302): preserve type info in wire message + full stack in log.
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] InvokeMethod target='{target}' method='{method}' failed: {ex}");
                    return new { success = false, message = $"{ex.GetType().Name}: {ex.Message}" };
                }
            });

            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            bool completed = result.Wait(MainThreadWaitTimeoutMs);
            if (!completed) return JToken.FromObject(new { success = false, message = "Timed out" });
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            return JToken.FromObject(result.Result);
        }

        /// <summary>
        /// Clicks a named Unity UI button. Pass buttonName="DINOForge_ModsButton" to test
        /// the injected Mods button, or any other button name visible in the active scene.
        /// </summary>
        private JToken HandleClickButton(JObject? parameters)
        {
            string buttonName = parameters?.Value<string>("buttonName") ?? "";

            var result = MainThreadDispatcher.RunOnMainThread(() =>
            {
                try
                {
                    var allButtons = Resources.FindObjectsOfTypeAll<UnityEngine.UI.Button>();
                    var summary = new System.Text.StringBuilder();
                    UnityEngine.UI.Button? target = null;

                    foreach (var btn in allButtons)
                    {
                        if (btn == null) continue;
                        if (!btn.gameObject.activeInHierarchy) continue;
                        string name = btn.name;
                        var txt = btn.GetComponentInChildren<UnityEngine.UI.Text>();
                        var tmptxt = btn.GetComponentInChildren<TMPro.TMP_Text>();
                        string label = (txt?.text ?? tmptxt?.text ?? "").Trim();
                        summary.Append($"[{name}:'{label}'] ");

                        if (string.IsNullOrEmpty(buttonName))
                            continue; // just listing

                        if (name == buttonName ||
                            name.Equals(buttonName, StringComparison.OrdinalIgnoreCase) ||
                            label.Equals(buttonName, StringComparison.OrdinalIgnoreCase))
                        {
                            target = btn;
                        }
                    }

                    if (string.IsNullOrEmpty(buttonName))
                        return new { success = true, message = $"Buttons: {summary.ToString().Substring(0, Math.Min(800, summary.Length))}" };

                    if (target == null)
                        return new { success = false, message = $"Button '{buttonName}' not found. Active buttons: {summary.ToString().Substring(0, Math.Min(600, summary.Length))}" };

                    // Primary: onClick.Invoke() fires the UnityEvent directly (works for modal dialogs)
                    // Guard against NRE inside button listeners that call EventSystem.current internally
                    try
                    {
                        target.onClick.Invoke();
                    }
                    catch (Exception onClickEx)
                    {
                        DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] onClick.Invoke NRE on '{target.name}', falling back to pointer sim: {onClickEx.Message}");
                    }

                    // Secondary: also fire pointer click for components that listen to IPointerClickHandler
                    // Guard EventSystem.current null — absent on main menu scenes with only TMP buttons (#NRE-fix)
                    var es = UnityEngine.EventSystems.EventSystem.current;
                    if (es != null)
                    {
                        try
                        {
                            UnityEngine.EventSystems.ExecuteEvents.Execute(
                                target.gameObject,
                                new UnityEngine.EventSystems.PointerEventData(es),
                                UnityEngine.EventSystems.ExecuteEvents.pointerClickHandler);
                        }
                        catch (Exception execEx)
                        {
                            DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] ExecuteEvents NRE on '{target.name}': {execEx.Message}");
                        }
                    }

                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Clicked button: {target.name}");
                    return new { success = true, message = $"Clicked '{target.name}'" };
                }
                catch (Exception ex)
                {
                    // Pattern #104 (Task #302): preserve type info in wire message + full stack in log.
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] ClickButton '{buttonName}' failed: {ex}");
                    return new { success = false, message = $"{ex.GetType().Name}: {ex.Message}" };
                }
            });

            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            // Use heavy timeout (30s) — scene-loading buttons like "continue" block main thread for 5-20s
            bool completed = result.Wait(30000);
            if (!completed)
            {
                // Fire-and-forget succeeded: scene load is in progress, return dispatched status
                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] ClickButton '{buttonName}': scene load in progress (fire-and-forget)");
                return JToken.FromObject(new { success = true, message = $"Dispatched '{buttonName}' (scene loading)" });
            }
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            return JToken.FromObject(result.Result);
        }

        /// <summary>
        /// Toggles DINOForge UI panels. target="modmenu" (F10 equivalent) or "debug" (F9 equivalent).
        /// Finds DFCanvas via MonoBehaviour reflection and calls ToggleModMenu()/ToggleDebug().
        /// Falls back to ModMenuOverlay.Toggle() if DFCanvas is not available.
        /// </summary>
        private JToken HandleToggleUi(JObject? parameters)
        {
            string target = (parameters?.Value<string>("target") ?? "modmenu").ToLowerInvariant();

            var result = MainThreadDispatcher.RunOnMainThread(() =>
            {
                try
                {
                    // Prefer the live DFCanvas owned by RuntimeDriver (avoids stale DontDestroyOnLoad copies).
                    var allMBs = Resources.FindObjectsOfTypeAll<MonoBehaviour>();
                    DFCanvas? dfCanvas = null;
                    MonoBehaviour? debugOverlay = null;
                    MonoBehaviour? modMenuOverlay = null;

                    foreach (var mb in allMBs)
                    {
                        if (mb == null) continue;
                        if (mb is RuntimeDriver driver
                            && driver._dfCanvas != null
                            && driver._dfCanvas.gameObject.activeInHierarchy)
                        {
                            dfCanvas = driver._dfCanvas;
                            break;
                        }
                    }

                    foreach (var mb in allMBs)
                    {
                        if (mb == null) continue;
                        string tName = mb.GetType().Name;
                        if (dfCanvas == null && tName == "DFCanvas" && mb.gameObject.activeInHierarchy)
                        {
                            dfCanvas = mb as DFCanvas;
                        }
                        else if (tName == "DebugOverlayBehaviour") debugOverlay = mb;
                        else if (tName == "ModMenuOverlay") modMenuOverlay = mb;
                    }

                    // Try DFCanvas first (UGUI path)
                    if (dfCanvas != null)
                    {
                        if (target == "debug")
                        {
                            dfCanvas.ToggleDebug();
                        }
                        else
                        {
                            dfCanvas.ToggleModMenu();
                        }

                        string methodName = target == "debug" ? "ToggleDebug" : "ToggleModMenu";
                        DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] ToggleUi: called DFCanvas.{methodName}");
                        return new { success = true, message = $"DFCanvas.{methodName}() invoked" };
                    }

                    // Fallback: ModMenuOverlay.Toggle() / DebugOverlayBehaviour.Toggle()
                    MonoBehaviour? fallback = target == "debug" ? debugOverlay : modMenuOverlay;
                    if (fallback != null)
                    {
                        MethodInfo? toggleMethod = fallback.GetType().GetMethod("Toggle",
                            BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);
                        if (toggleMethod != null)
                        {
                            toggleMethod.Invoke(fallback, null);
                            DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] ToggleUi: called {fallback.GetType().Name}.Toggle()");
                            return new { success = true, message = $"{fallback.GetType().Name}.Toggle() invoked" };
                        }
                    }

                    // Last resort: find any active component whose name contains the target
                    string sbAll = string.Join(", ", Array.ConvertAll(allMBs, mb => mb?.GetType().Name ?? "null"));
                    return new { success = false, message = $"No UI handler found for '{target}'. MBs: {sbAll.Substring(0, Math.Min(400, sbAll.Length))}" };
                }
                catch (Exception ex)
                {
                    // Pattern #104 (Task #302): preserve type info in wire message + full stack in log.
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] ToggleUi target='{target}' failed: {ex}");
                    return new { success = false, message = $"{ex.GetType().Name}: {ex.Message}" };
                }
            });

            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            bool completed = result.Wait(MainThreadWaitTimeoutMs);
            if (!completed) return JToken.FromObject(new { success = false, message = "Timed out" });
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            return JToken.FromObject(result.Result);
        }

        private JToken HandleListSaves()
        {
            var result = MainThreadDispatcher.RunOnMainThread(() =>
            {
                try
                {
                    // Find save manager via reflection
                    Type? saveManagerType = null;
                    foreach (var asm in AppDomain.CurrentDomain.GetAssemblies())
                    {
                        foreach (string typeName in new[] {
                            "Systems.SaveLoadSystem", "Systems.GameWorldLoaderSystem",
                            "Systems.Save.SaveSystem", "Systems.SaveSystem",
                            "SaveManager", "SaveSystem"
                        })
                        {
                            saveManagerType = asm.GetType(typeName);
                            if (saveManagerType != null) break;
                        }
                        if (saveManagerType != null) break;
                    }

                    // Search for saves in DNO's actual paths
                    string persistPath = Application.persistentDataPath;
                    var saves = new List<string>();

                    // DINO saves: persistentDataPath/DNOPersistentData/<branch>/
                    string dnoDataDir = System.IO.Path.Combine(persistPath, "DNOPersistentData");
                    string saveDir = dnoDataDir;
                    if (System.IO.Directory.Exists(dnoDataDir))
                    {
                        foreach (string branchDir in System.IO.Directory.GetDirectories(dnoDataDir))
                        {
                            string branchName = System.IO.Path.GetFileName(branchDir);
                            foreach (var f in System.IO.Directory.GetFiles(branchDir, "*.dat"))
                                saves.Add($"{branchName}/{System.IO.Path.GetFileNameWithoutExtension(f)}");
                        }
                    }
                    else
                    {
                        // Fallback to standard Saves dir
                        saveDir = System.IO.Path.Combine(persistPath, "Saves");
                        if (System.IO.Directory.Exists(saveDir))
                        {
                            foreach (var f in System.IO.Directory.GetFiles(saveDir, "*.sav"))
                                saves.Add(System.IO.Path.GetFileNameWithoutExtension(f));
                            foreach (var f in System.IO.Directory.GetFiles(saveDir, "*.dat"))
                                saves.Add(System.IO.Path.GetFileNameWithoutExtension(f));
                        }
                    }

                    return new
                    {
                        saveManagerType = saveManagerType?.FullName ?? "not found",
                        persistentDataPath = persistPath,
                        saveDir = saveDir,
                        saveDirExists = System.IO.Directory.Exists(saveDir),
                        saves = saves,
                        dataPath = Application.dataPath
                    };
                }
                catch (Exception ex)
                {
                    return new
                    {
                        saveManagerType = "error",
                        persistentDataPath = "",
                        saveDir = "",
                        saveDirExists = false,
                        saves = new List<string>(),
                        dataPath = ex.Message
                    };
                }
            });

            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            bool completed = result.Wait(MainThreadWaitTimeoutMs);
            if (!completed) return JToken.FromObject(new { error = "Timed out" });
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            return JToken.FromObject(result.Result);
        }

        private JToken HandleLoadSave(JObject? parameters)
        {
            string saveName = parameters?.Value<string>("saveName") ?? "AutoSave_1";

            var result = MainThreadDispatcher.RunOnMainThread(() =>
            {
                try
                {
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleLoadSave: '{saveName}'");

                    // Strategy 0: Create a LoadRequest ECS entity — the game's SaveLoadSystem
                    // reads Components.RawComponents.LoadRequest singletons and triggers a load.
                    // Fields: NameToLoad (FixedString128Bytes), FromMenu (Boolean)
                    World? world = GetActiveWorld();
                    if (world != null && world.IsCreated)
                    {
                        Type? loadRequestType = null;
                        foreach (var asm in AppDomain.CurrentDomain.GetAssemblies())
                        {
                            loadRequestType = asm.GetType("Components.RawComponents.LoadRequest");
                            if (loadRequestType != null) break;
                        }

                        if (loadRequestType != null)
                        {
                            DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Found LoadRequest type: {loadRequestType.FullName}");

                            // Create the component value
                            object loadRequest = System.Activator.CreateInstance(loadRequestType)
                                ?? throw new InvalidOperationException($"Could not create instance of {loadRequestType.FullName}.");

                            // Set NameToLoad — it's a Unity.Collections.FixedString128Bytes
                            FieldInfo? nameField = loadRequestType.GetField("NameToLoad",
                                BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);
                            FieldInfo? fromMenuField = loadRequestType.GetField("FromMenu",
                                BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance);

                            if (nameField != null)
                            {
                                // FixedString128Bytes can be set from a regular string via implicit conversion
                                // We need to box/unbox correctly
                                Type fsType = nameField.FieldType; // Unity.Collections.FixedString128Bytes
                                // Try to create FixedString128Bytes from string
                                try
                                {
                                    // FixedString128Bytes has implicit operator from string in Unity
                                    MethodInfo? op = fsType.GetMethod("op_Implicit",
                                        BindingFlags.Public | BindingFlags.Static,
                                        null, new[] { typeof(string) }, null);
                                    if (op != null)
                                    {
                                        object? fs = op.Invoke(null, new object[] { saveName });
                                        nameField.SetValue(loadRequest, fs);
                                        DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Set NameToLoad = '{saveName}' via op_Implicit");
                                    }
                                    else
                                    {
                                        // Try ctor with string
                                        System.Reflection.ConstructorInfo? ctor = fsType.GetConstructor(new[] { typeof(string) });
                                        if (ctor != null)
                                        {
                                            object? fs = ctor.Invoke(new object[] { saveName });
                                            nameField.SetValue(loadRequest, fs);
                                            DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Set NameToLoad via ctor");
                                        }
                                        else
                                        {
                                            DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] No string ctor or op_Implicit for {fsType.Name}");
                                        }
                                    }
                                }
                                catch (Exception ex)
                                {
                                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] NameToLoad set failed: {ex.Message}");
                                }
                            }

                            if (fromMenuField != null)
                                fromMenuField.SetValue(loadRequest, true);

                            // Create entity and add LoadRequest component
                            try
                            {
                                ComponentType ct = ComponentType.ReadWrite(loadRequestType);
                                Entity e = world.EntityManager.CreateEntity(ct);

                                // Set the component data via reflection
                                MethodInfo? setComp = typeof(EntityManager).GetMethod("SetComponentData",
                                    BindingFlags.Public | BindingFlags.Instance);
                                if (setComp != null)
                                {
                                    MethodInfo genSet = setComp.MakeGenericMethod(loadRequestType);
                                    genSet.Invoke(world.EntityManager, new object?[] { e, loadRequest });
                                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Created LoadRequest entity {e.Index} with NameToLoad='{saveName}'");
                                    return new { success = true, message = $"Created LoadRequest entity {e.Index} NameToLoad='{saveName}'", foundPath = "" };
                                }
                            }
                            catch (Exception ex)
                            {
                                DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] LoadRequest entity creation failed: {ex.Message}");
                            }
                        }
                        else
                        {
                            DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] LoadRequest type NOT found");
                        }
                    }

                    // Find the save file in DINO's DNOPersistentData structure
                    string persistPath = Application.persistentDataPath;
                    string dnoDataDir = System.IO.Path.Combine(persistPath, "DNOPersistentData");

                    string foundPath = "";
                    if (System.IO.Directory.Exists(dnoDataDir))
                    {
                        foreach (string branchDir in System.IO.Directory.GetDirectories(dnoDataDir))
                        {
                            foreach (string f in System.IO.Directory.GetFiles(branchDir, "*.dat"))
                            {
                                string fn = System.IO.Path.GetFileNameWithoutExtension(f).ToUpperInvariant();
                                string sn = saveName.ToUpperInvariant();
                                if (fn.Contains(sn) || sn.Contains(fn))
                                {
                                    foundPath = f;
                                    break;
                                }
                            }
                            if (!string.IsNullOrEmpty(foundPath)) break;
                        }
                    }

                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Save file found: '{foundPath}'");
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] PersistentDataPath: {persistPath}");

                    // Strategy 3: Find the game's native UI buttons via Unity's UI system
                    // Use Resources.FindObjectsOfTypeAll to find ALL button instances including inactive
                    var allButtons = Resources.FindObjectsOfTypeAll<UnityEngine.UI.Button>();
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Found {allButtons.Length} buttons (Resources.FindObjectsOfTypeAll)");

                    // Also try FindObjectsOfType (scene-only)
                    var sceneButtons = UnityEngine.Object.FindObjectsOfType<UnityEngine.UI.Button>();
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Found {sceneButtons.Length} buttons (FindObjectsOfType scene-only)");

                    // Dump ALL GameObjects to find what the menu uses
                    if (allButtons.Length == 0 && sceneButtons.Length == 0)
                    {
                        // Search for any MonoBehaviour with "Click" or "Button" in name
                        var allMBs = UnityEngine.Object.FindObjectsOfType<MonoBehaviour>();
                        var interesting = new System.Text.StringBuilder();
                        foreach (var mb in allMBs)
                        {
                            if (mb == null) continue;
                            string tName = mb.GetType().Name;
                            if (tName.Contains("Button") || tName.Contains("Click") || tName.Contains("Menu") || tName.Contains("Interactable"))
                                interesting.Append($"[{tName}:{mb.gameObject.name}] ");
                        }
                        DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Button-like MonoBehaviours: {interesting}");
                    }

                    var saveNameUpper = saveName.ToUpperInvariant();
                    UnityEngine.UI.Button? targetButton = null;
                    UnityEngine.UI.Button? continueButton = null;
                    UnityEngine.UI.Button? okButton = null;
                    var buttonSummary = new System.Text.StringBuilder();

                    foreach (var btn in allButtons)
                    {
                        if (btn == null) continue;
                        // Skip the DINOForge mods button only
                        if (btn.name == "DINOForge_ModsButton") continue;
                        // Skip inactive
                        if (!btn.gameObject.activeInHierarchy) continue;

                        var txt = btn.GetComponentInChildren<UnityEngine.UI.Text>();
                        var tmptxt = btn.GetComponentInChildren<TMPro.TMP_Text>();
                        string label = (txt?.text ?? tmptxt?.text ?? "").Trim();
                        string btnName = btn.name;
                        buttonSummary.Append($"[{btnName}:'{label}'] ");

                        string labelUpper = label.ToUpperInvariant();
                        string nameUpper = btnName.ToUpperInvariant();

                        if (labelUpper == "OK" && nameUpper == "BUTTON_INTERCEPTED")
                        {
                            // Only capture unnamed "Button" as OK — not named buttons like Continue
                            if (okButton == null) okButton = btn;
                        }
                        string nameBase = btnName.Replace("_intercepted", "").ToUpperInvariant();
                        if (nameBase == "CONTINUE" || labelUpper == "CONTINUE")
                        {
                            continueButton = btn;
                        }
                        if (!string.IsNullOrEmpty(saveNameUpper))
                        {
                            // Match save name against button label or name
                            if (labelUpper.Contains(saveNameUpper) || nameBase.Contains(saveNameUpper))
                            {
                                targetButton = btn;
                            }
                            // Special: if searching for CONTINUE, match the Continue button
                            if (saveNameUpper == "CONTINUE" && (nameBase == "CONTINUE" || labelUpper == "CONTINUE"))
                                targetButton = btn;
                            // Special: if searching for OK or CONFIRM, match the ok button
                            if ((saveNameUpper == "OK" || saveNameUpper == "CONFIRM") && labelUpper == "OK")
                                targetButton = btn;
                            // Match Load buttons: LOAD_1, LOAD buttons by date position
                            if (saveNameUpper.StartsWith("LOAD") && nameBase == "LOAD")
                            {
                                if (targetButton == null) targetButton = btn; // first Load button
                            }
                        }
                    }

                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Active buttons: {buttonSummary}");
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] okButton={okButton?.name ?? "null"} continueButton={continueButton?.name ?? "null"} targetButton={targetButton?.name ?? "null"}");

                    // Priority: explicit name match > CONTINUE > OK fallback
                    UnityEngine.UI.Button? toInvoke = targetButton ?? continueButton ?? okButton;
                    if (toInvoke != null)
                    {
                        DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] Invoking button: {toInvoke.name}");
                        // Try ExecuteEvents for proper UI simulation, fall back to onClick.Invoke
                        try
                        {
                            UnityEngine.EventSystems.ExecuteEvents.Execute(
                                toInvoke.gameObject,
                                new UnityEngine.EventSystems.PointerEventData(UnityEngine.EventSystems.EventSystem.current),
                                UnityEngine.EventSystems.ExecuteEvents.pointerClickHandler);
                        }
                        catch
                        {
                            toInvoke.onClick.Invoke();
                        }
                        return new { success = true, message = $"Invoked button '{toInvoke.name}' (label search: '{saveName}')", foundPath };
                    }

                    return new { success = false, message = $"No suitable button found for '{saveName}'. Active buttons: {buttonSummary}. Save path: '{foundPath}'", foundPath };
                }
                catch (Exception ex)
                {
                    DebugLog.Write("GameBridgeServer", $"[GameBridgeServer] HandleLoadSave failed: {ex.Message}");
                    return new { success = false, message = ex.Message, foundPath = "" };
                }
            });

            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            bool completed = result.Wait(MainThreadHeavyWaitTimeoutMs);
            if (!completed) return JToken.FromObject(new { success = false, message = "Timed out" });
            // sync-over-async-unavoidable: ECS-bound, main-thread-required
            return JToken.FromObject(result.Result);
        }

        /// <summary>
        /// Reads a single UTF-8 line from the pipe byte-by-byte.
        /// Returns null if the pipe is closed. This avoids StreamReader buffering
        /// issues on Mono where a large buffer causes blocking on partial reads.
        /// </summary>
        private static string? ReadLineFromPipe(Stream pipe)
        {
            var sb = new System.Text.StringBuilder();
            int b;
            while ((b = pipe.ReadByte()) != -1)
            {
                char c = (char)b;
                if (c == '\n') return sb.ToString();
                if (c != '\r') sb.Append(c);
            }
            return sb.Length > 0 ? sb.ToString() : null;
        }

        /// <summary>
        /// Serializes a successful JSON-RPC response.
        /// </summary>
        private static string SerializeSuccess(string? id, JToken result)
        {
            JsonRpcResponse response = new JsonRpcResponse
            {
                Id = id,
                Result = result
            };
            return JsonConvert.SerializeObject(response, Formatting.None);
        }

        /// <summary>
        /// Serializes a JSON-RPC error response.
        /// </summary>
        private static string SerializeError(string? id, int code, string message)
        {
            JsonRpcResponse response = new JsonRpcResponse
            {
                Id = id,
                Error = new JsonRpcError
                {
                    Code = code,
                    Message = message
                }
            };
            return JsonConvert.SerializeObject(response, Formatting.None);
        }

        /// <summary>
        /// Returns the ECS world to use for entity queries.
        ///
        /// After scene transitions, KeyInputSystem may live in a different world than
        /// GetActiveWorld() (because OnCreate fires before the default
        /// world is set). We query DefaultGameObjectInjectionWorld first (has all game entities).
        /// If that's null (startup edge case), we scan all worlds to find one with entities.
        /// </summary>
        private static World? GetActiveWorld()
        {
            World? preferred = World.DefaultGameObjectInjectionWorld;
            if (preferred != null && preferred.IsCreated)
                return preferred;
            return null;
        }

    }
}
