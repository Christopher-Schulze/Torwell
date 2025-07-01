<script lang="ts">

        // Props
        export let isConnected = false;
	export let entryCountry = 'Germany';
	export let middleCountry = 'Germany';
	export let exitCountry = 'Germany';
	export let isActive = true;
	export let cloudflareEnabled = false;

	// Node data with IPs and names
	export let nodeData: { nickname: string; ip_address: string; country: string }[] = [];

        const getCountryFlag = (countryCode: string) => {
        if (!countryCode || countryCode === '??' || countryCode === 'XX') return '🏳️';
                const codePoints = countryCode
                        .toUpperCase()
                        .split('')
                        .map(char =>  127397 + char.charCodeAt(0));
                return String.fromCodePoint(...codePoints);
	};

        const countries = ['Germany','France','Belgium','Switzerland','Liechtenstein','Luxembourg','Austria','Spain','Italy','Portugal','Russia','Romania','Turkey','UK','USA','Canada','Mexico','Brazil','Argentina','Japan','China','Antarctica'];

        import { uiStore } from '$lib/stores/uiStore';

        const exitCountryOptions = [
                { code: 'DE', name: 'Germany' },
                { code: 'FR', name: 'France' },
                { code: 'BE', name: 'Belgium' },
                { code: 'CH', name: 'Switzerland' },
                { code: 'AT', name: 'Austria' },
                { code: 'ES', name: 'Spain' },
                { code: 'IT', name: 'Italy' },
                { code: 'PT', name: 'Portugal' },
                { code: 'RU', name: 'Russia' },
                { code: 'RO', name: 'Romania' },
                { code: 'TR', name: 'Turkey' },
                { code: 'GB', name: 'UK' },
                { code: 'US', name: 'USA' },
                { code: 'CA', name: 'Canada' },
                { code: 'MX', name: 'Mexico' },
                { code: 'BR', name: 'Brazil' },
                { code: 'AR', name: 'Argentina' },
                { code: 'JP', name: 'Japan' },
                { code: 'CN', name: 'China' },
        ];

        let selectedExitCountry: string | null = null;
        $: selectedExitCountry = $uiStore.settings.exitCountry;

        function changeExitCountry(event: Event) {
                const value = (event.target as HTMLSelectElement).value;
                uiStore.actions.setExitCountry(value || null);
        }

	function handleCountryChange(nodeType: string, event: Event) {
		const target = event.target as HTMLSelectElement;
		const newCountry = target.value;
		
		if (nodeType === 'entry') {
			entryCountry = newCountry;
		} else if (nodeType === 'middle') {
			middleCountry = newCountry;
		} else if (nodeType === 'exit') {
			exitCountry = newCountry;
		}
	}
</script>

<div class="bg-black/20 rounded-xl p-6">
	<!-- Single Row with Title, Dropdowns and Active Switch -->
	<div class="grid grid-cols-5 gap-4 mb-4 items-center h-8">
		<!-- Tor Chain Title -->
		<div class="flex items-center h-8">
			<h3 class="text-sm font-semibold text-white">Chain of Nodes</h3>
        </div>

		
		<!-- Entry Node Dropdown -->
		<div class="flex items-center h-8">
			<div class="relative w-full h-8">
                                <select
                                        class="w-full h-8 bg-black/50 border border-white/20 rounded-lg px-2 py-1 text-xs text-white focus:outline-none focus:border-white/40 hover:bg-black/60 transition-all appearance-none cursor-pointer"
                                        value={entryCountry}
                                        aria-label="Entry node country"
                                        on:change={(e) => handleCountryChange('entry', e)}
                                >
					{#each countries as country}
						<option value={country} class="bg-gray-800 text-xs">
							{getCountryFlag(country)} {country}
						</option>
					{/each}
				</select>
				<div class="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
					<svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
					</svg>
				</div>
			</div>
		</div>
		
		<!-- Middle Node Dropdown -->
		<div class="flex items-center h-8">
			<div class="relative w-full h-8">
                                <select
                                        class="w-full h-8 bg-black/50 border border-white/20 rounded-lg px-2 py-1 text-xs text-white focus:outline-none focus:border-white/40 hover:bg-black/60 transition-all appearance-none cursor-pointer"
                                        value={middleCountry}
                                        aria-label="Middle node country"
                                        on:change={(e) => handleCountryChange('middle', e)}
                                >
					{#each countries as country}
						<option value={country} class="bg-gray-800 text-xs">
							{getCountryFlag(country)} {country}
						</option>
					{/each}
				</select>
				<div class="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
					<svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
					</svg>
				</div>
			</div>
		</div>
		
		<!-- Exit Node Dropdown -->
		<div class="flex items-center h-8">
			<div class="relative w-full h-8">
                                <select
                                        class="w-full h-8 bg-black/50 border border-white/20 rounded-lg px-2 py-1 text-xs text-white focus:outline-none focus:border-white/40 hover:bg-black/60 transition-all appearance-none cursor-pointer"
                                        value={exitCountry}
                                        aria-label="Exit node country"
                                        on:change={(e) => handleCountryChange('exit', e)}
                                >
					{#each countries as country}
						<option value={country} class="bg-gray-800 text-xs">
							{getCountryFlag(country)} {country}
						</option>
					{/each}
				</select>
				<div class="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
					<svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
					</svg>
				</div>
			</div>
		</div>
		
		<!-- Active Switch -->
		<div class="flex items-center justify-end h-8">
			<div class="relative w-full h-8">
				<div class="w-full h-8 bg-black/50 border border-white/20 rounded-lg px-2 py-1 hover:bg-black/60 transition-all flex items-center">
					<label class="flex items-center justify-between cursor-pointer w-full">
						<span class="text-xs text-white font-medium">Active</span>
                                                <input
                                                        type="checkbox"
                                                        bind:checked={isActive}
                                                        class="sr-only"
                                                        aria-label="Toggle chain active"
                                                />
						<div class="relative w-8 h-4 bg-gray-600 rounded-full transition-colors {isActive ? 'bg-green-500' : 'bg-gray-600'}">
							<div class="absolute top-0.5 left-0.5 w-3 h-3 bg-white rounded-full transition-transform {isActive ? 'translate-x-4' : 'translate-x-0'}"></div>
						</div>
					</label>
				</div>
			</div>
		</div>
        </div>

        <!-- Exit Country Selection -->
        <div class="flex items-center mb-4">
                <label class="text-xs text-white mr-2" for="exit-country">Exit Country:</label>
                <div class="relative w-48 h-8">
                        <select
                                id="exit-country"
                                class="w-full h-8 bg-black/50 border border-white/20 rounded-lg px-2 py-1 text-xs text-white focus:outline-none focus:border-white/40 hover:bg-black/60 transition-all appearance-none cursor-pointer"
                                bind:value={selectedExitCountry}
                                aria-label="Preferred exit country"
                                on:change={changeExitCountry}
                        >
                                <option value="">Auto</option>
                                {#each exitCountryOptions as opt}
                                        <option value={opt.code}>{getCountryFlag(opt.code)} {opt.name}</option>
                                {/each}
                        </select>
                        <div class="absolute inset-y-0 right-0 flex items-center pr-2 pointer-events-none">
                                <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                                </svg>
                        </div>
                </div>
        </div>

        <!-- Node Cards Row -->
	<div class="grid grid-cols-5 gap-4 mt-8">

		<!-- You Card -->
		<div class="bg-black/50 rounded-xl p-4 flex flex-col min-h-[200px]">
			<div class="flex justify-center items-center h-12 mb-2">
				<div class="text-3xl">🌐</div>
			</div>
			<div class="text-center flex-1 flex flex-col justify-center space-y-1">
				<div class="text-sm text-white font-medium h-5 flex items-center justify-center">You</div>
				<div class="text-xs text-gray-200 h-4 flex items-center justify-center">
                                        {#if isConnected}
                                                {nodeData[0]?.ip_address || '-'}
                                        {:else}
                                                -
                                        {/if}
				</div>
				<div class="text-lg h-6 flex items-center justify-center">
					{#if isConnected}
					    🏠
				    {:else}
					-
				    {/if}
				</div>
				<div class="text-xs text-gray-200 h-4 flex items-center justify-center">
					-
				</div>
			</div>
		</div>
		
		<!-- Entry Node Card -->
		<div class="bg-black/50 rounded-xl p-4 flex flex-col min-h-[200px]">
			<div class="flex justify-center items-center h-12 mb-2">
				<div class="text-3xl">🔒</div>
			</div>
			<div class="text-center flex-1 flex flex-col justify-center space-y-1">
				<div class="text-sm text-white font-medium h-5 flex items-center justify-center">Entry Node</div>
				<div class="text-xs text-gray-200 h-4 flex items-center justify-center">
					{#if isConnected && nodeData[0]}
						{nodeData[0].ip_address}
					{:else}
						-
					{/if}
				</div>
				<div class="text-lg h-6 flex items-center justify-center">
					{#if isConnected && nodeData[0]}
					    {getCountryFlag(nodeData[0].country)}
				    {:else}
					-
				    {/if}
				</div>
				<div class="text-xs text-gray-200 h-4 flex items-center justify-center">
					{#if isConnected && nodeData[0]}
						{nodeData[0].nickname}
					{:else}
						-
					{/if}
				</div>
			</div>
		</div>
		
		<!-- Middle Node Card -->
		<div class="bg-black/50 rounded-xl p-4 flex flex-col min-h-[200px]">
			<div class="flex justify-center items-center h-12 mb-2">
				<div class="text-3xl">🔒</div>
			</div>
			<div class="text-center flex-1 flex flex-col justify-center space-y-1">
				<div class="text-sm text-white font-medium h-5 flex items-center justify-center">Middle Node</div>
				<div class="text-xs text-gray-200 h-4 flex items-center justify-center">
					{#if isConnected && nodeData[1]}
						{nodeData[1].ip_address}
					{:else}
						-
					{/if}
				</div>
				<div class="text-lg h-6 flex items-center justify-center">
					{#if isConnected && nodeData[1]}
					    {getCountryFlag(nodeData[1].country)}
				    {:else}
					-
				    {/if}
				</div>
				<div class="text-xs text-gray-200 h-4 flex items-center justify-center">
					{#if isConnected && nodeData[1]}
						{nodeData[1].nickname}
					{:else}
						-
					{/if}
				</div>
			</div>
		</div>
		
		<!-- Exit Node Card -->
		<div class="bg-black/50 rounded-xl p-4 flex flex-col min-h-[200px]">
			<div class="flex justify-center items-center h-12 mb-2">
				<div class="text-3xl">🔓</div>
			</div>
			<div class="text-center flex-1 flex flex-col justify-center space-y-1">
				<div class="text-sm text-white font-medium h-5 flex items-center justify-center">Exit Node</div>
				<div class="text-xs text-gray-200 h-4 flex items-center justify-center">
					{#if isConnected && nodeData[2]}
						{nodeData[2].ip_address}
					{:else}
						-
					{/if}
				</div>
				<div class="text-lg h-6 flex items-center justify-center">
					{#if isConnected && nodeData[2]}
					    {getCountryFlag(nodeData[2].country)}
				    {:else}
					-
				    {/if}
				</div>
				<div class="text-xs text-gray-200 h-4 flex items-center justify-center">
					{#if isConnected && nodeData[2]}
						{nodeData[2].nickname}
					{:else}
						-
					{/if}
				</div>
			</div>
		</div>
		
		<!-- Cloudflare Card -->
		<div class="bg-black/50 rounded-xl p-4 flex flex-col min-h-[200px] {cloudflareEnabled ? '' : 'opacity-50'}">
			<div class="flex justify-center items-center h-12 mb-2">
				<div class="text-3xl {cloudflareEnabled ? '' : 'opacity-50'}">☁️</div>
			</div>
			<div class="text-center flex-1 flex flex-col justify-center space-y-1">
				<div class="text-sm {cloudflareEnabled ? 'text-white' : 'text-gray-300'} font-medium h-5 flex items-center justify-center">Cloudflare</div>
				<div class="text-xs {cloudflareEnabled ? 'text-gray-200' : 'text-gray-400'} h-4 flex items-center justify-center">
					-
				</div>
				<div class="text-lg h-6 flex items-center justify-center">
					-
				</div>
				<div class="text-xs text-gray-200 h-4 flex items-center justify-center">
					-
				</div>
			</div>
		</div>
	</div>
</div>