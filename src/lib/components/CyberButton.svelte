<script>
  import { spring } from 'svelte/motion';

  /** @type {'primary' | 'secondary' | 'danger'} */
  export let variant = 'primary';
  export let disabled = false;
  export let onClick = () => {};

  // Physics state
  const scale = spring(1, { stiffness: 0.2, damping: 0.4 });
  const shadow = spring(0, { stiffness: 0.1, damping: 0.5 });

  function handleMouseEnter() {
    if (disabled) return;
    scale.set(1.02);
    shadow.set(1);
  }

  function handleMouseLeave() {
    if (disabled) return;
    scale.set(1);
    shadow.set(0);
  }

  function handleMouseDown() {
    if (disabled) return;
    scale.set(0.95);
  }

  function handleMouseUp() {
    if (disabled) return;
    scale.set(1.02);
    onClick();
  }

  // Styles
  const baseStyles = "relative px-6 py-3 rounded-lg font-mono font-bold uppercase tracking-widest text-sm transition-colors duration-300 border overflow-hidden group";

  const variants = {
    primary: "bg-white/5 border-neon-green/30 text-neon-green hover:bg-neon-green/10 hover:border-neon-green",
    secondary: "bg-white/5 border-neon-purple/30 text-neon-purple hover:bg-neon-purple/10 hover:border-neon-purple",
    danger: "bg-white/5 border-red-500/30 text-red-500 hover:bg-red-500/10 hover:border-red-500"
  };

  // @ts-ignore
  $: selectedVariant = variants[variant] || variants.primary;
</script>

<button
  class="{baseStyles} {selectedVariant} {disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}"
  style="transform: scale({$scale}); box-shadow: 0 0 {$shadow * 20}px {$shadow * 5}px {variant === 'primary' ? 'rgba(0,255,65,0.2)' : 'rgba(188,19,254,0.2)'}"
  on:mouseenter={handleMouseEnter}
  on:mouseleave={handleMouseLeave}
  on:mousedown={handleMouseDown}
  on:mouseup={handleMouseUp}
  {disabled}
>
  <span class="relative z-10 flex items-center gap-2">
    <slot />
  </span>

  <!-- Scanline effect -->
  <div class="absolute inset-0 bg-gradient-to-b from-transparent via-white/5 to-transparent translate-y-[-100%] group-hover:translate-y-[100%] transition-transform duration-1000 pointer-events-none"></div>
</button>
