"""Docker isolation backend — containerized DINO game launches.

Status: SCAFFOLD (v0.26.0 wave 1). Real launch + capture + inject pipelines
are stubbed beyond the v0.23.0 skeleton but are not yet end-to-end functional.
Marked NotImplementedError where the inner integration is still TBD.

Design intent
=============
The DockerBackend is the Tier-4 fallback in the isolation hierarchy:

    Tier 1: VDD virtual display (future)
    Tier 2: HiddenDesktopBackend (Win32 CreateDesktop)
    Tier 3: PlayCUABackend (cross-platform bare-cua-native)
    Tier 4: DockerBackend (THIS — for headless CI / Kubernetes / pheno-compose)

Container image requirements
============================
The image must:
- Be a Windows container (process or hyperv isolation) with the DINO Steam
  install bind-mounted (read-only) at C:\\\\game.
- Run BepInEx with DINOForge.Runtime.dll deployed under
  C:\\\\game\\\\BepInEx\\\\plugins.
- Expose an input-injection RPC channel (currently TBD; planned: a small
  REST/gRPC server inside the container that talks to a SendInput shim).
- Stream the framebuffer over stdout (ffmpeg ``-f rawvideo -`` pipe) so the
  host can transcode to PNG/MP4 without needing X11/Wayland inside the
  container.

The image is referenced by its tag; default is ``dinoforge/game-headless:latest``.
If the image is not present locally, ``launch_window`` emits a clear "build
image first" error rather than attempting a registry pull (which would require
authentication and isn't safe to do silently in CI).
"""

from __future__ import annotations

import logging
import shutil
import subprocess
from dataclasses import dataclass
from typing import List, Optional

from .isolation_layer import Frame, IsolationBackend, WindowInfo

logger = logging.getLogger(__name__)


DEFAULT_IMAGE_TAG = "dinoforge/game-headless:latest"
STEAM_GAME_DIR = r"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
CONTAINER_GAME_MOUNT = r"C:\game"


@dataclass
class DockerLaunchHandle:
    """Tracks state for a launched DINO container."""
    container_id: str
    image_tag: str
    exe_inside: str = r"C:\game\Diplomacy is Not an Option.exe"

    def docker_id_short(self) -> str:
        return self.container_id[:12] if self.container_id else "<unknown>"


# ---------------------------------------------------------------------------
# DockerBackend
# ---------------------------------------------------------------------------

class DockerBackend(IsolationBackend):
    """Containerized backend for headless game launches.

    Implements:
      - is_available() — real (probes ``docker --version`` and image presence)
      - launch_container() — real subprocess (``docker run -d ...``) returning
        container ID; raises with explicit "build image first" if image missing
      - terminate_container() — real (``docker rm -f``)
      - capture_frame() — SCAFFOLD: ffmpeg cmdline assembled but stream
        consumption is TBD; raises NotImplementedError for actual frames
      - inject_input() — SCAFFOLD: ``docker exec`` cmdline assembled but
        injection tool inside container is TBD; raises NotImplementedError

    Async surface (IsolationBackend conformance):
      - capture_window / capture_display delegate to capture_frame/capture_screen
      - inject_key / type_text / mouse_click / mouse_scroll wrap inject_input
      - list_windows reports a synthetic single-window per running container
      - launch_process wraps launch_container
    """

    def __init__(
        self,
        image_tag: str = DEFAULT_IMAGE_TAG,
        steam_mount: str = STEAM_GAME_DIR,
        container_mount: str = CONTAINER_GAME_MOUNT,
    ):
        self.image_tag = image_tag
        self.steam_mount = steam_mount
        self.container_mount = container_mount
        self._handle: Optional[DockerLaunchHandle] = None

    # ------------------------------------------------------------------
    # Availability probe
    # ------------------------------------------------------------------

    def is_available(self) -> bool:
        """Return True if docker CLI is on PATH AND the image is present locally."""
        if shutil.which("docker") is None:
            logger.info("docker CLI not on PATH; DockerBackend unavailable")
            return False
        try:
            result = subprocess.run(
                ["docker", "image", "inspect", self.image_tag],
                capture_output=True, text=True, timeout=10,
            )
            if result.returncode != 0:
                logger.info(f"image '{self.image_tag}' not present locally; build first")
                return False
            return True
        except Exception as e:
            logger.warning(f"DockerBackend.is_available probe failed: {e}")
            return False

    # ------------------------------------------------------------------
    # Launch / Terminate
    # ------------------------------------------------------------------

    def launch_container(self, exe_path: Optional[str] = None, args: Optional[List[str]] = None) -> DockerLaunchHandle:
        """Launch DINO inside a Windows container; return DockerLaunchHandle.

        If the image is missing locally, raises RuntimeError with a clear
        ``build image first`` message. We deliberately do NOT attempt a
        ``docker pull`` because the image is internal (not on Docker Hub).
        """
        if shutil.which("docker") is None:
            raise RuntimeError("docker CLI not found on PATH — install Docker Desktop first")

        # Confirm image is present
        inspect = subprocess.run(
            ["docker", "image", "inspect", self.image_tag],
            capture_output=True, text=True, timeout=10,
        )
        if inspect.returncode != 0:
            raise RuntimeError(
                f"Image '{self.image_tag}' not present locally. "
                f"Build it first: `docker build -t {self.image_tag} "
                f"-f scripts/docker/Dockerfile.game-headless .`"
            )

        # Compose the run command
        # NOTE: --isolation=process requires Windows Server / Pro host.
        cmd = [
            "docker", "run", "-d",
            "--isolation=process",
            "--mount", f"type=bind,source={self.steam_mount},target={self.container_mount},readonly",
            "--name", "dinoforge-headless-game",
            self.image_tag,
        ]
        if args:
            cmd.extend(args)

        logger.info(f"docker launch: {' '.join(cmd)}")
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        except subprocess.TimeoutExpired:
            raise RuntimeError("docker run timed out after 30s")

        if result.returncode != 0:
            raise RuntimeError(f"docker run failed: {result.stderr.strip()}")

        container_id = result.stdout.strip()
        if not container_id:
            raise RuntimeError(f"docker run returned no container ID; stderr: {result.stderr!r}")

        handle = DockerLaunchHandle(container_id=container_id, image_tag=self.image_tag)
        self._handle = handle
        logger.info(f"Launched container {handle.docker_id_short()} (image {self.image_tag})")
        return handle

    def terminate_container(self, handle: Optional[DockerLaunchHandle] = None) -> bool:
        """Force-remove the container. Returns True on success."""
        h = handle or self._handle
        if h is None:
            logger.warning("terminate_window: no handle to terminate")
            return False
        try:
            result = subprocess.run(
                ["docker", "rm", "-f", h.container_id],
                capture_output=True, text=True, timeout=15,
            )
            if result.returncode != 0:
                logger.error(f"docker rm -f failed: {result.stderr.strip()}")
                return False
            if self._handle and self._handle.container_id == h.container_id:
                self._handle = None
            return True
        except Exception as e:
            logger.error(f"terminate_window error: {e}")
            return False

    # ------------------------------------------------------------------
    # Capture (SCAFFOLD — ffmpeg streaming TBD)
    # ------------------------------------------------------------------

    def capture_frame(self, handle: Optional[DockerLaunchHandle] = None) -> Frame:
        """Capture a single frame from the container's framebuffer.

        SCAFFOLD: builds the ffmpeg cmdline that *would* extract a single PNG
        frame from the container's screen-recording output, but the actual
        stream consumption + decoding is TBD. The container image must expose
        a screen-recording service (e.g. ffmpeg listening on stdin/stdout) for
        this to function end-to-end.

        Raises:
            NotImplementedError: until the container-side screen-recording
            service is integrated. The cmdline is logged for reference.
        """
        h = handle or self._handle
        if h is None:
            raise RuntimeError("capture_window: no container handle available")

        # Scaffolded ffmpeg command — emits the cmdline that would be used.
        # Final form once implemented:
        #   docker exec <id> ffmpeg -f gdigrab -framerate 30 -i desktop \\
        #     -frames:v 1 -f image2 -vcodec png -
        # Then pipe stdout into Pillow.
        cmd = [
            "docker", "exec", h.container_id,
            "ffmpeg", "-f", "gdigrab", "-framerate", "30", "-i", "desktop",
            "-frames:v", "1", "-f", "image2", "-vcodec", "png", "-",
        ]
        logger.info(f"ffmpeg streaming TBD; cmdline: {' '.join(cmd)}")

        raise NotImplementedError(
            "Docker capture_frame pipeline scaffolded; container-side ffmpeg "
            "service not yet integrated. Track in v0.26.0 task list."
        )

    def capture_screen(self, monitor: int = 0) -> Frame:
        """Per-monitor capture (same TBD as capture_frame)."""
        return self.capture_frame()

    # ------------------------------------------------------------------
    # Input injection (SCAFFOLD — docker exec cmdline only)
    # ------------------------------------------------------------------

    def inject_input(
        self,
        kind: str,
        payload: dict,
        handle: Optional[DockerLaunchHandle] = None,
    ) -> bool:
        """Inject keyboard/mouse input into the container.

        SCAFFOLD: builds the ``docker exec`` cmdline that *would* invoke the
        container-side input-injection helper, but the helper binary (planned:
        a small ``input-shim.exe`` calling SendInput) is not yet present in
        the image.

        Raises:
            NotImplementedError: until the in-container input-shim is built.
        """
        h = handle or self._handle
        if h is None:
            raise RuntimeError("inject_input: no container handle available")

        cmd = [
            "docker", "exec", h.container_id,
            r"C:\\dinoforge\\input-shim.exe",
            "--kind", kind,
            "--payload", str(payload),
        ]
        logger.info(f"docker exec input shim TBD; cmdline: {' '.join(cmd)}")

        raise NotImplementedError(
            f"Docker inject_input ({kind}) scaffolded; in-container input shim "
            "not yet built. Track in v0.26.0 task list."
        )

    # ------------------------------------------------------------------
    # IsolationBackend abstract conformance
    #
    # The base IsolationBackend is async; DockerBackend provides synchronous
    # implementations of the lifecycle/capture/inject methods above and adapts
    # them here so callers using the abstract interface still work.
    # ------------------------------------------------------------------

    async def capture_window(self, title: str) -> Frame:
        # Delegate to sync capture_frame; title is unused — Docker has 1 window per container.
        return self.capture_frame()

    async def capture_display(self, monitor: int = 0) -> Frame:
        return self.capture_screen(monitor)

    async def inject_key(self, key: str, duration: float = 0.05) -> bool:
        return self.inject_input("key", {"key": key, "duration": duration})

    async def type_text(self, text: str) -> bool:
        return self.inject_input("text", {"text": text})

    async def mouse_click(self, x: int, y: int, button: str = "left") -> bool:
        return self.inject_input("click", {"x": x, "y": y, "button": button})

    async def mouse_scroll(self, x: int, y: int, delta: int) -> bool:
        return self.inject_input("scroll", {"x": x, "y": y, "delta": delta})

    async def list_windows(self) -> List[WindowInfo]:
        # In Docker we report a single synthetic "window" per running container.
        if self._handle is None:
            return []
        return [WindowInfo(
            hwnd=0,
            title=f"docker:{self._handle.docker_id_short()}",
            process_id=0,
            visible=True,
        )]

    async def focus_window(self, title: str) -> bool:
        # Docker containers don't have focusable windows in the conventional sense.
        # Returning True so callers that expect a no-op focus succeed gracefully.
        logger.debug(f"DockerBackend.focus_window('{title}') — no-op")
        return True

    async def launch_process(
        self, exe: str, args: Optional[List[str]] = None, cwd: Optional[str] = None,
    ) -> int:
        handle = self.launch_container(exe_path=exe, args=args)
        # Return the truncated docker id as a synthetic 'pid'.
        try:
            return int(handle.container_id[:8], 16)
        except ValueError:
            return 0
