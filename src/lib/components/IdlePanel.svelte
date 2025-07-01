<script>
        export let connectionProgress = 0; // 0-100
        export let currentStatus = 'Idle'; // Current Tor status
        export let retryCount = 0;
        export let retryDelay = 0;
        export let bootstrapMessage = '';

	// Animation for status text changes
	let isAnimating = false;
	let previousStatus = currentStatus;

	$: if (currentStatus !== previousStatus) {
		isAnimating = true;
		setTimeout(() => {
			previousStatus = currentStatus;
			isAnimating = false;
		}, 300);
	}
</script>

<div class="bg-black/20 rounded-xl p-6">
	<div class="flex flex-col items-center gap-3">
		<!-- Progress Bar -->
                <div class="w-full bg-gray-700/50 rounded-full h-2">
                        <div
                                class="bg-white h-2 rounded-full transition-all duration-500 ease-out"
                                style="width: {connectionProgress}%"
                        ></div>
                </div>
                <p class="text-xs text-gray-200">{Math.round(connectionProgress)}%</p>
                {#if bootstrapMessage}
                        <p class="text-xs text-gray-200 italic">{bootstrapMessage}</p>
                {/if}
		
		<!-- Animated Status Text -->
                <div class="text-center relative h-4 flex items-center justify-center">
                        <p
                                class="text-xs font-medium text-white absolute transition-all duration-300 {isAnimating ? 'opacity-0 transform scale-95' : 'opacity-100 transform scale-100'}"
                        >
                                {currentStatus}
                        </p>
                </div>
                {#if currentStatus === 'RETRYING'}
                        <p class="text-xs text-yellow-300">retry {retryCount} in {retryDelay}s</p>
                {/if}
        </div>
</div>
