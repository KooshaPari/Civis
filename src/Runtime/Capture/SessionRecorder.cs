#nullable enable
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using UnityEngine;
using UnityEngine.EventSystems;
using UnityEngine.SceneManagement;
using DINOForge.Runtime.Bridge;
using DINOForge.Runtime.Diagnostics;

namespace DINOForge.Runtime.Capture
{
    /// <summary>
    /// In-process SESSION RECORDER for DINOForge (#971).
    ///
    /// WHY: synthetic OS input (SendInput / SetCursorPos / MCP <c>game_input</c>) is NOT delivered to
    /// DINO's Unity <see cref="EventSystem"/> — even the native Options button does not hover under
    /// synthetic input. REAL user input DOES reach the EventSystem. So we record a REAL user session
    /// (pointer + key events, plus the widget the EventSystem actually resolved, plus screen frames),
    /// then replay it via an in-process EventSystem driver (#972). This unblocks autonomous
    /// vision-verify for all UI/world tasks.
    ///
    /// EXECUTION MODEL (DINO ECS facts):
    ///  - <c>MonoBehaviour.Update()</c> / <c>OnGUI()</c> NEVER fire (DINO replaces Unity's PlayerLoop).
    ///    We therefore tick the per-frame sampler from a PlayerLoop-injected delegate
    ///    (<see cref="PlayerLoopKeyInputInjection"/>), which runs on the Unity MAIN THREAD every frame
    ///    regardless of scene or ECS group state.
    ///  - <see cref="EventSystem.current"/> reads happen ONLY on that main-thread tick (never a bg thread).
    ///  - All <see cref="EventSystem.current"/> reads are null-guarded (Pattern #235).
    ///  - The F11 toggle is detected on a Win32 <c>GetAsyncKeyState</c> background thread (mirrors
    ///    <see cref="KeyInputSystem"/>); the bg thread only flips a volatile flag — the actual
    ///    start/stop and all Unity API access run on the main-thread tick.
    ///  - Frame capture reuses the proven <c>ScreenCapture.CaptureScreenshot</c> path (GPU backbuffer,
    ///    works on Parsec), invoked on the main-thread tick.
    ///
    /// OUTPUT: a JOURNEY RECORD per session at
    ///   <c>BepInEx/dinoforge_recordings/&lt;session-id&gt;/</c>
    ///     timeline.json   — ordered list of events (see <see cref="SessionEvent"/>)
    ///     manifest.json   — session metadata (id, start/end, screen size, counts)
    ///     frames/*.png    — screen frames tagged by timestamp + scene
    /// Format maps cleanly to phenotype-journeys / JourneyViewer.vue annotations (#966) and to the
    /// in-process replay driver (#972). See docs/capture/session-record-format.md.
    /// </summary>
    public static class SessionRecorder
    {
        // ── F11 toggle via Win32 (background thread; flag-only, no Unity calls) ──────────────
        [DllImport("user32.dll")]
        private static extern ushort GetAsyncKeyState(int vKey);

        private const int VK_F11 = 0x7A;
        private const ushort KEY_PRESSED = 0x8000;

        private static Thread? _toggleThread;
        private static volatile bool _toggleThreadRunning;
        private static bool _f11PreviousState;

        /// <summary>Set true by the bg toggle thread on an F11 edge; consumed on the main-thread tick.</summary>
        private static volatile bool _togglePending;

        // ── Recording state (mutated only on the main-thread tick) ──────────────────────────
        private static bool _recording;
        private static string _sessionDir = "";
        private static string _framesDir = "";
        private static string _sessionId = "";
        private static long _startTicksUtc;
        private static readonly List<SessionEvent> _events = new List<SessionEvent>();
        private static readonly object _eventsLock = new object();

        // Frame cadence + per-frame input edge tracking.
        private static int _frameSeq;
        private static long _lastPeriodicFrameMs = -1;
        private static int _periodicFrameIntervalMs = 500;
        private static bool _prevMouseLeftDown;
        private static bool _prevMouseRightDown;
        private static Vector3 _lastMousePos = new Vector3(float.NaN, float.NaN, 0f);

        // Config-tunable (bound by Plugin.Awake). Defaults are safe if never bound.
        private static bool _captureFramePerEvent = true;
        private static bool _enabled = true;

        /// <summary>True while a recording session is active.</summary>
        public static bool IsRecording => _recording;

        /// <summary>Active session directory (empty when not recording).</summary>
        public static string SessionDir => _sessionDir;

        /// <summary>
        /// Wire config from Plugin.Awake. <paramref name="enabled"/> gates the whole feature;
        /// <paramref name="frameIntervalMs"/> is the periodic frame cadence;
        /// <paramref name="capturePerEvent"/> grabs a frame on every pointer/key event too.
        /// </summary>
        public static void Configure(bool enabled, int frameIntervalMs, bool capturePerEvent)
        {
            _enabled = enabled;
            _periodicFrameIntervalMs = Math.Max(100, frameIntervalMs);
            _captureFramePerEvent = capturePerEvent;
            DebugLog.Write("SessionRecorder",
                $"[SessionRecorder] Configured: enabled={enabled} frameIntervalMs={_periodicFrameIntervalMs} perEvent={capturePerEvent}");
        }

        /// <summary>
        /// Starts the F11 toggle background thread and injects the per-frame sampler into the
        /// PlayerLoop. Idempotent. Call from Plugin.Awake (after KeyInputSystem starts).
        /// </summary>
        public static void Initialize()
        {
            if (!_enabled)
            {
                DebugLog.Write("SessionRecorder", "[SessionRecorder] Disabled by config; not initializing.");
                return;
            }

            StartToggleThread();

            try
            {
                PlayerLoopKeyInputInjection.InjectIntoCurrentPlayerLoop(
                    typeof(SessionRecorderLoopMarker),
                    MainThreadTick);
                DebugLog.Write("SessionRecorder", "[SessionRecorder] PlayerLoop sampler injected. Press F11 to start/stop recording.");
            }
            catch (Exception ex)
            {
                DebugLog.Write("SessionRecorder", $"[SessionRecorder] PlayerLoop inject failed: {ex.Message}");
            }
        }

        /// <summary>Stops the toggle thread and finalizes any in-flight recording. Call from Plugin.OnDestroy.</summary>
        public static void Shutdown()
        {
            _toggleThreadRunning = false;
            _toggleThread = null;
            // If a recording is in flight, flush it (this runs on OnDestroy = frame 0, main thread).
            if (_recording)
            {
                try { StopRecording("plugin-shutdown"); }
                catch (Exception ex) { DebugLog.Write("SessionRecorder", $"[SessionRecorder] Shutdown flush failed: {ex.Message}"); }
            }
        }

        /// <summary>PlayerLoop marker type so the sampler survives DINO's SetPlayerLoop rebuilds.</summary>
        internal struct SessionRecorderLoopMarker { }

        private static void StartToggleThread()
        {
            if (_toggleThreadRunning) return;
            _toggleThreadRunning = true;
            _toggleThread = new Thread(ToggleLoop)
            {
                IsBackground = true,
                Name = "DINOForge-SessionRecorder-F11",
            };
            _toggleThread.Start();
            DebugLog.Write("SessionRecorder", "[SessionRecorder] F11 toggle thread started.");
        }

        private static void ToggleLoop()
        {
            while (_toggleThreadRunning)
            {
                try
                {
                    bool f11Now = (GetAsyncKeyState(VK_F11) & KEY_PRESSED) != 0;
                    if (f11Now && !_f11PreviousState)
                    {
                        _togglePending = true; // consumed on main-thread tick
                        DebugLog.Write("SessionRecorder", "[SessionRecorder] F11 edge detected (bg thread).");
                    }
                    _f11PreviousState = f11Now;
                }
                catch (Exception ex)
                {
                    /* safe-swallow: best-effort polling continues; logged. */
                    try { DebugLog.Write("SessionRecorder", $"[SessionRecorder] toggle iter exception: {ex.Message}"); }
                    catch { /* logging unavailable */ }
                }
                try { Thread.Sleep(40); }
                catch (ThreadInterruptedException) { break; }
            }
        }

        /// <summary>
        /// Per-frame sampler. Runs on the Unity MAIN THREAD via PlayerLoop injection. Consumes the
        /// pending F11 toggle, samples pointer/key state + the EventSystem-resolved widget, and
        /// triggers frame capture. All <see cref="EventSystem.current"/> reads are here (main thread).
        /// </summary>
        private static void MainThreadTick()
        {
            try
            {
                if (_togglePending)
                {
                    _togglePending = false;
                    if (_recording) StopRecording("f11");
                    else StartRecording("f11");
                }

                if (!_recording) return;

                long t = NowMs();

                // ── Pointer move (debounced: only when position changes meaningfully) ──
                Vector3 mouse = Input.mousePosition;
                bool moved = float.IsNaN(_lastMousePos.x)
                             || Mathf.Abs(mouse.x - _lastMousePos.x) >= 1f
                             || Mathf.Abs(mouse.y - _lastMousePos.y) >= 1f;
                if (moved)
                {
                    _lastMousePos = mouse;
                    RecordEvent(new SessionEvent
                    {
                        T = t,
                        Type = "pointer.move",
                        X = mouse.x,
                        Y = mouse.y,
                        Target = ResolveHoverTarget(mouse),
                        Scene = ActiveSceneName(),
                    }, captureFrame: false);
                }

                // ── Mouse buttons ──
                bool leftDown = Input.GetMouseButton(0);
                if (leftDown && !_prevMouseLeftDown)
                    RecordPointerButton(t, mouse, "pointer.down", 0);
                else if (!leftDown && _prevMouseLeftDown)
                    RecordPointerButton(t, mouse, "pointer.up", 0);
                _prevMouseLeftDown = leftDown;

                bool rightDown = Input.GetMouseButton(1);
                if (rightDown && !_prevMouseRightDown)
                    RecordPointerButton(t, mouse, "pointer.down", 1);
                else if (!rightDown && _prevMouseRightDown)
                    RecordPointerButton(t, mouse, "pointer.up", 1);
                _prevMouseRightDown = rightDown;

                // ── Keys (sample the common interaction keys; full keyboard via Input.anyKeyDown) ──
                SampleKeys(t);

                // ── Periodic frame ──
                if (_lastPeriodicFrameMs < 0 || (t - _lastPeriodicFrameMs) >= _periodicFrameIntervalMs)
                {
                    _lastPeriodicFrameMs = t;
                    CaptureFrame(t, "periodic");
                }
            }
            catch (Exception ex)
            {
                DebugLog.Write("SessionRecorder", $"[SessionRecorder] MainThreadTick exception: {ex.Message}");
            }
        }

        private static void RecordPointerButton(long t, Vector3 mouse, string type, int button)
        {
            // For pointer.down resolve the press target; for clicks the EventSystem's selected object.
            string target = ResolvePressTarget(mouse);
            string? selected = SelectedGameObjectPath();
            RecordEvent(new SessionEvent
            {
                T = t,
                Type = type,
                X = mouse.x,
                Y = mouse.y,
                Button = button,
                Target = target,
                Selected = selected,
                Scene = ActiveSceneName(),
            }, captureFrame: _captureFramePerEvent);
        }

        private static readonly KeyCode[] _trackedKeys =
        {
            KeyCode.Return, KeyCode.KeypadEnter, KeyCode.Escape, KeyCode.Space, KeyCode.Tab,
            KeyCode.Backspace, KeyCode.Delete,
            KeyCode.UpArrow, KeyCode.DownArrow, KeyCode.LeftArrow, KeyCode.RightArrow,
            KeyCode.LeftShift, KeyCode.RightShift, KeyCode.LeftControl, KeyCode.RightControl,
        };

        private static void SampleKeys(long t)
        {
            for (int i = 0; i < _trackedKeys.Length; i++)
            {
                KeyCode k = _trackedKeys[i];
                if (Input.GetKeyDown(k))
                    RecordKey(t, "key.down", k);
                else if (Input.GetKeyUp(k))
                    RecordKey(t, "key.up", k);
            }
            // Capture typed characters (letters/digits) on key-down for fields.
            if (Input.anyKeyDown && !string.IsNullOrEmpty(Input.inputString))
            {
                foreach (char c in Input.inputString)
                {
                    if (c == '\b' || c == '\n' || c == '\r') continue; // covered by tracked keys
                    RecordEvent(new SessionEvent
                    {
                        T = t,
                        Type = "key.char",
                        Key = c.ToString(),
                        Scene = ActiveSceneName(),
                    }, captureFrame: false);
                }
            }
        }

        private static void RecordKey(long t, string type, KeyCode k)
        {
            RecordEvent(new SessionEvent
            {
                T = t,
                Type = type,
                Key = k.ToString(),
                Scene = ActiveSceneName(),
            }, captureFrame: false);
        }

        // ── EventSystem widget resolution (Pattern #235: null-guard EventSystem.current) ─────

        /// <summary>Path of the GameObject the EventSystem currently has selected, or null.</summary>
        private static string? SelectedGameObjectPath()
        {
            EventSystem es = EventSystem.current;
            if (es == null) return null;
            GameObject sel = es.currentSelectedGameObject;
            return sel != null ? GameObjectPath(sel) : null;
        }

        /// <summary>
        /// Resolves the UI widget under the pointer by running an EventSystem raycast at
        /// <paramref name="screenPos"/>. Returns the topmost hit's GameObject path, or "".
        /// </summary>
        private static string ResolveHoverTarget(Vector3 screenPos) => RaycastTopPath(screenPos);

        private static string ResolvePressTarget(Vector3 screenPos) => RaycastTopPath(screenPos);

        private static readonly List<RaycastResult> _raycastScratch = new List<RaycastResult>(16);

        private static string RaycastTopPath(Vector3 screenPos)
        {
            EventSystem es = EventSystem.current;
            if (es == null) return "";
            try
            {
                var ped = new PointerEventData(es)
                {
                    position = new Vector2(screenPos.x, screenPos.y),
                };
                _raycastScratch.Clear();
                es.RaycastAll(ped, _raycastScratch);
                if (_raycastScratch.Count == 0) return "";
                GameObject? go = _raycastScratch[0].gameObject;
                return go != null ? GameObjectPath(go) : "";
            }
            catch (Exception ex)
            {
                DebugLog.Write("SessionRecorder", $"[SessionRecorder] RaycastTopPath failed: {ex.Message}");
                return "";
            }
        }

        /// <summary>Builds a stable "/"-delimited transform path for a GameObject (root → leaf).</summary>
        private static string GameObjectPath(GameObject go)
        {
            var sb = new StringBuilder(128);
            Transform? t = go.transform;
            var stack = new List<string>(8);
            while (t != null)
            {
                stack.Add(t.name);
                t = t.parent;
            }
            for (int i = stack.Count - 1; i >= 0; i--)
            {
                sb.Append('/').Append(stack[i]);
            }
            return sb.ToString();
        }

        private static string ActiveSceneName()
        {
            try { return SceneManager.GetActiveScene().name ?? ""; }
            catch { return ""; }
        }

        // ── Session lifecycle ────────────────────────────────────────────────────────────

        private static void StartRecording(string reason)
        {
            _sessionId = "session-" + DateTime.UtcNow.ToString("yyyyMMddTHHmmssZ", CultureInfo.InvariantCulture);
            string root = Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge_recordings", _sessionId);
            _sessionDir = root;
            _framesDir = Path.Combine(root, "frames");
            Directory.CreateDirectory(_framesDir);

            lock (_eventsLock) { _events.Clear(); }
            _frameSeq = 0;
            _lastPeriodicFrameMs = -1;
            _prevMouseLeftDown = false;
            _prevMouseRightDown = false;
            _lastMousePos = new Vector3(float.NaN, float.NaN, 0f);
            _startTicksUtc = DateTime.UtcNow.Ticks;
            _recording = true;

            DebugLog.Write("SessionRecorder",
                $"[SessionRecorder] ▶ RECORDING STARTED ({reason}) id={_sessionId} dir={_sessionDir}");
            // Initial frame so the timeline has a baseline.
            CaptureFrame(0, "start");
        }

        private static void StopRecording(string reason)
        {
            _recording = false;
            try
            {
                WriteTimeline();
                WriteManifest(reason);
                int count;
                lock (_eventsLock) { count = _events.Count; }
                DebugLog.Write("SessionRecorder",
                    $"[SessionRecorder] ■ RECORDING STOPPED ({reason}) id={_sessionId} events={count} frames={_frameSeq} dir={_sessionDir}");
            }
            catch (Exception ex)
            {
                DebugLog.Write("SessionRecorder", $"[SessionRecorder] StopRecording flush failed: {ex.Message}");
            }
        }

        private static void RecordEvent(SessionEvent e, bool captureFrame)
        {
            if (captureFrame)
            {
                string frame = CaptureFrame(e.T, e.Type);
                e.Frame = frame;
            }
            lock (_eventsLock) { _events.Add(e); }
        }

        /// <summary>
        /// Captures a screen frame to frames/ using the proven ScreenCapture path (GPU backbuffer).
        /// Returns the relative frame filename (frames/NNNNNN.png). Async on Unity's side — the file
        /// appears a frame or two later, which is fine for the timeline reference.
        /// </summary>
        private static string CaptureFrame(long t, string tag)
        {
            int seq = _frameSeq++;
            string fileName = seq.ToString("D6", CultureInfo.InvariantCulture) + ".png";
            string rel = "frames/" + fileName;
            string abs = Path.Combine(_framesDir, fileName);
            try
            {
                ScreenCapture.CaptureScreenshot(abs);
            }
            catch (Exception ex)
            {
                DebugLog.Write("SessionRecorder", $"[SessionRecorder] CaptureFrame failed ({tag}): {ex.Message}");
            }
            return rel;
        }

        // ── Serialization (manual JSON — netstandard2.0, no Newtonsoft dependency required) ──

        private static void WriteTimeline()
        {
            var sb = new StringBuilder(8192);
            sb.Append("{\n");
            sb.Append("  \"version\": 1,\n");
            sb.Append("  \"sessionId\": ").Append(JsonStr(_sessionId)).Append(",\n");
            sb.Append("  \"events\": [\n");
            List<SessionEvent> snapshot;
            lock (_eventsLock) { snapshot = new List<SessionEvent>(_events); }
            for (int i = 0; i < snapshot.Count; i++)
            {
                sb.Append("    ").Append(snapshot[i].ToJson());
                if (i < snapshot.Count - 1) sb.Append(',');
                sb.Append('\n');
            }
            sb.Append("  ]\n");
            sb.Append("}\n");
            File.WriteAllText(Path.Combine(_sessionDir, "timeline.json"), sb.ToString(), new UTF8Encoding(false));
        }

        private static void WriteManifest(string stopReason)
        {
            int width = 0, height = 0;
            try { width = Screen.width; height = Screen.height; } catch { /* Screen unavailable */ }
            int count;
            lock (_eventsLock) { count = _events.Count; }

            var sb = new StringBuilder(1024);
            sb.Append("{\n");
            sb.Append("  \"version\": 1,\n");
            sb.Append("  \"sessionId\": ").Append(JsonStr(_sessionId)).Append(",\n");
            sb.Append("  \"startedUtc\": ").Append(JsonStr(new DateTime(_startTicksUtc, DateTimeKind.Utc).ToString("o", CultureInfo.InvariantCulture))).Append(",\n");
            sb.Append("  \"endedUtc\": ").Append(JsonStr(DateTime.UtcNow.ToString("o", CultureInfo.InvariantCulture))).Append(",\n");
            sb.Append("  \"stopReason\": ").Append(JsonStr(stopReason)).Append(",\n");
            sb.Append("  \"screenWidth\": ").Append(width).Append(",\n");
            sb.Append("  \"screenHeight\": ").Append(height).Append(",\n");
            sb.Append("  \"eventCount\": ").Append(count).Append(",\n");
            sb.Append("  \"frameCount\": ").Append(_frameSeq).Append(",\n");
            sb.Append("  \"framesDir\": \"frames\",\n");
            sb.Append("  \"timeline\": \"timeline.json\"\n");
            sb.Append("}\n");
            File.WriteAllText(Path.Combine(_sessionDir, "manifest.json"), sb.ToString(), new UTF8Encoding(false));
        }

        internal static string JsonStr(string? s)
        {
            if (s == null) return "null";
            var sb = new StringBuilder(s.Length + 2);
            sb.Append('"');
            foreach (char c in s)
            {
                switch (c)
                {
                    case '"': sb.Append("\\\""); break;
                    case '\\': sb.Append("\\\\"); break;
                    case '\n': sb.Append("\\n"); break;
                    case '\r': sb.Append("\\r"); break;
                    case '\t': sb.Append("\\t"); break;
                    default:
                        if (c < 0x20) sb.Append("\\u").Append(((int)c).ToString("x4", CultureInfo.InvariantCulture));
                        else sb.Append(c);
                        break;
                }
            }
            sb.Append('"');
            return sb.ToString();
        }

        private static long NowMs()
        {
            long elapsedTicks = DateTime.UtcNow.Ticks - _startTicksUtc;
            return elapsedTicks / TimeSpan.TicksPerMillisecond;
        }
    }

    /// <summary>
    /// One timeline event in a journey record. Times are milliseconds since session start.
    /// Fields are sparse — only set fields are emitted. Maps to the replay driver (#972):
    ///   pointer.move/down/up → in-process PointerEventData synthesis at (X,Y) targeting <see cref="Target"/>;
    ///   key.down/up/char     → EventSystem key/text injection;
    /// and to JourneyViewer.vue annotations (#966) via T + Frame + Target.
    /// </summary>
    public sealed class SessionEvent
    {
        /// <summary>Milliseconds since session start.</summary>
        public long T;

        /// <summary>Event type: pointer.move | pointer.down | pointer.up | key.down | key.up | key.char.</summary>
        public string Type = "";

        /// <summary>Pointer X (screen px, bottom-left origin) — pointer events only.</summary>
        public float X;

        /// <summary>Pointer Y (screen px, bottom-left origin) — pointer events only.</summary>
        public float Y;

        /// <summary>Mouse button: 0 = left, 1 = right — pointer.down/up only.</summary>
        public int Button = -1;

        /// <summary>EventSystem raycast-resolved widget path under the pointer ("/Canvas/.../Button").</summary>
        public string? Target;

        /// <summary>EventSystem.current.currentSelectedGameObject path at the time of the event.</summary>
        public string? Selected;

        /// <summary>Key name (KeyCode) or typed character — key events only.</summary>
        public string? Key;

        /// <summary>Active scene name when the event occurred.</summary>
        public string? Scene;

        /// <summary>Relative frame path (frames/NNNNNN.png) captured with this event, if any.</summary>
        public string? Frame;

        internal string ToJson()
        {
            var sb = new StringBuilder(160);
            sb.Append("{");
            sb.Append("\"t\":").Append(T.ToString(CultureInfo.InvariantCulture));
            sb.Append(",\"type\":").Append(SessionRecorder.JsonStr(Type));
            if (Type.StartsWith("pointer", StringComparison.Ordinal))
            {
                sb.Append(",\"x\":").Append(X.ToString("0.##", CultureInfo.InvariantCulture));
                sb.Append(",\"y\":").Append(Y.ToString("0.##", CultureInfo.InvariantCulture));
                if (Button >= 0) sb.Append(",\"button\":").Append(Button.ToString(CultureInfo.InvariantCulture));
            }
            if (Target != null) sb.Append(",\"target\":").Append(SessionRecorder.JsonStr(Target));
            if (Selected != null) sb.Append(",\"selected\":").Append(SessionRecorder.JsonStr(Selected));
            if (Key != null) sb.Append(",\"key\":").Append(SessionRecorder.JsonStr(Key));
            if (Scene != null) sb.Append(",\"scene\":").Append(SessionRecorder.JsonStr(Scene));
            if (Frame != null) sb.Append(",\"frame\":").Append(SessionRecorder.JsonStr(Frame));
            sb.Append("}");
            return sb.ToString();
        }
    }
}
