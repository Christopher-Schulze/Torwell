<script lang="ts">
        import '../app.css';
        import ErrorDisplay from '$lib/components/ErrorDisplay.svelte';
        import { uiStore } from '$lib/stores/uiStore';
        import { onMount } from 'svelte';

        onMount(() => {
                const handler = (e: ErrorEvent) => uiStore.actions.setError(e.message);
                const rej = (e: PromiseRejectionEvent) => uiStore.actions.setError(e.reason?.message ?? String(e.reason));
                window.addEventListener('error', handler);
                window.addEventListener('unhandledrejection', rej);
                return () => {
                        window.removeEventListener('error', handler);
                        window.removeEventListener('unhandledrejection', rej);
                };
        });
</script>

<main>
        <ErrorDisplay />
        <slot />
</main>