<script lang="ts">
        import { createEventDispatcher, tick } from 'svelte';
        import { X, Edit3 } from 'lucide-svelte';
        import { uiStore } from '$lib/stores/uiStore';

        const availableBridges = [
                'Bridge obfs4 192.0.2.1:443 0123456789ABCDEF0123456789ABCDEF01234567 cert=AAAA iat-mode=0',
                'Bridge obfs4 192.0.2.2:443 89ABCDEF0123456789ABCDEF0123456789ABCDEF cert=BBBB iat-mode=0'
        ];

        let selectedBridges: string[] = [];
        let torrcConfig = '';
        let workerListString = '';
        let maxLogLines = 1000;
        // import TorrcEditorModal from './TorrcEditorModal.svelte';
	
	
export let show: boolean;

const dispatch = createEventDispatcher();
let showTorrcEditor = false; // This will be unused for now
let closeButton: HTMLButtonElement | null = null;

        $: if (show) {
                selectedBridges = [...$uiStore.settings.bridges];
                torrcConfig = $uiStore.settings.torrcConfig;
                workerListString = $uiStore.settings.workerList.join('\n');
                maxLogLines = $uiStore.settings.maxLogLines;
                tick().then(() => closeButton && closeButton.focus());
        }

        function handleKeyDown(event: KeyboardEvent) {
                if (event.key === 'Escape') {
                        show = false;
                        dispatch('close');
                }
        }

        function saveTorrc() {
                uiStore.actions.saveTorrcConfig(torrcConfig);
        }

        function saveWorkers() {
                const list = workerListString
                        .split(/\r?\n/)
                        .map((l) => l.trim())
                        .filter((l) => l.length > 0);
                uiStore.actions.saveWorkerList(list);
        }

        function saveLogLimit() {
                const limit = parseInt(String(maxLogLines));
                if (!isNaN(limit) && limit > 0) {
                        uiStore.actions.setLogLimit(limit);
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
                        aria-labelledby="settings-modal-title"
                        on:keydown={handleKeyDown}
                >
			<section
				class="bg-black/40 backdrop-blur-3xl rounded-2xl border border-white/10 w-[90%] max-w-2xl min-h-[500px] p-6 flex flex-col"
				on:click|stopPropagation
				role="document"
			>
                                <div class="flex justify-between items-center mb-4 shrink-0">
                                        <h2 id="settings-modal-title" class="text-2xl font-semibold">Settings</h2>
                                        <button
                                                class="text-gray-200 hover:text-white transition-colors"
                                                on:click={() => (show = false)}
                                                aria-label="Close settings"
                                                bind:this={closeButton}
                                        >
						<X size={24} />
					</button>
				</div>
				<div class="overflow-y-auto flex-grow">
                                        <!-- Torrc Configuration -->
                                        <div class="mb-8">
                                                <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">Torrc Configuration</h3>
                                                <textarea
                                                        class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm font-mono"
                                                        rows="6"
                                                        bind:value={torrcConfig}
                                                ></textarea>
                                                <button
                                                        class="text-sm py-2 px-4 mt-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
                                                        on:click={saveTorrc}
                                                        aria-label="Save torrc configuration"
                                                >
                                                        Save
                                                </button>
                                        </div>

                                        <div class="mb-8">
                                                <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">Bridges</h3>
                                                <p class="text-sm text-gray-200 mb-4">Select one or more bridges to use for connecting.</p>
                                                {#each availableBridges as bridge}
                                                        <label class="flex items-center gap-2 mb-2">
                                                                <input type="checkbox" value={bridge} bind:group={selectedBridges} />
                                                                <span class="text-sm">{bridge}</span>
                                                        </label>
                                                {/each}
                                                <button
                                                        class="text-sm py-2 px-4 mt-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
                                                        on:click={() => uiStore.actions.setBridges(selectedBridges)}
                                                        aria-label="Apply bridge selection"
                                                >
                                                        Apply
                                                </button>
                                        </div>

                                        <div class="mb-8">
                                                <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">Worker List</h3>
                                                <textarea
                                                        class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm font-mono"
                                                        rows="4"
                                                        bind:value={workerListString}
                                                ></textarea>
                                                <button
                                                        class="text-sm py-2 px-4 mt-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
                                                        on:click={saveWorkers}
                                                        aria-label="Save worker list"
                                                >
                                                        Save
                                                </button>
                                                <p class="text-xs text-gray-300 mt-2">One worker URL per line</p>
                                        </div>

                                        <div class="mb-8">
                                                <h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">Max Log Lines</h3>
                                                <input type="number" min="1" class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm" bind:value={maxLogLines} />
                                                <button
                                                        class="text-sm py-2 px-4 mt-2 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
                                                        on:click={saveLogLimit}
                                                        aria-label="Save log limit"
                                                >
                                                        Save
                                                </button>
                                        </div>

					<!-- Worker Management section has been removed as it was placeholder functionality. -->
				</div>
			</section>
		</div>
{/if}

<!-- TORRC Editor Modal -->
<!-- <TorrcEditorModal
	bind:show={showTorrcEditor}
	on:close={() => showTorrcEditor = false}
/> -->