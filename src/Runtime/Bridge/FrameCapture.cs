#nullable enable
using System;
using System.Collections;
using System.IO;
using System.Threading;
using DINOForge.Runtime.Diagnostics;
using UnityEngine;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Robust in-process frame capture that works in ALL game states (main menu, loading,
    /// active gameplay).
    ///
    /// <para>
    /// WHY THIS EXISTS (issue #972): the legacy path used
    /// <see cref="UnityEngine.ScreenCapture.CaptureScreenshot(string)"/>, which is
    /// <b>asynchronous</b> — it queues a capture Unity flushes at the end of a later frame and
    /// writes the PNG itself. The bridge handler returned <c>Success=true</c> the instant the
    /// request was queued, without ever confirming a file landed on disk. In DINO's custom
    /// PlayerLoop that deferred flush was unreliable during active gameplay, so the handler
    /// reported "saved" while no PNG was written.
    /// </para>
    ///
    /// <para>
    /// THE FIX: <see cref="ScreenCapture.CaptureScreenshotIntoRenderTexture(RenderTexture)"/>
    /// composites the FINAL frame (all cameras + Screen-Space-Overlay UI) into a RenderTexture.
    /// We invoke it inside a coroutine that yields <see cref="WaitForEndOfFrame"/>, then
    /// <see cref="Texture2D.ReadPixels"/> the RT back, <c>EncodeToPNG()</c>, and
    /// <see cref="File.WriteAllBytes"/>. The coroutine signals a <see cref="ManualResetEventSlim"/>
    /// the calling bridge thread blocks on, so the file is fully written before the RPC returns.
    /// This captures exactly what is on screen in every state (it was the overlay-UI / final-
    /// composite gap that made the naive camera-RT readback come back black at the menu).
    /// </para>
    /// </summary>
    public static class FrameCapture
    {
        /// <summary>Result of a capture attempt.</summary>
        public readonly struct Result
        {
            public Result(bool success, string path, int width, int height, long bytes, string method, string? error)
            {
                Success = success;
                Path = path;
                Width = width;
                Height = height;
                Bytes = bytes;
                Method = method;
                Error = error;
            }

            public bool Success { get; }
            public string Path { get; }
            public int Width { get; }
            public int Height { get; }
            public long Bytes { get; }

            /// <summary>"endframe" (CaptureScreenshotIntoRenderTexture) or "camera" (fallback).</summary>
            public string Method { get; }
            public string? Error { get; }
        }

        /// <summary>
        /// MonoBehaviour that hosts the capture coroutine. Attached to a DontDestroyOnLoad host
        /// so it survives scene transitions. Coroutines (and thus <see cref="WaitForEndOfFrame"/>)
        /// DO run in DINO even though MonoBehaviour.Update() does not.
        /// </summary>
        private sealed class CaptureRunner : MonoBehaviour { }

        private static CaptureRunner? _runner;
        private static readonly object _runnerLock = new object();

        private static CaptureRunner GetRunner()
        {
            lock (_runnerLock)
            {
                if (_runner == null)
                {
                    GameObject host = Plugin.PersistentRoot != null
                        ? Plugin.PersistentRoot
                        : new GameObject("DINOForge_CaptureRunner");
                    if (Plugin.PersistentRoot == null)
                        UnityEngine.Object.DontDestroyOnLoad(host);
                    _runner = host.GetComponent<CaptureRunner>() ?? host.AddComponent<CaptureRunner>();
                }
                return _runner;
            }
        }

        /// <summary>
        /// Capture the current frame to <paramref name="path"/> as a PNG. Blocks the calling
        /// thread (up to <paramref name="timeoutMs"/>) until the coroutine has written the file
        /// at end-of-frame. Safe to call from a background thread; the GPU work happens on the
        /// main thread inside the coroutine.
        /// </summary>
        public static Result Capture(string path, int timeoutMs = 4000)
        {
            if (string.IsNullOrEmpty(path))
                return new Result(false, path, 0, 0, 0, "none", "path was null/empty");

            try
            {
                string? dir = System.IO.Path.GetDirectoryName(path);
                if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir))
                    Directory.CreateDirectory(dir);
            }
            catch (Exception ex)
            {
                return new Result(false, path, 0, 0, 0, "none", $"mkdir failed: {ex.Message}");
            }

            using ManualResetEventSlim done = new ManualResetEventSlim(false);
            Result captured = new Result(false, path, 0, 0, 0, "none", "coroutine did not run");

            void Run()
            {
                try
                {
                    GetRunner().StartCoroutine(CaptureCoroutine(path, r =>
                    {
                        captured = r;
                        done.Set();
                    }));
                }
                catch (Exception ex)
                {
                    captured = new Result(false, path, 0, 0, 0, "none", $"start failed: {ex.Message}");
                    done.Set();
                }
            }

            // If we are already on the main thread (e.g. NativeMenuInjector.Update), run directly;
            // otherwise marshal through the dispatcher and block until the coroutine signals.
            if (MainThreadDispatcher.IsMainThread)
            {
                Run();
            }
            else
            {
                System.Threading.Tasks.Task t = MainThreadDispatcher.RunOnMainThread((Action)Run);
                // The dispatched action only STARTS the coroutine; completion is signalled below.
                t.Wait(timeoutMs); // sync-over-async-unavoidable: coroutine completion gated on `done`
            }

            if (!done.Wait(timeoutMs))
            {
                DebugLog.Write("FrameCapture", $"[FrameCapture] timed out after {timeoutMs}ms: {path}");
                return new Result(false, path, 0, 0, 0, "none", "timeout waiting for end-of-frame capture");
            }

            return captured;
        }

        private static IEnumerator CaptureCoroutine(string path, Action<Result> onDone)
        {
            // Wait until the frame is fully rendered & presented so the composite is complete.
            yield return new WaitForEndOfFrame();

            Result result;
            int width = Mathf.Max(1, Screen.width);
            int height = Mathf.Max(1, Screen.height);
            RenderTexture rt = RenderTexture.GetTemporary(width, height, 24, RenderTextureFormat.ARGB32);
            RenderTexture? prevActive = RenderTexture.active;
            Texture2D? tex = null;
            try
            {
                // Composites ALL cameras + overlay UI into rt — exactly what is on screen.
                ScreenCapture.CaptureScreenshotIntoRenderTexture(rt);

                RenderTexture.active = rt;
                tex = new Texture2D(width, height, TextureFormat.RGB24, false);
                tex.ReadPixels(new Rect(0, 0, width, height), 0, 0);
                tex.Apply(false);

                // CaptureScreenshotIntoRenderTexture writes with origin bottom-left on some
                // graphics APIs; the PNG comes out flipped. Flip vertically to match screen.
                FlipVertical(tex);

                byte[] png = tex.EncodeToPNG();
                File.WriteAllBytes(path, png);
                long size = new FileInfo(path).Length;
                DebugLog.Write("FrameCapture", $"[FrameCapture] endframe OK: {width}x{height} {size}B -> {path}");
                result = new Result(true, path, width, height, size, "endframe", null);
            }
            catch (Exception ex)
            {
                DebugLog.Write("FrameCapture", $"[FrameCapture] endframe capture failed for '{path}' ({ex.GetType().Name}): {ex}");
                result = new Result(false, path, 0, 0, 0, "endframe", ex.Message);
            }
            finally
            {
                RenderTexture.active = prevActive;
                RenderTexture.ReleaseTemporary(rt);
                if (tex != null)
                    UnityEngine.Object.Destroy(tex);
            }

            onDone(result);
        }

        private static void FlipVertical(Texture2D tex)
        {
            int w = tex.width;
            int h = tex.height;
            Color[] pixels = tex.GetPixels();
            Color[] flipped = new Color[pixels.Length];
            for (int y = 0; y < h; y++)
            {
                int srcRow = (h - 1 - y) * w;
                int dstRow = y * w;
                Array.Copy(pixels, srcRow, flipped, dstRow, w);
            }
            tex.SetPixels(flipped);
            tex.Apply(false);
        }
    }
}