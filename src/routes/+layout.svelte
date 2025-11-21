<script>
  import '../app.css';
  import AppErrorBoundary from '$lib/components/AppErrorBoundary.svelte';
  import ErrorOverlay from '$lib/components/ErrorOverlay.svelte';
  import SecurityBadge from '$lib/components/SecurityBadge.svelte';
  import ToastContainer from '$lib/components/ToastContainer.svelte';
  import { fade } from 'svelte/transition';
  import { page } from '$app/stores';
</script>

<div class="relative min-h-screen w-full overflow-hidden selection:bg-neon-green selection:text-black">

  <!-- Global Security Overlay (Cyber Grid) -->
  <div class="fixed inset-0 pointer-events-none z-0 opacity-[0.03]"
       style="background-image: linear-gradient(#333 1px, transparent 1px), linear-gradient(90deg, #333 1px, transparent 1px); background-size: 40px 40px;">
  </div>

  <!-- Main Content Area with Transition -->
  <main class="relative z-10 flex flex-col min-h-screen">
    <div class="fixed top-4 right-4 z-50">
      <SecurityBadge />
    </div>

    <AppErrorBoundary>
      {#key $page.url.pathname}
        <div
          in:fade={{ duration: 300, delay: 150 }}
          out:fade={{ duration: 150 }}
          class="flex-1 flex flex-col"
        >
          <slot />
        </div>
      {/key}
    </AppErrorBoundary>

    <ErrorOverlay />
    <ToastContainer />
  </main>

  <!-- Vignette -->
  <div class="fixed inset-0 pointer-events-none z-20 bg-radial-gradient-void"></div>
</div>

<style>
  .bg-radial-gradient-void {
    background: radial-gradient(circle at center, transparent 0%, #050507 120%);
  }
</style>
