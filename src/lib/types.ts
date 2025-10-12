export interface StatusSummary {
  status: string;
  connected_since: string | null;
  uptime_seconds: number | null;
  total_traffic_bytes: number;
  network_bytes_per_sec: number;
  total_network_bytes: number;
  latency_ms: number;
  memory_bytes: number;
  circuit_count: number;
  oldest_circuit_age: number;
  cpu_percent: number;
  tray_warning: string | null;
  retry_count: number;
}
