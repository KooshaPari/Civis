export function zoomDistanceFromWheel({
  distance,
  deltaY,
  minDistance,
  maxDistance,
}) {
  const zoomFactor = Math.exp(deltaY * 0.0012);
  const nextDistance = distance * zoomFactor;
  return clamp(nextDistance, minDistance, maxDistance);
}

export function zoomDistanceRoundTrip({
  distance,
  deltaY,
  minDistance,
  maxDistance,
}) {
  const zoomed = zoomDistanceFromWheel({ distance, deltaY, minDistance, maxDistance });
  return zoomDistanceFromWheel({
    distance: zoomed,
    deltaY: -deltaY,
    minDistance,
    maxDistance,
  });
}

function clamp(value, min, max) {
  return Math.min(Math.max(value, min), max);
}
