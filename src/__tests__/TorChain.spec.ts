import { describe, it, expect, vi, afterEach } from "vitest";
import { render, fireEvent } from "@testing-library/svelte";

// Mock tauri event listener so store initialization doesn't fail
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

// Provide a minimal mock for the uiStore used by TorChain
const setExitCountry = vi.fn();
const setCircuitCountries = vi.fn();
vi.mock("$lib/stores/uiStore", () => {
  const { writable, get } = require("svelte/store");
  const store = writable({
    settings: { exitCountry: null, entryCountry: null, middleCountry: null },
    cloudflareEnabled: false,
  });
  const updateSettings = (changes: Record<string, string | null>) => {
    const current = get(store);
    store.set({
      ...current,
      settings: { ...current.settings, ...changes },
    });
  };
  return {
    uiStore: {
      subscribe: store.subscribe,
      actions: {
        setExitCountry: (country: string | null) => {
          setExitCountry(country);
          updateSettings({ exitCountry: country });
        },
        setCircuitCountries: (countries: {
          entry?: string | null;
          middle?: string | null;
          exit?: string | null;
        }) => {
          setCircuitCountries(countries);
          const current = get(store).settings;
          updateSettings({
            entryCountry:
              countries.entry !== undefined ? countries.entry : current.entryCountry,
            middleCountry:
              countries.middle !== undefined ? countries.middle : current.middleCountry,
            exitCountry:
              countries.exit !== undefined ? countries.exit : current.exitCountry,
          });
        },
        setCloudflareEnabled: (val: boolean) => {
          const current = get(store);
          store.set({ ...current, cloudflareEnabled: val });
        },
      },
    },
  };
});

import TorChain from "../lib/components/TorChain.svelte";

const nodeData = [
  { nickname: "entry", ip_address: "1.1.1.1", country: "DE" },
  { nickname: "middle", ip_address: "2.2.2.2", country: "FR" },
  { nickname: "exit", ip_address: "3.3.3.3", country: "US" },
];

afterEach(() => {
  vi.clearAllMocks();
});

describe("TorChain", () => {
  it("renders node card data only when connected", () => {
    const { queryByText: queryDisconnected, getByRole } = render(TorChain, {
      props: { isConnected: false, nodeData },
    });
    expect(getByRole("region")).toHaveAttribute(
      "aria-label",
      "Tor chain configuration",
    );
    expect(queryDisconnected("1.1.1.1")).not.toBeInTheDocument();

    const { getByText } = render(TorChain, {
      props: { isConnected: true, nodeData },
    });
    expect(getByText("1.1.1.1")).toBeInTheDocument();
    expect(getByText("entry")).toBeInTheDocument();
  });

  it("calls setExitCountry when exit dropdown changes", async () => {
    const { getByLabelText } = render(TorChain, {
      props: { isConnected: true, nodeData },
    });

    const select = getByLabelText(
      "Preferred exit country",
    ) as HTMLSelectElement;
    await fireEvent.change(select, { target: { value: "US" } });

    expect(setExitCountry).toHaveBeenCalledWith("US");
  });

  it("persists entry selection through setCircuitCountries", async () => {
    const { getByLabelText } = render(TorChain, {
      props: { isConnected: true, nodeData },
    });

    const entrySelect = getByLabelText("Entry node") as HTMLSelectElement;
    await fireEvent.change(entrySelect, { target: { value: "FR" } });

    expect(setCircuitCountries).toHaveBeenCalledWith(
      expect.objectContaining({ entry: "FR" }),
    );
  });

  it("displays isolated circuits list", () => {
    const isolatedCircuits = [
      {
        domain: "example.com",
        nodes: [
          { nickname: "n1", ip_address: "1.1.1.1", country: "DE" },
          { nickname: "n2", ip_address: "2.2.2.2", country: "FR" },
        ],
      },
    ];

    const { getByText } = render(TorChain, {
      props: { isConnected: true, nodeData, isolatedCircuits },
    });

    expect(getByText("Isolated Circuits")).toBeInTheDocument();
    expect(getByText(/example.com/)).toBeInTheDocument();
  });

  it("updates cloudflare node info when store value changes", async () => {
    const cfNode = { nickname: "cf", ip_address: "4.4.4.4", country: "NL" };
    const { uiStore } = await import("../lib/stores/uiStore");
    const { queryByText } = render(TorChain, {
      props: { isConnected: true, nodeData: [...nodeData, cfNode] },
    });

    expect(queryByText("4.4.4.4")).not.toBeInTheDocument();

    uiStore.actions.setCloudflareEnabled(true);
    await Promise.resolve();

    expect(queryByText("4.4.4.4")).toBeInTheDocument();
  });

  it("shows diagnostics when hops reuse a country", () => {
    const duplicated = [
      { nickname: "entry", ip_address: "1.1.1.1", country: "DE" },
      { nickname: "middle", ip_address: "2.2.2.2", country: "DE" },
      { nickname: "exit", ip_address: "3.3.3.3", country: "US" },
    ];

    const { getByText } = render(TorChain, {
      props: { isConnected: true, nodeData: duplicated },
    });

    expect(
      getByText(/Multiple hops share the same country/i),
    ).toBeInTheDocument();
  });
});
