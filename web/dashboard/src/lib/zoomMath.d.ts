export function zoomDistanceFromWheel(args: {
  distance: number;
  deltaY: number;
  minDistance: number;
  maxDistance: number;
}): number;

export function zoomDistanceRoundTrip(args: {
  distance: number;
  deltaY: number;
  minDistance: number;
  maxDistance: number;
}): number;
