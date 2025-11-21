<script>
  import { onMount } from 'svelte';
  import { spring } from 'svelte/motion';

  // Props
  export let delay = 0;
  export let stiffness = 0.1;
  export let damping = 0.4;
  export let yOffset = 20;

  // Physics stores
  const y = spring(yOffset, { stiffness, damping });
  const opacity = spring(0, { stiffness, damping });

  onMount(() => {
    setTimeout(() => {
      y.set(0);
      opacity.set(1);
    }, delay);
  });
</script>

<div style="transform: translateY({$y}px); opacity: {$opacity}; will-change: transform, opacity;">
  <slot />
</div>
