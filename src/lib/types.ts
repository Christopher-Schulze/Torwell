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

export interface RelayInfo {
  nickname: string;
  ip_address: string;
  country: string;
}

export interface CircuitPolicyReport {
  requested_entry: string | null;
  requested_middle: string | null;
  requested_exit: string | null;
  effective_entry: string | null;
  effective_middle: string | null;
  effective_exit: string | null;
  matches_policy: boolean;
  relays: RelayInfo[];
}

export interface TorrcProfile {
  generated_at: string;
  config: string;
  entry: string;
  middle: string;
  exit: string;
  requested_entry: string | null;
  requested_middle: string | null;
  requested_exit: string | null;
  fast_fallback: string[];
  bridges: string[];
  fast_only: boolean;
}
