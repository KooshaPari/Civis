import { useEffect, useState } from "react";
import { mockDevFrameMs } from "../lib/framePerf";
import type { FrameSampleSource } from "../store";

type FramePerfDispatch = React.Dispatch<
  | { type: "push_frame_sample"; ms: number; source?: FrameSampleSource }
  | { type: "set_frame_sample_source"; source: FrameSampleSource }
  | { type: "reset_frame_samples" }
>;

/**
 * When no live attach stream is available, seed the sparkline from rAF in dev.
 */
export function useFramePerfMock(
  connection: "live" | "reconnecting" | "disconnected",
  enabled: boolean,
  dispatch: FramePerfDispatch,
) {
  const [isVisible, setIsVisible] = useState(
    () => typeof document !== "undefined" && document.visibilityState === "visible",
  );

  useEffect(() => {
    const onVisibilityChange = () => setIsVisible(document.visibilityState === "visible");
    document.addEventListener("visibilitychange", onVisibilityChange);
    return () => document.removeEventListener("visibilitychange", onVisibilityChange);
  }, []);

  useEffect(() => {
    if (!import.meta.env.DEV) return;
    if (connection === "live") {
      dispatch({ type: "set_frame_sample_source", source: "attach" });
      return;
    }
    if (connection !== "disconnected" || !enabled || !isVisible) return;

    dispatch({ type: "set_frame_sample_source", source: "mock" });
    let raf = 0;
    let index = 0;

    const tick = () => {
      dispatch({
        type: "push_frame_sample",
        ms: mockDevFrameMs(index),
        source: "mock",
      });
      index += 1;
      raf = window.requestAnimationFrame(tick);
    };

    raf = window.requestAnimationFrame(tick);
    return () => window.cancelAnimationFrame(raf);
  }, [connection, dispatch, enabled, isVisible]);
}
