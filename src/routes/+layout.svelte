<script>
  import '../app.css';
  import AppErrorBoundary from '$lib/components/AppErrorBoundary.svelte';
  import ErrorOverlay from '$lib/components/ErrorOverlay.svelte';
</script>

<main>
  <AppErrorBoundary>
    <slot />
  </AppErrorBoundary>
  <ErrorOverlay />
</main>
