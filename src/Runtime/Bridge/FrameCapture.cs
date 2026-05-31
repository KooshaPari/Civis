#nullable enable
using System;
using System.IO;
using DINOForge.Runtime.Diagnostics;
using UnityEngine;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Robust, synchronous in-process frame capture that works in ALL game states
    /// (main menu, loading, active gameplay).
    ///
    /// <para>
    /// WHY THIS EXISTS (issue #972): the legacy path used
    /// <see cref="UnityEngine.ScreenCapture.CaptureScreenshot(string)"/>, which is
    /// <b>asynchronous</b> — it merely queues a capture that Unity flushes at the end of a
    /// later frame via an internal PlayerLoop callback, then writes the PNG itself. The
    /// bridge handler returned <c>Success=true</c> the instant the request was queued,
    /// without ever confirming a file landed on disk. In DINO's custom PlayerLoop the
    /// deferred end-of-frame screenshot flush is unreliable during active gameplay, so the
    /// handler reported "saved" while no PNG was written. (It happened to work at the main
    /// menu where the stock present path still ran.)
    /// </para>
    ///
    /// <para>
    /// THE FIX: render the active camera into a temporary <see cref="RenderTexture"/>,
    /// <see cref="Texture2D.ReadPixels"/> it back into a CPU texture, <c>EncodeToPNG()</c>,
    /// and <see cref="File.WriteAllBytes"/> — all synchronously on the main thread inside a
    /// single <see cref="MainThreadDispatcher"/> dispatch. No dependency on Unity's deferred
    /// screenshot path; the file exists before the call returns. Falls back to a backbuffer
    /// <c>ReadPixels</c> (no camera, e.g. pure-UI menu) which captures whatever was last
    /// presented.
    /// </para>
    ///
    /// MUST be invoked on the Unity main thread (it touches Camera/RenderTexture/GL state).
    /// </summary>
    public static class FrameCapture
    {
        /// <summary>
        /// Result of a capture attempt. <see cref="Bytes"/> is the on-disk file size and is
        /// only meaningful when <see cref="Success"/> is true.
        /// </summary>
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

            /// <summary>"camera" (RT readback) or "backbuffer" (no active camera).</summary>
            public string Method { get; }
            public string? Error { get; }
        }

        /// <summary>
        /// Capture the current frame to <paramref name="path"/> as a PNG. Synchronous: the
        /// file is fully written (or the failure is known) before this returns.
        /// Call ONLY on the Unity main thread.
        /// </summary>
        public static Result Capture(string path)
        {
            if (string.IsNullOrEmpty(path))
                return new Result(false, path, 0, 0, 0, "none", "path was null/empty");

            try
            {
                string? dir = System.IO.Path.GetDirectoryName(path);
                if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir))
                    Directory.CreateDirectory(dir);

                Camera? cam = SelectCamera();
                return cam != null ? CaptureViaCamera(cam, path) : CaptureViaBackbuffer(path);
            }
            catch (Exception ex)
            {
                DebugLog.Write("FrameCapture", $"[FrameCapture] Capture failed for '{path}' ({ex.GetType().Name}): {ex}");
                return new Result(false, path, 0, 0, 0, "none", ex.Message);
            }
        }

        /// <summary>
        /// Picks the best camera to capture: the highest-depth enabled camera among
        /// <see cref="Camera.allCameras"/>. DINO uses multiple cameras (world + UI overlays);
        /// the highest-depth one renders last/on top. Returns null when no enabled camera
        /// exists (pure-UI / loading screens).
        /// </summary>
        private static Camera? SelectCamera()
        {
            Camera? best = null;
            float bestDepth = float.NegativeInfinity;

            Camera[] cams = Camera.allCameras;
            for (int i = 0; i < cams.Length; i++)
            {
                Camera c = cams[i];
                if (c == null || !c.isActiveAndEnabled)
                    continue;
                if (c.depth > bestDepth)
                {
                    bestDepth = c.depth;
                    best = c;
                }
            }

            return best;
        }

        /// <summary>
        /// Renders <paramref name="cam"/> into a temporary RenderTexture, reads it back, and
        /// writes a PNG. This bypasses Unity's deferred screenshot flush entirely.
        /// </summary>
        private static Result CaptureViaCamera(Camera cam, string path)
        {
            int width = Mathf.Max(1, Screen.width > 0 ? Screen.width : (cam.pixelWidth > 0 ? cam.pixelWidth : 1920));
            int height = Mathf.Max(1, Screen.height > 0 ? Screen.height : (cam.pixelHeight > 0 ? cam.pixelHeight : 1080));

            RenderTexture rt = RenderTexture.GetTemporary(width, height, 24, RenderTextureFormat.ARGB32);
            RenderTexture? prevCamTarget = cam.targetTexture;
            RenderTexture? prevActive = RenderTexture.active;
            Texture2D? tex = null;
            try
            {
                cam.targetTexture = rt;
                cam.Render();

                RenderTexture.active = rt;
                tex = new Texture2D(width, height, TextureFormat.RGB24, false);
                tex.ReadPixels(new Rect(0, 0, width, height), 0, 0);
                tex.Apply(false);

                byte[] png = tex.EncodeToPNG();
                File.WriteAllBytes(path, png);

                long size = new FileInfo(path).Length;
                DebugLog.Write("FrameCapture", $"[FrameCapture] camera readback OK: {width}x{height} {size}B cam='{cam.name}' -> {path}");
                return new Result(true, path, width, height, size, "camera", null);
            }
            finally
            {
                cam.targetTexture = prevCamTarget;
                RenderTexture.active = prevActive;
                RenderTexture.ReleaseTemporary(rt);
                if (tex != null)
                    UnityEngine.Object.Destroy(tex);
            }
        }

        /// <summary>
        /// Fallback when no active camera exists: read the current backbuffer directly. This
        /// captures whatever was last presented (UI menus draw to the backbuffer even when no
        /// scene camera is enabled). Less robust than the camera path but better than nothing.
        /// </summary>
        private static Result CaptureViaBackbuffer(string path)
        {
            int width = Mathf.Max(1, Screen.width);
            int height = Mathf.Max(1, Screen.height);

            Texture2D? tex = null;
            RenderTexture? prevActive = RenderTexture.active;
            try
            {
                // Reading from the active (null => backbuffer) target.
                RenderTexture.active = null;
                tex = new Texture2D(width, height, TextureFormat.RGB24, false);
                tex.ReadPixels(new Rect(0, 0, width, height), 0, 0);
                tex.Apply(false);

                byte[] png = tex.EncodeToPNG();
                File.WriteAllBytes(path, png);

                long size = new FileInfo(path).Length;
                DebugLog.Write("FrameCapture", $"[FrameCapture] backbuffer readback OK: {width}x{height} {size}B -> {path}");
                return new Result(true, path, width, height, size, "backbuffer", null);
            }
            finally
            {
                RenderTexture.active = prevActive;
                if (tex != null)
                    UnityEngine.Object.Destroy(tex);
            }
        }
    }
}
