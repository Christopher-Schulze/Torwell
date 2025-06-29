<script>
	import { Activity, Settings, Play, Square, RotateCcw, RefreshCw, AlertCircle } from 'lucide-svelte';
	import { createEventDispatcher } from 'svelte';
	
	import { torStore } from '$lib/stores/torStore';
	import { invoke } from '@tauri-apps/api';

	let connectError = null;

	const dispatch = createEventDispatcher();

	let isCreatingCircuit = false;
	let isCreatingIdentity = false;

	async function handleConnect() {
		await invoke('connect');
	}

	async function handleDisconnect() {
		await invoke('disconnect');
	}

	async function handleNewCircuit() {
		if (!isConnected || isCreatingCircuit) return;
		
		isCreatingCircuit = true;
		try {
			await invoke('new_identity');
			console.log('New circuit requested successfully');
		} catch (error) {
			console.error('New circuit failed:', error);
		} finally {
			isCreatingCircuit = false;
		}
	}

	async function handleNewIdentity() {
		if (isCreatingIdentity) return;
		isCreatingIdentity = true;
		try {
			await invoke('new_identity');
			console.log('New identity request sent successfully');
		} catch (error) {
			console.error('New identity failed:', error);
		} finally {
			isCreatingIdentity = false;
		}
	}

	$: isConnected = $torStore.status === 'CONNECTED';
	$: isStopped = $torStore.status === 'DISCONNECTED';
	$: isConnecting = $torStore.status === 'CONNECTING';
	$: isDisconnecting = $torStore.status === 'DISCONNECTING';

</script>

<div class="bg-black/20 rounded-xl p-6">
	<!-- Error Message -->
	{#if $torStore.errorMessage}
		<div class="mb-4 p-3 bg-red-900/30 border border-red-700/50 text-red-300 rounded-lg flex items-center gap-2">
			<AlertCircle size={16} />
			<span>{$torStore.errorMessage}</span>
		</div>
	{/if}

	<!-- Four Buttons Layout -->
	<div class="grid grid-cols-4 gap-3">
		<!-- Connect/Disconnect Button -->
		{#if isStopped}
			<button
				class="py-3 px-4 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all duration-300 ease-in-out text-sm bg-green-600/20 text-green-400 hover:bg-green-600/30 border border-green-500/30 transform hover:scale-105"
				on:click={handleConnect}
			>
				<Play size={16} /> Connect
			</button>
		{:else if isConnecting}
			<button
				class="py-3 px-4 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 transition-all duration-300 ease-in-out text-sm bg-yellow-600/20 text-yellow-400 border border-yellow-500/30 opacity-75 cursor-not-allowed"
				disabled={true}
			>
				<div class="animate-spin"><RefreshCw size={16} /></div>
				Connecting...
			</button>
		{:else if isConnected}
			<button
				class="py-3 px-4 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all duration-300 ease-in-out text-sm bg-red-600/20 text-red-400 hover:bg-red-600/30 border border-red-500/30 transform hover:scale-105"
				on:click={handleDisconnect}
			>
				<Square size={16} /> Disconnect
			</button>
		{:else if isDisconnecting}
			<button
				class="py-3 px-4 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 transition-all duration-300 ease-in-out text-sm bg-yellow-600/20 text-yellow-400 border border-yellow-500/30 opacity-75 cursor-not-allowed"
				disabled={true}
			>
				<div class="animate-spin"><RefreshCw size={16} /></div>
				Disconnecting...
			</button>
		{/if}
		
		<!-- New Circuit Button -->
		<button
			class="py-3 px-4 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 transition-all duration-300 ease-in-out text-sm {isConnected && !isCreatingCircuit ? 'bg-black/50 text-gray-300 hover:bg-black/60 cursor-pointer transform hover:scale-105' : 'bg-black/30 text-gray-500 cursor-not-allowed opacity-50'}"
			on:click={handleNewCircuit}
			disabled={!isConnected || isCreatingCircuit}
		>
			{#if isCreatingCircuit}
				<div class="animate-spin"><RefreshCw size={16} /></div>
				Creating...
			{:else}
				<RotateCcw size={16} /> New Circuit
			{/if}
		</button>
		
		<!-- Logs Button -->
		<button
			class="py-3 px-4 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all text-sm bg-black/50 text-gray-300 hover:bg-black/60"
			on:click={() => dispatch('openLogs')}
		>
			<Activity size={16} /> Logs
		</button>
		
		<!-- Settings Button -->
		<button
			class="py-3 px-4 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all text-sm bg-black/50 text-gray-300 hover:bg-black/60"
			on:click={() => dispatch('openSettings')}
		>
			<Settings size={16} /> Settings
		</button>
	</div>
</div>