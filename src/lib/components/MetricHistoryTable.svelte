<script lang="ts">
  import type { MetricPoint } from '$lib/stores/torStore';
  import { formatRelativeTime, type Severity } from '$lib/utils/metrics';

  export let metrics: MetricPoint[] = [];
  export let maxRows = 20;

  $: rows = [...metrics].slice(-maxRows).reverse();

  function severityForRow(row: MetricPoint): Severity {
    if (row.failedAttempts >= 5 || row.latencyMs >= 600 || row.memoryMB >= 1100) {
      return 'critical';
    }
    if (row.failedAttempts >= 2 || row.latencyMs >= 300 || row.memoryMB >= 900) {
      return 'warning';
    }
    return 'good';
  }

  function rowClass(row: MetricPoint): string {
    const severity = severityForRow(row);
    if (severity === 'critical') return 'bg-rose-500/10 border-rose-400/30';
    if (severity === 'warning') return 'bg-amber-500/10 border-amber-400/30';
    return 'bg-slate-900/20 border-slate-700/40';
  }
</script>

<div class="rounded-xl border border-slate-700/60 bg-slate-900/40">
  <div class="flex items-center justify-between border-b border-slate-700/60 px-4 py-3">
    <h3 class="text-sm font-semibold text-white">Historie</h3>
    <span class="text-xs text-slate-300">Letzte {rows.length} Einträge</span>
  </div>
  <div class="overflow-x-auto">
    <table class="min-w-full divide-y divide-slate-700/60 text-left text-sm text-slate-200">
      <thead class="bg-slate-900/60 text-xs uppercase text-slate-300">
        <tr>
          <th scope="col" class="px-4 py-3 font-medium">Zeit</th>
          <th scope="col" class="px-4 py-3 font-medium">Speicher</th>
          <th scope="col" class="px-4 py-3 font-medium">Circuits</th>
          <th scope="col" class="px-4 py-3 font-medium">Latenz</th>
          <th scope="col" class="px-4 py-3 font-medium">Durchsatz</th>
          <th scope="col" class="px-4 py-3 font-medium">Fehler</th>
          <th scope="col" class="px-4 py-3 font-medium">Build Ø</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-slate-800/50">
        {#if rows.length === 0}
          <tr>
            <td class="px-4 py-4 text-center text-slate-400" colspan="7">Keine historischen Daten verfügbar.</td>
          </tr>
        {:else}
          {#each rows as row}
            <tr class={`border-l-2 ${rowClass(row)}`}>
              <td class="px-4 py-3 text-slate-200">{formatRelativeTime(row.time)}</td>
              <td class="px-4 py-3">
                <span class="font-medium text-white">{row.memoryMB} MB</span>
              </td>
              <td class="px-4 py-3">{row.circuitCount}</td>
              <td class="px-4 py-3">{row.latencyMs} ms</td>
              <td class="px-4 py-3">{row.networkBytes} B/s</td>
              <td class="px-4 py-3">{row.failedAttempts}</td>
              <td class="px-4 py-3">{row.avgCreateMs} ms</td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</div>
