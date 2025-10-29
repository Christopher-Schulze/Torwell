import type { MetricPoint } from '$lib/stores/torStore';

export interface MetricSeries {
  length: number;
  memoryMB: Float64Array;
  circuitCount: Float64Array;
  latencyMs: Float64Array;
  oldestAge: Float64Array;
  avgCreateMs: Float64Array;
  failedAttempts: Float64Array;
  cpuPercent: Float64Array;
  networkBytes: Float64Array;
  networkTotal: Float64Array;
}

function createBuffer(length: number): MetricSeries {
  return {
    length,
    memoryMB: new Float64Array(length),
    circuitCount: new Float64Array(length),
    latencyMs: new Float64Array(length),
    oldestAge: new Float64Array(length),
    avgCreateMs: new Float64Array(length),
    failedAttempts: new Float64Array(length),
    cpuPercent: new Float64Array(length),
    networkBytes: new Float64Array(length),
    networkTotal: new Float64Array(length),
  };
}

function toFinite(value: number | undefined | null): number {
  if (typeof value !== 'number' || Number.isNaN(value) || !Number.isFinite(value)) {
    return 0;
  }
  return value;
}

export function buildMetricSeries(points: MetricPoint[]): MetricSeries {
  const buffer = createBuffer(points.length);
  points.forEach((point, index) => {
    buffer.memoryMB[index] = toFinite(point.memoryMB);
    buffer.circuitCount[index] = toFinite(point.circuitCount);
    buffer.latencyMs[index] = toFinite(point.latencyMs as number | undefined);
    buffer.oldestAge[index] = toFinite(point.oldestAge);
    buffer.avgCreateMs[index] = toFinite(point.avgCreateMs);
    buffer.failedAttempts[index] = toFinite(point.failedAttempts);
    buffer.cpuPercent[index] = toFinite(point.cpuPercent);
    buffer.networkBytes[index] = toFinite(point.networkBytes);
    buffer.networkTotal[index] = toFinite(point.networkTotal);
  });
  return buffer;
}

export function sliceMetricSeries(series: MetricSeries, start: number, end: number): MetricSeries {
  const length = Math.max(0, end - start);
  const slice = createBuffer(length);
  slice.memoryMB.set(series.memoryMB.subarray(start, end));
  slice.circuitCount.set(series.circuitCount.subarray(start, end));
  slice.latencyMs.set(series.latencyMs.subarray(start, end));
  slice.oldestAge.set(series.oldestAge.subarray(start, end));
  slice.avgCreateMs.set(series.avgCreateMs.subarray(start, end));
  slice.failedAttempts.set(series.failedAttempts.subarray(start, end));
  slice.cpuPercent.set(series.cpuPercent.subarray(start, end));
  slice.networkBytes.set(series.networkBytes.subarray(start, end));
  slice.networkTotal.set(series.networkTotal.subarray(start, end));
  return slice;
}
