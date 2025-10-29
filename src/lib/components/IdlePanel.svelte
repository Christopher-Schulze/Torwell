<script lang="ts">
  import { tweened } from "svelte/motion";
  import { cubicOut } from "svelte/easing";
  import { derived } from "svelte/store";
  import { reducedMotion } from "$lib/utils/motion";

  export let connectionProgress = 0; // 0-100
  export let currentStatus = "Idle"; // Current Tor status
  export let retryCount = 0;
  export let retryDelay = 0;
  export let bootstrapMessage = "";
  export let errorStep: string | null = null;
  export let errorSource: string | null = null;

  const statusMap: Record<string, string> = {
    DISCONNECTED: "Disconnected",
    CONNECTING: "Connecting",
    RETRYING: "Retrying",
    CONNECTED: "Connected",
    DISCONNECTING: "Disconnecting",
    ERROR: "Error",
  };

  const progressValue = tweened(connectionProgress, {
    duration: 420,
    easing: cubicOut,
  });

  const easedProgress = derived([progressValue, reducedMotion], ([$value, $reduced]) =>
    Number.isFinite($value) && !$reduced ? $value : connectionProgress,
  );

  $: progressValue.set(connectionProgress, {
    duration: $reducedMotion ? 0 : 420,
    easing: cubicOut,
  });

  $: statusText = statusMap[currentStatus] ?? currentStatus;

  const statusMeta: Record<string, { tone: string; helper: string }> = {
    DISCONNECTED: {
      tone: "text-slate-200",
      helper: "Bereit, eine Verbindung aufzubauen.",
    },
    CONNECTING: {
      tone: "text-amber-200",
      helper: "Tor-Bootstrap läuft – bitte warten.",
    },
    RETRYING: {
      tone: "text-amber-200",
      helper: "Erneuter Versuch mit Backoff aktiviert.",
    },
    CONNECTED: {
      tone: "text-emerald-200",
      helper: "Tor-Tunnel aktiv und stabil.",
    },
    DISCONNECTING: {
      tone: "text-blue-200",
      helper: "Trenne aktuell die Tor-Sitzung.",
    },
    ERROR: {
      tone: "text-rose-200",
      helper: "Eingriff erforderlich – siehe Fehlermeldung unten.",
    },
  };

  $: meta = statusMeta[currentStatus] ?? {
    tone: "text-slate-200",
    helper: "Status aktualisiert",
  };
</script>

<div
  class="glass-md relative overflow-hidden rounded-3xl p-6"
  role="region"
  aria-label="Connection progress"
>
  <div
    class="pointer-events-none absolute -inset-12 bg-gradient-to-br from-sky-500/15 via-indigo-500/10 to-purple-500/5 blur-3xl"
    class:opacity-0={$reducedMotion}
  ></div>

  <div class="flex w-full flex-col gap-5">
    <div class="space-y-3">
      <div
        class="relative h-3 w-full overflow-hidden rounded-full border border-white/10 bg-white/10"
        role="progressbar"
        aria-valuemin="0"
        aria-valuemax="100"
        aria-valuenow={connectionProgress}
      >
        <div
          class="absolute inset-0 bg-gradient-to-r from-sky-300/60 via-indigo-400/70 to-purple-500/60 opacity-90"
          style={`width: ${Math.min(100, Math.max(0, $easedProgress))}%`}
        ></div>
        <div
          class="absolute inset-0 animate-[pulse_4s_ease-in-out_infinite] bg-white/40 mix-blend-overlay"
          class:opacity-0={$reducedMotion}
        ></div>
        <div
          class="pointer-events-none absolute inset-0 bg-gradient-to-r from-transparent via-white/25 to-transparent opacity-60 animate-[tw-orbit_22s_linear_infinite]"
          class:opacity-0={$reducedMotion}
        ></div>
      </div>
      <div class="flex items-baseline justify-between text-xs">
        <span class={`font-semibold uppercase tracking-[0.24em] text-white/80`}>Bootstrap</span>
        <span class="font-mono text-sm text-slate-100">{Math.round($easedProgress)}%</span>
      </div>
    </div>

    {#if bootstrapMessage}
      <p class="text-xs text-slate-200/80 italic" aria-live="polite">
        {bootstrapMessage}
      </p>
    {/if}

    <div class="rounded-2xl border border-white/10 bg-slate-900/40 p-4 shadow-inner">
      <div class="flex flex-col gap-2" aria-live="polite">
        <div class="flex items-center gap-3">
          <span class="inline-flex h-2.5 w-2.5 rounded-full bg-gradient-to-r from-emerald-300 via-sky-400 to-violet-400 shadow-[0_0_12px_rgba(94,234,212,0.45)]"></span>
          <div class="flex flex-col">
            <span class={`text-sm font-semibold leading-tight ${meta.tone}`}>
              {statusText}
            </span>
            <span class="text-[11px] text-slate-300/80">{meta.helper}</span>
          </div>
        </div>

        {#if currentStatus === "RETRYING"}
          <p class="text-xs text-amber-200/90">
            Wiederholung in {retryDelay}s · Versuch {retryCount}
          </p>
        {:else if currentStatus === "ERROR"}
          <p class="text-xs text-rose-200/95">
            Verbindungsfehler
            {#if errorStep}
              – {errorStep}
              {#if errorSource}
                : {errorSource}
              {/if}
            {/if}
          </p>
        {/if}
      </div>
    </div>
  </div>
</div>
