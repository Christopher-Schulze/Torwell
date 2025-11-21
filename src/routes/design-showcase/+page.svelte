<script>
  import Motion from '$lib/motion/Motion.svelte';
  import CyberButton from '$lib/components/CyberButton.svelte';
  import { onMount } from 'svelte';

  let mounted = false;
  onMount(() => mounted = true);
</script>

<div class="min-h-screen flex flex-col items-center justify-center p-10 gap-12">

  <!-- Header Section -->
  <Motion delay={200} yOffset={-50}>
    <div class="text-center space-y-4">
      <h1 class="text-6xl font-black tracking-tighter text-white uppercase relative inline-block">
        <span class="cyber-text-glow">Torwell</span><span class="text-neon-green">.84</span>
      </h1>
      <p class="text-gray-400 font-mono text-sm tracking-[0.5em] uppercase">
        Secure . Private . Invisible
      </p>
    </div>
  </Motion>

  <!-- Interactive Elements Grid -->
  <div class="grid grid-cols-1 md:grid-cols-3 gap-8 w-full max-w-5xl">

    <!-- Card 1: Status -->
    <Motion delay={400}>
      <div class="cyber-glass p-8 rounded-2xl h-full flex flex-col justify-between group hover:border-neon-green/30 transition-colors duration-500">
        <div>
          <div class="flex justify-between items-start mb-4">
            <span class="text-xs font-mono text-neon-green border border-neon-green/20 px-2 py-1 rounded">SYS.ONLINE</span>
            <div class="w-2 h-2 rounded-full bg-neon-green shadow-[0_0_10px_#00ff41] animate-pulse"></div>
          </div>
          <h3 class="text-2xl font-bold text-white mb-2">Tor Connection</h3>
          <p class="text-gray-400 text-sm leading-relaxed">
            Routed through 3 nodes. Traffic is encrypted and anonymous.
          </p>
        </div>
        <div class="mt-6">
           <div class="h-1 w-full bg-white/5 rounded-full overflow-hidden">
             <div class="h-full bg-neon-green w-[84%] shadow-[0_0_10px_#00ff41]"></div>
           </div>
           <div class="flex justify-between text-xs font-mono text-gray-500 mt-2">
             <span>SIGNAL</span>
             <span>84%</span>
           </div>
        </div>
      </div>
    </Motion>

    <!-- Card 2: Controls -->
    <Motion delay={500}>
      <div class="cyber-glass-high p-8 rounded-2xl h-full flex flex-col items-center justify-center gap-6 text-center border-t-4 border-t-neon-purple">
        <div class="text-neon-purple mb-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>
        </div>
        <h3 class="text-xl font-bold text-white">Identity Control</h3>
        <div class="flex flex-col gap-3 w-full">
          <CyberButton variant="primary" onClick={() => console.log('New Circuit')}>
            New Identity
          </CyberButton>
          <CyberButton variant="secondary" onClick={() => console.log('Settings')}>
            Configure
          </CyberButton>
        </div>
      </div>
    </Motion>

    <!-- Card 3: Logs -->
    <Motion delay={600}>
      <div class="cyber-glass p-1 rounded-2xl h-full relative overflow-hidden">
        <div class="absolute inset-0 bg-black/40 z-0"></div>
        <div class="relative z-10 p-6 font-mono text-xs h-full flex flex-col">
          <div class="mb-4 text-gray-500 border-b border-white/10 pb-2">SYSTEM LOGS</div>
          <div class="space-y-2 overflow-hidden text-gray-300">
            <div class="flex gap-2"><span class="text-neon-green">➜</span> <span>Init handshake...</span></div>
            <div class="flex gap-2"><span class="text-neon-green">➜</span> <span>Loading circuits...</span></div>
            <div class="flex gap-2"><span class="text-neon-green">➜</span> <span>Guard node verified</span></div>
            <div class="flex gap-2"><span class="text-neon-green">➜</span> <span>Relay connected</span></div>
            <div class="flex gap-2"><span class="text-neon-green">➜</span> <span class="text-neon-purple">Circuit built (240ms)</span></div>
          </div>
        </div>
      </div>
    </Motion>

  </div>

  <Motion delay={800}>
    <CyberButton variant="danger">Disconnect</CyberButton>
  </Motion>

</div>
