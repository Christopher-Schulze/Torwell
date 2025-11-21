<script lang="ts">
import { onMount } from 'svelte';
import { errorStore } from '$lib/stores/errorStore';

// Dynamically import ErrorBoundary to avoid SSR issues
let ErrorBoundary: any;
let mounted = false;

onMount(async () => {
  const module = await import('svelte-error-boundary');
  ErrorBoundary = module.default || module.ErrorBoundary; // Handle different export structures
  mounted = true;
});

function handleError(err: Error) {
  errorStore.set(err);
}
</script>

{#if mounted && ErrorBoundary}
  <svelte:component this={ErrorBoundary} name="app" {handleError}>
    <slot />
  </svelte:component>
{:else}
  <slot />
{/if}
