import { useEffect, useState } from "react";
import { mockDevFrameMs } from "../lib/framePerf";
import type { FrameSampleSource } from "../store";

type FramePerfDispatch = React.Dispatch<
  | { type: "push_frame_sample"; ms: number; source?: FrameSampleSource }
  | { type: "set_frame_sample_source"; source: FrameSampleSource }
  | { type: "reset_frame_samples" }
>;

const MOCK_SAMPLE_INTERVAL_MS = 125;

/**
 * When no live attach stream is available, seed the sparkline from a throttled
 * synthetic frame clock in dev without driving the dashboard at render rate.
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
    let index = 0;

    const sample = () => {
      dispatch({
        type: "push_frame_sample",
        ms: mockDevFrameMs(index),
        source: "mock",
      });
      index += 1;
    };

    sample();
    const interval = window.setInterval(sample, MOCK_SAMPLE_INTERVAL_MS);
    return () => window.clearInterval(interval);
  }, [connection, dispatch, enabled, isVisible]);
}
