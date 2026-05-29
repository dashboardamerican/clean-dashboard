export interface ResourceSweepElccInput {
  capacity: number;
  peakGas: number;
}

export interface ResourceSweepElccOutput {
  avg_elcc?: number;
  marginal_elcc?: number;
}

export function calculateResourceSweepElcc(
  points: ResourceSweepElccInput[]
): ResourceSweepElccOutput[] {
  if (points.length === 0) {
    return [];
  }

  const baselinePeakGas = points[0].peakGas;

  return points.map((point, index) => {
    if (!Number.isFinite(baselinePeakGas) || !Number.isFinite(point.peakGas)) {
      return {};
    }

    const avg_elcc =
      point.capacity > 0 ? ((baselinePeakGas - point.peakGas) / point.capacity) * 100 : 0;

    if (index === 0) {
      return {
        avg_elcc,
        marginal_elcc: 0,
      };
    }

    const previous = points[index - 1];
    if (!Number.isFinite(previous.peakGas)) {
      return {
        avg_elcc,
      };
    }

    const capDiff = point.capacity - previous.capacity;
    return {
      avg_elcc,
      marginal_elcc: capDiff > 0 ? ((previous.peakGas - point.peakGas) / capDiff) * 100 : 0,
    };
  });
}
