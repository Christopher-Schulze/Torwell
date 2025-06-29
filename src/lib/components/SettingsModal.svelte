<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import { X, Edit3 } from 'lucide-svelte';
	// import TorrcEditorModal from './TorrcEditorModal.svelte';
	
	
	export let show: boolean;
	
	const dispatch = createEventDispatcher();
	let showTorrcEditor = false; // This will be unused for now

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
					<div class="mb-8">
						<h3 class="text-lg font-semibold mb-4 border-b border-white/10 pb-2">Tor Chain Configuration</h3>
						<p class="text-sm text-gray-400 mb-4">Edit the TORRC file to customize Tor settings and connection behavior.</p>
						<button 
							class="text-sm py-2 px-4 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
							on:click={() => showTorrcEditor = true}
						>
							<Edit3 size={16} />
							Editor
						</button>
						<p class="text-xs text-gray-500 mt-2">Edit the TORRC file</p>
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