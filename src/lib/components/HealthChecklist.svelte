<script lang="ts">
  import type { HealthAssessment } from '$lib/utils/metrics';

  export let assessments: HealthAssessment[] = [];

  const badgeClass = {
    good: 'bg-emerald-500/10 text-emerald-200 border-emerald-400/40',
    warning: 'bg-amber-500/10 text-amber-100 border-amber-400/40',
    critical: 'bg-rose-500/10 text-rose-100 border-rose-400/40',
  } as const;
</script>

<div class="space-y-3">
  {#if assessments.length === 0}
    <p class="rounded-lg border border-slate-700/60 bg-slate-900/40 p-4 text-sm text-slate-200">
      Es liegen keine Gesundheitsbewertungen vor.
    </p>
  {:else}
    {#each assessments as item}
      <article class={`rounded-xl border px-4 py-3 tw-surface ${badgeClass[item.severity]}`}>
        <header class="flex items-start justify-between gap-3">
          <h3 class="text-sm font-semibold text-white">{item.title}</h3>
          <span class="rounded-full border px-2 py-0.5 text-xs uppercase tracking-wide">
            {item.severity === 'good' ? 'stabil' : item.severity === 'warning' ? 'beobachten' : 'kritisch'}
          </span>
        </header>
        <p class="mt-2 text-sm text-slate-100">{item.detail}</p>
        {#if item.hint}
          <p class="mt-1 text-xs text-slate-300">{item.hint}</p>
        {/if}
      </article>
    {/each}
  {/if}
</div>
