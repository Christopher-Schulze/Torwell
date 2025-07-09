<script lang="ts">
  export let connectionProgress = 0; // 0-100
  export let currentStatus = "Idle"; // Current Tor status
  export let retryCount = 0;
  export let retryDelay = 0;
  export let bootstrapMessage = "";
  export let errorStep: string | null = null;
  export let errorSource: string | null = null;

  const statusMap: Record<string, string> = {
    DISCONNECTED: "Disconnected",
    CONNECTING: "Connecting",
    RETRYING: "Retrying",
    CONNECTED: "Connected",
    DISCONNECTING: "Disconnecting",
    ERROR: "Error",
  };

  $: statusText = statusMap[currentStatus] ?? currentStatus;

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

<div
  class="glass-md rounded-xl p-6"
  role="region"
  aria-label="Connection progress"
>
  <div class="flex flex-col items-center gap-3">
    <!-- Progress Bar -->
    <div
      class="w-full bg-gray-700/50 rounded-full h-2"
      role="progressbar"
      aria-valuemin="0"
      aria-valuemax="100"
      aria-valuenow={connectionProgress}
    >
      <div
        class="bg-white h-2 rounded-full transition-all duration-500 ease-out"
        style="width: {connectionProgress}%"
      ></div>
    </div>
    <p class="text-xs text-gray-100">{Math.round(connectionProgress)}%</p>
    {#if bootstrapMessage}
      <p class="text-xs text-gray-100 italic">{bootstrapMessage}</p>
    {/if}

    <!-- Animated Status Text -->
    <div
      class="text-center relative h-4 flex items-center justify-center"
      aria-live="polite"
    >
      <p
        class="text-xs font-medium text-white absolute transition-all duration-300 {isAnimating
          ? 'opacity-0 transform scale-95'
          : 'opacity-100 transform scale-100'}"
      >
        {statusText}
      </p>
    </div>
    {#if currentStatus === "RETRYING"}
      <p class="text-xs text-yellow-300">retry {retryCount} in {retryDelay}s</p>
    {:else if currentStatus === "ERROR"}
      <p class="text-xs text-red-300">
        connection failed
        {#if errorStep}
          - {errorStep}: {errorSource}
        {/if}
      </p>
    {/if}
  </div>
</div>
