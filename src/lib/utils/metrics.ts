import type { MetricPoint } from "../stores/torStore";

export type MetricNumericField =
  | "memoryMB"
  | "circuitCount"
  | "latencyMs"
  | "oldestAge"
  | "avgCreateMs"
  | "failedAttempts"
  | "cpuPercent"
  | "networkBytes"
  | "networkTotal";

export type TrendDirection = "up" | "down" | "flat";

export interface TrendDetails {
  direction: TrendDirection;
  change: number;
  percent: number;
  previous: number;
  current: number;
}

export interface MetricSummary {
  field: MetricNumericField;
  current: number;
  average: number;
  min: number;
  max: number;
  trend: TrendDetails;
  values: number[];
}

export type Severity = "good" | "warning" | "critical";

export interface Thresholds {
  warning: number;
  critical: number;
  direction?: "higher-is-worse" | "lower-is-worse";
}

export interface HealthAssessment {
  title: string;
  severity: Severity;
  detail: string;
  hint?: string;
}

const DEFAULT_TREND_WINDOW = 5;
const FLAT_TOLERANCE_PERCENT = 2;

function pickNumericFieldValue(point: MetricPoint, field: MetricNumericField): number {
  const value = point[field];
  if (typeof value !== "number" || Number.isNaN(value)) {
    return 0;
  }
  return value;
}

export function getRecentWindow(metrics: MetricPoint[], size: number): MetricPoint[] {
  if (!metrics.length) return [];
  const limit = Math.max(1, Math.floor(size));
  return metrics.slice(-limit);
}

export function calculateTrend(
  metrics: MetricPoint[],
  field: MetricNumericField,
  window: number = DEFAULT_TREND_WINDOW
): TrendDetails {
  if (!metrics.length) {
    return { direction: "flat", change: 0, percent: 0, previous: 0, current: 0 };
  }

  const sample = metrics.slice(-Math.max(2, window));
  const first = pickNumericFieldValue(sample[0], field);
  const last = pickNumericFieldValue(sample[sample.length - 1], field);
  const change = last - first;
  const base = Math.abs(first) < 1e-3 ? 1 : first;
  const percent = (change / base) * 100;

  let direction: TrendDirection = "flat";
  if (Math.abs(percent) <= FLAT_TOLERANCE_PERCENT) {
    direction = "flat";
  } else if (change > 0) {
    direction = "up";
  } else if (change < 0) {
    direction = "down";
  }

  return { direction, change, percent, previous: first, current: last };
}

export function summarizeMetric(
  metrics: MetricPoint[],
  field: MetricNumericField,
  options?: { window?: number }
): MetricSummary | null {
  if (!metrics.length) return null;
  const window = options?.window;
  const scope = typeof window === "number" ? metrics.slice(-Math.max(1, window)) : metrics;
  const values = scope.map((point) => pickNumericFieldValue(point, field));
  if (!values.length) return null;
  const current = values[values.length - 1];
  const average = values.reduce((sum, value) => sum + value, 0) / values.length;
  const min = Math.min(...values);
  const max = Math.max(...values);
  const trend = calculateTrend(scope, field, window ?? DEFAULT_TREND_WINDOW);
  return { field, current, average, min, max, trend, values };
}

export function resolveSeverity(value: number, thresholds?: Thresholds): Severity {
  if (!thresholds) return "good";
  const { warning, critical, direction = "higher-is-worse" } = thresholds;
  if (direction === "higher-is-worse") {
    if (value >= critical) return "critical";
    if (value >= warning) return "warning";
    return "good";
  }
  if (value <= critical) return "critical";
  if (value <= warning) return "warning";
  return "good";
}

export function describeTrend(trend: TrendDetails, unit = ""): string {
  if (trend.direction === "flat" || Math.abs(trend.change) < 1e-1) {
    return "Stabil.";
  }
  const formattedChange = trend.change.toFixed(1);
  if (trend.direction === "up") {
    return `Steigend (+${formattedChange}${unit}).`;
  }
  return `Fallend (${formattedChange}${unit}).`;
}

export function formatRelativeTime(timestamp: number | null | undefined): string {
  if (!timestamp) return "Keine Daten";
  const delta = Date.now() - timestamp;
  if (delta < 1_000) return "gerade eben";
  const seconds = Math.round(delta / 1_000);
  if (seconds < 60) return `vor ${seconds} s`;
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `vor ${minutes} min`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `vor ${hours} h`;
  const days = Math.round(hours / 24);
  return `vor ${days} d`;
}

export function evaluateTorHealth(metrics: MetricPoint[]): HealthAssessment[] {
  const latest = metrics.length ? metrics[metrics.length - 1] : undefined;
  if (!latest) {
    return [
      {
        title: "Metriken",
        severity: "warning",
        detail: "Noch keine Metriken empfangen.",
        hint: "Stellen Sie sicher, dass eine Tor-Verbindung aktiv ist.",
      },
    ];
  }

  const memoryTrend = calculateTrend(metrics, "memoryMB");
  const memorySeverity = resolveSeverity(latest.memoryMB, {
    warning: 900,
    critical: 1100,
    direction: "higher-is-worse",
  });

  const circuitTrend = calculateTrend(metrics, "circuitCount");
  const circuitSeverity = resolveSeverity(latest.circuitCount, {
    warning: 12,
    critical: 16,
    direction: "higher-is-worse",
  });

  const latencyTrend = calculateTrend(metrics, "latencyMs");
  const latestLatency = typeof latest.latencyMs === "number" ? latest.latencyMs : 0;
  const latencySeverity = resolveSeverity(latestLatency, {
    warning: 300,
    critical: 600,
    direction: "higher-is-worse",
  });

  const failureTrend = calculateTrend(metrics, "failedAttempts");
  const failureSeverity = resolveSeverity(latest.failedAttempts, {
    warning: 2,
    critical: 5,
    direction: "higher-is-worse",
  });

  const throughputTrend = calculateTrend(metrics, "networkBytes");
  const throughputSeverity = resolveSeverity(latest.networkBytes, {
    warning: 20_000,
    critical: 5_000,
    direction: "lower-is-worse",
  });

  const assessments: HealthAssessment[] = [
    {
      title: "Speicherauslastung",
      severity: memorySeverity,
      detail: `Aktuell ${latest.memoryMB} MB. ${describeTrend(memoryTrend, " MB")}`,
      hint:
        memorySeverity === "good"
          ? "Keine Aktion erforderlich."
          : "Schließen Sie inaktive Tabs oder erzwingen Sie eine neue Identität, um Speicher freizugeben.",
    },
    {
      title: "Circuit-Pool",
      severity: circuitSeverity,
      detail: `Bereitgestellte Circuits: ${latest.circuitCount}. ${describeTrend(circuitTrend, "")}`,
      hint:
        circuitSeverity === "good"
          ? "Ausreichende Circuit-Reserven verfügbar."
          : "Senken Sie die Parallelität oder starten Sie den Tor-Dienst neu.",
    },
    {
      title: "Latenz",
      severity: latencySeverity,
      detail: `End-to-End-Latenz: ${latestLatency} ms. ${describeTrend(latencyTrend, " ms")}`,
      hint:
        latencySeverity === "good"
          ? "Verbindung reagiert normal."
          : "Wählen Sie schnellere Relays oder wechseln Sie die Route.",
    },
    {
      title: "Fehlgeschlagene Versuche",
      severity: failureSeverity,
      detail: `Fehlerhäufigkeit: ${latest.failedAttempts}. ${describeTrend(failureTrend, "")}`,
      hint:
        failureSeverity === "good"
          ? "Keine auffälligen Fehler."
          : "Überprüfen Sie die Bridge- oder Firewall-Konfiguration.",
    },
    {
      title: "Durchsatz",
      severity: throughputSeverity,
      detail: `Aktueller Durchsatz: ${latest.networkBytes} B/s. ${describeTrend(throughputTrend, " B/s")}`,
      hint:
        throughputSeverity === "good"
          ? "Datenübertragung aktiv."
          : "Überprüfen Sie die Bandbreite oder warten Sie auf neue Verbindungen.",
    },
  ];

  return assessments;
}

export function toRounded(value: number, digits = 1): number {
  const factor = Math.pow(10, digits);
  return Math.round(value * factor) / factor;
}

export function humanizeBytes(bytes: number, fractionDigits = 1): string {
  if (Number.isNaN(bytes)) return "0 B";
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / Math.pow(1024, exponent);
  return `${value.toFixed(fractionDigits)} ${units[exponent]}`;
}

export function computeRollingAverage(values: number[], window: number): number[] {
  if (!values.length) return [];
  const size = Math.max(1, window);
  const result: number[] = [];
  for (let i = 0; i < values.length; i += 1) {
    const slice = values.slice(Math.max(0, i - size + 1), i + 1);
    const average = slice.reduce((sum, val) => sum + val, 0) / slice.length;
    result.push(average);
  }
  return result;
}
