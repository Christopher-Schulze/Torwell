import { render } from "@testing-library/svelte";
import { vi, describe, it, expect } from "vitest";
import { tick } from "svelte";

let metricsCallback: (e: any) => void = () => {};
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((ev: string, cb: any) => {
    if (ev === "metrics-update") metricsCallback = cb;
    return Promise.resolve(() => {});
  }),
}));

import ResourceDashboard from "../lib/components/ResourceDashboard.svelte";

describe("ResourceDashboard", () => {
  it("updates metrics and shows warnings", async () => {
    const { getByText, getAllByRole, container } = render(ResourceDashboard);
    await tick();
    metricsCallback({
      payload: {
        memory_bytes: 1500_000_000,
        circuit_count: 25,
        latency_ms: 0,
        oldest_age: 0,
        avg_create_ms: 50,
        failed_attempts: 3,
        cpu_percent: 12.5,
        network_bytes: 2048,
      },
    });
    await tick();
    await tick();

    expect(getByText(/Memory: 1500 MB/)).toBeInTheDocument();
    expect(getByText(/Circuits: 25/)).toBeInTheDocument();
    expect(getByText(/Avg build: 50 ms/)).toBeInTheDocument();
    expect(getByText(/Failures: 3/)).toBeInTheDocument();
    expect(getByText(/CPU: 12.5 %/)).toBeInTheDocument();
    expect(getByText(/Network: 2048 B\/s/)).toBeInTheDocument();
    expect(getAllByRole("alert").length).toBe(2);
    expect(container).toMatchSnapshot();
  });
});
