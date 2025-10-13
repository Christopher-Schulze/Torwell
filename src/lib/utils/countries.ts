export type CountryOption = {
  code: string;
  name: string;
};

const RAW_COUNTRIES: CountryOption[] = [
  { code: "AL", name: "Albania" },
  { code: "AR", name: "Argentina" },
  { code: "AT", name: "Austria" },
  { code: "AU", name: "Australia" },
  { code: "BE", name: "Belgium" },
  { code: "BG", name: "Bulgaria" },
  { code: "BR", name: "Brazil" },
  { code: "CA", name: "Canada" },
  { code: "CH", name: "Switzerland" },
  { code: "CL", name: "Chile" },
  { code: "CN", name: "China" },
  { code: "CY", name: "Cyprus" },
  { code: "CZ", name: "Czech Republic" },
  { code: "DE", name: "Germany" },
  { code: "DK", name: "Denmark" },
  { code: "EE", name: "Estonia" },
  { code: "ES", name: "Spain" },
  { code: "FI", name: "Finland" },
  { code: "FR", name: "France" },
  { code: "GB", name: "United Kingdom" },
  { code: "GR", name: "Greece" },
  { code: "HU", name: "Hungary" },
  { code: "IE", name: "Ireland" },
  { code: "IL", name: "Israel" },
  { code: "IN", name: "India" },
  { code: "IS", name: "Iceland" },
  { code: "IT", name: "Italy" },
  { code: "JP", name: "Japan" },
  { code: "KR", name: "South Korea" },
  { code: "LT", name: "Lithuania" },
  { code: "LU", name: "Luxembourg" },
  { code: "LV", name: "Latvia" },
  { code: "MT", name: "Malta" },
  { code: "MX", name: "Mexico" },
  { code: "NL", name: "Netherlands" },
  { code: "NO", name: "Norway" },
  { code: "NZ", name: "New Zealand" },
  { code: "PL", name: "Poland" },
  { code: "PT", name: "Portugal" },
  { code: "RO", name: "Romania" },
  { code: "RS", name: "Serbia" },
  { code: "RU", name: "Russia" },
  { code: "SE", name: "Sweden" },
  { code: "SG", name: "Singapore" },
  { code: "SI", name: "Slovenia" },
  { code: "SK", name: "Slovakia" },
  { code: "TR", name: "Turkey" },
  { code: "UA", name: "Ukraine" },
  { code: "US", name: "United States" },
];

export const COUNTRY_OPTIONS: ReadonlyArray<CountryOption> = RAW_COUNTRIES.sort((a, b) =>
  a.name.localeCompare(b.name),
);

const COUNTRY_NAME_BY_CODE = new Map(
  COUNTRY_OPTIONS.map((option) => [option.code, option.name]),
);

const FAST_COUNTRY_CODES: ReadonlyArray<string> = [
  "CA",
  "CH",
  "DE",
  "DK",
  "EE",
  "FI",
  "FR",
  "GB",
  "IS",
  "JP",
  "LT",
  "LU",
  "LV",
  "NL",
  "NO",
  "SE",
  "SG",
  "US",
];

const FAST_COUNTRY_CODE_SET = new Set(FAST_COUNTRY_CODES);

export const DEFAULT_FAST_COUNTRY_CODES: ReadonlyArray<string> = FAST_COUNTRY_CODES;

export function createFastCountrySet(
  extra?: Iterable<string | null | undefined>,
): Set<string> {
  const set = new Set(FAST_COUNTRY_CODE_SET);
  if (!extra) {
    return set;
  }
  for (const value of extra) {
    const code = normaliseCountryCode(value);
    if (code) {
      set.add(code);
    }
  }
  return set;
}

export const DEFAULT_ROUTE_CODES = ["DE", "NL", "SE"] as const;

export function normaliseCountryCode(value: string | null | undefined): string | null {
  if (!value) return null;
  const trimmed = value.trim().toUpperCase();
  return /^[A-Z]{2}$/.test(trimmed) ? trimmed : null;
}

export function getCountryName(value: string | null | undefined): string | null {
  const code = normaliseCountryCode(value);
  return code ? COUNTRY_NAME_BY_CODE.get(code) ?? null : null;
}

export function getCountryLabel(value: string | null | undefined): string {
  return getCountryName(value) ?? "Unknown";
}

export function getCountryFlag(value: string | null | undefined): string {
  const code = normaliseCountryCode(value);
  if (!code) return "🏳";
  const codePoints = [...code].map((char) => 127397 + char.charCodeAt(0));
  return String.fromCodePoint(...codePoints);
}

export function ensureUniqueRoute(
  codes: Array<string | null | undefined>,
  fallbacks: ReadonlyArray<string> = DEFAULT_ROUTE_CODES,
): string[] {
  const used = new Set<string>();
  return codes.map((value, index) => {
    const normalised = normaliseCountryCode(value);
    if (normalised && !used.has(normalised)) {
      used.add(normalised);
      return normalised;
    }

    const fallback = fallbacks.find((code) => !used.has(code));
    if (fallback) {
      used.add(fallback);
      return fallback;
    }

    if (normalised) {
      return normalised;
    }

    return fallbacks[index] ?? COUNTRY_OPTIONS[0]?.code ?? "US";
  });
}

export function isKnownCountry(value: string | null | undefined): boolean {
  const code = normaliseCountryCode(value);
  return !!(code && COUNTRY_NAME_BY_CODE.has(code));
}

export function isFastCountry(
  value: string | null | undefined,
  fastSet: ReadonlySet<string> = FAST_COUNTRY_CODE_SET,
): boolean {
  const code = normaliseCountryCode(value);
  return !!(code && fastSet.has(code));
}
