<script lang="ts">
  import AppErrorBoundary from '../lib/components/AppErrorBoundary.svelte';
  import FaultyComponent from './FaultyComponent.svelte';
</script>

<AppErrorBoundary>
  <FaultyComponent trigger={true} />
</AppErrorBoundary>
