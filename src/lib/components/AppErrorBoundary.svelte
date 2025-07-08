<script lang="ts">
import ErrorBoundary from 'svelte-error-boundary';
import { errorStore } from '$lib/stores/errorStore';

function handleError(err: Error) {
  errorStore.set(err);
}
</script>

<ErrorBoundary name="app" {handleError}>
  <slot />
</ErrorBoundary>
