<script context="module" lang="ts">
import { writable } from 'svelte/store';
export const errorStore = writable<Error | null>(null);
</script>

<script lang="ts">
import ErrorBoundary from 'svelte-error-boundary';

function handleError(err: Error) {
  errorStore.set(err);
}
</script>

<ErrorBoundary name="app" {handleError}>
  <slot />
</ErrorBoundary>
