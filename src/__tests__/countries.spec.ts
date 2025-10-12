import { describe, it, expect } from "vitest";
import {
  ensureUniqueRoute,
  getCountryFlag,
  getCountryLabel,
  isKnownCountry,
  normaliseCountryCode,
  DEFAULT_ROUTE_CODES,
} from "../lib/utils/countries";

describe("countries utility", () => {
  it("normalises codes to uppercase ISO strings", () => {
    expect(normaliseCountryCode("de")).toBe("DE");
    expect(normaliseCountryCode(" Fr ")).toBe("FR");
    expect(normaliseCountryCode("invalid")).toBeNull();
  });

  it("ensures unique routes using fallbacks", () => {
    const route = ensureUniqueRoute(["DE", "DE", "DE"], DEFAULT_ROUTE_CODES);
    expect(new Set(route).size).toBe(route.length);
  });

  it("provides readable labels and flags", () => {
    expect(getCountryLabel("DE")).toBe("Germany");
    expect(Array.from(getCountryFlag("DE")).length).toBe(2);
  });

  it("detects known countries", () => {
    expect(isKnownCountry("DE")).toBe(true);
    expect(isKnownCountry("XX")).toBe(false);
  });
});
