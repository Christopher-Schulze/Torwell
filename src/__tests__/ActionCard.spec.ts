import { vi, describe, it, expect, beforeEach } from "vitest";
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));
vi.mock("@tauri-apps/api", () => ({ invoke: vi.fn() }));
import { render, fireEvent } from "@testing-library/svelte";
import ActionCard from "../lib/components/ActionCard.svelte";
import { invoke } from "@tauri-apps/api";
import { connectionQueue } from "../lib/utils/actionQueue";

// Reset store between tests by importing after test to ensure fresh instance
import { torStore } from "../lib/stores/torStore";

describe("ActionCard", () => {
  beforeEach(() => {
    connectionQueue.reset();
  });

  it("renders Connect button when stopped", () => {
    torStore.set({
      status: "DISCONNECTED",
      bootstrapProgress: 0,
      bootstrapMessage: "",
      errorMessage: null,
      errorStep: null,
      errorSource: null,
      securityWarning: null,
      retryCount: 0,
      retryDelay: 0,
      memoryUsageMB: 0,
      circuitCount: 0,
      pingMs: undefined,
      metrics: [],
      lastTransition: null,
      systemProxyEnabled: false,
    });

    const { getByRole } = render(ActionCard);
    expect(getByRole("region")).toHaveAttribute("aria-label", "Tor controls");
    expect(
      getByRole("button", { name: /connect to tor/i }),
    ).toBeInTheDocument();
  });

  it("dispatches openLogs event when Logs button is clicked", async () => {
    const { getByRole, component } = render(ActionCard);
    const handler = vi.fn();
    component.$on("openLogs", handler);
    await fireEvent.click(getByRole("button", { name: /open logs/i }));
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it("calls disconnect when Disconnect button clicked", async () => {
    torStore.set({
      status: "CONNECTED",
      bootstrapProgress: 0,
      bootstrapMessage: "",
      errorMessage: null,
      errorStep: null,
      errorSource: null,
      securityWarning: null,
      retryCount: 0,
      retryDelay: 0,
      memoryUsageMB: 0,
      circuitCount: 0,
      pingMs: undefined,
      metrics: [],
      lastTransition: null,
      systemProxyEnabled: false,
    });

    const { getByRole } = render(ActionCard);
    await fireEvent.click(
      getByRole("button", { name: /disconnect from tor/i }),
    );
    expect(invoke).toHaveBeenCalledWith("disconnect");
  });
});
