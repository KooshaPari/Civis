import { useEffect, useRef } from "react";
import {
  averageFps,
  averageFrameMs,
  sparklinePoints,
  sparklineScaleMax,
} from "./lib/framePerf";

type SparklineProps = {
  samples: number[];
  width?: number;
  height?: number;
  className?: string;
};

export function Sparkline({
  samples,
  width = 220,
  height = 48,
  className = "sparkline-canvas",
}: SparklineProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.round(width * dpr);
    canvas.height = Math.round(height * dpr);
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, width, height);

    const maxMs = sparklineScaleMax(samples);
    const points = sparklinePoints(samples, width, height, maxMs);

    ctx.fillStyle = "rgba(126, 198, 255, 0.08)";
    ctx.strokeStyle = "rgba(126, 198, 255, 0.95)";
    ctx.lineWidth = 1.5;
    ctx.lineJoin = "round";
    ctx.lineCap = "round";

    if (points.length === 0) {
      ctx.strokeStyle = "rgba(146, 163, 196, 0.35)";
      ctx.beginPath();
      ctx.moveTo(0, height / 2);
      ctx.lineTo(width, height / 2);
      ctx.stroke();
      return;
    }

    if (points.length === 1) {
      ctx.fillStyle = "rgba(126, 198, 255, 0.95)";
      ctx.beginPath();
      ctx.arc(points[0].x, points[0].y, 2, 0, Math.PI * 2);
      ctx.fill();
      return;
    }

    ctx.beginPath();
    points.forEach((point, index) => {
      if (index === 0) ctx.moveTo(point.x, point.y);
      else ctx.lineTo(point.x, point.y);
    });
    ctx.stroke();

    ctx.lineTo(points[points.length - 1].x, height);
    ctx.lineTo(points[0].x, height);
    ctx.closePath();
    ctx.fill();
  }, [samples, width, height]);

  return (
    <canvas
      ref={canvasRef}
      className={className}
      width={width}
      height={height}
      role="img"
      aria-label={`Frame interval sparkline, average ${averageFps(samples).toFixed(0)} FPS`}
    />
  );
}

export function formatPerfSummary(samples: number[]): {
  fps: number;
  frameMs: number;
  count: number;
} {
  const frameMs = averageFrameMs(samples);
  return {
    fps: averageFps(samples),
    frameMs,
    count: samples.length,
  };
}
