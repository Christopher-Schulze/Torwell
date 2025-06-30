<script lang="ts">
        import { createEventDispatcher } from 'svelte';
        import { X, Edit3 } from 'lucide-svelte';
        import { uiStore } from '$lib/stores/uiStore';
        // import TorrcEditorModal from './TorrcEditorModal.svelte';
	
	
	export let show: boolean;
	
	const dispatch = createEventDispatcher();
        let showTorrcEditor = false; // This will be unused for now
        let torrcConfig = '';
        let workerListText = '';

        $: if (show) {
                torrcConfig = $uiStore.settings.torrcConfig;
                workerListText = $uiStore.settings.workerList.join('\n');
        }

        async function save() {
                const list = workerListText
                        .split('\n')
                        .map((l) => l.trim())
                        .filter((l) => l.length > 0);
                await uiStore.actions.saveWorkerList(list);
                await uiStore.actions.saveTorrcConfig(torrcConfig);
                show = false;
                dispatch('close');
        }

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			show = false;
			dispatch('close');
		}
	}
</script>

<svelte:window on:keydown={handleKeyDown} />

{#if show}
		<div
			class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
			on:click|stopPropagation={() => (show = false)}
			role="dialog"
			aria-modal="true"
			on:keydown={handleKeyDown}
		>
			<section
				class="bg-black/40 backdrop-blur-3xl rounded-2xl border border-white/10 w-[90%] max-w-2xl min-h-[500px] p-6 flex flex-col"
				on:click|stopPropagation
				role="document"
			>
				<div class="flex justify-between items-center mb-4 shrink-0">
					<h2 class="text-2xl font-semibold">Settings</h2>
					<button
						class="text-gray-400 hover:text-white transition-colors"
						on:click={() => (show = false)}
					>
						<X size={24} />
					</button>
				</div>
				<div class="overflow-y-auto flex-grow">
					<!-- Tor Chain Configuration -->
                                        <div class="mb-8 space-y-4">
                                                <h3 class="text-lg font-semibold mb-2 border-b border-white/10 pb-2">Tor Configuration</h3>
                                                <textarea
                                                        class="w-full p-2 rounded-md bg-black/30 border border-white/10 font-mono text-sm"
                                                        rows="6"
                                                        bind:value={torrcConfig}
                                                ></textarea>
                                                <p class="text-xs text-gray-500">Custom TOML configuration for Arti/Tor.</p>
                                        </div>

                                        <div class="mb-8 space-y-4">
                                                <h3 class="text-lg font-semibold mb-2 border-b border-white/10 pb-2">Worker List</h3>
                                                <textarea
                                                        class="w-full p-2 rounded-md bg-black/30 border border-white/10 font-mono text-sm"
                                                        rows="4"
                                                        bind:value={workerListText}
                                                ></textarea>
                                                <p class="text-xs text-gray-500">One host per line</p>
                                        </div>

                                        <div class="flex justify-end mt-4">
                                                <button
                                                        class="py-2 px-4 rounded-xl bg-blue-500/20 text-blue-400 hover:bg-blue-500/30 transition-colors"
                                                        on:click={save}
                                                >
                                                        Save
                                                </button>
                                        </div>
                                </div>
			</section>
		</div>
{/if}

<!-- TORRC Editor Modal -->
<!-- <TorrcEditorModal
	bind:show={showTorrcEditor}
	on:close={() => showTorrcEditor = false}
/> -->