<script>
  import '../app.css';
  import AppErrorBoundary from '$lib/components/AppErrorBoundary.svelte';
  import ErrorOverlay from '$lib/components/ErrorOverlay.svelte';
  import SecurityBadge from '$lib/components/SecurityBadge.svelte';
  import ToastContainer from '$lib/components/ToastContainer.svelte';
</script>

<main>
  <SecurityBadge />
  <AppErrorBoundary>
    <slot />
  </AppErrorBoundary>
  <ErrorOverlay />
  <ToastContainer />
</main>
