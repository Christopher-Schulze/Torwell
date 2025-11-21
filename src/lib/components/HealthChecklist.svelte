<script lang="ts">
  import type { HealthAssessment } from '$lib/utils/metrics';

  export let assessments: HealthAssessment[] = [];

  const badgeClass = {
    good: 'bg-neon-green/5 text-neon-green border-neon-green/30 shadow-[0_0_10px_-5px_rgba(0,255,65,0.2)]',
    warning: 'bg-neon-purple/5 text-neon-purple border-neon-purple/30 shadow-[0_0_10px_-5px_rgba(188,19,254,0.2)]',
    critical: 'bg-red-500/5 text-red-500 border-red-500/30 shadow-[0_0_10px_-5px_rgba(239,68,68,0.2)]',
  } as const;

  const statusLabels = {
      good: 'STABLE',
      warning: 'WARNING',
      critical: 'CRITICAL'
  };
</script>

<div class="space-y-3">
  {#if assessments.length === 0}
    <p class="rounded-lg border border-white/5 bg-black/20 p-4 text-xs font-mono text-slate-400">
      NO HEALTH ASSESSMENTS AVAILABLE.
    </p>
  {:else}
    {#each assessments as item}
      <article class={`rounded-xl border px-4 py-3 backdrop-blur-sm transition-all duration-300 ${badgeClass[item.severity]}`}>
        <header class="flex items-start justify-between gap-3">
          <h3 class="text-sm font-bold uppercase tracking-wider">{item.title}</h3>
          <span class="rounded px-1.5 py-0.5 text-[10px] font-mono border border-current opacity-80">
            {statusLabels[item.severity]}
          </span>
        </header>
        <p class="mt-2 text-xs opacity-90 font-mono">{item.detail}</p>
        {#if item.hint}
          <div class="mt-2 pt-2 border-t border-dashed border-current/20 flex items-start gap-2">
            <span class="text-[10px] uppercase font-bold opacity-70">Hint:</span>
            <p class="text-[10px] opacity-80">{item.hint}</p>
          </div>
        {/if}
      </article>
    {/each}
  {/if}
</div>
