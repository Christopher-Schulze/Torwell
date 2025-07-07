<script lang="ts">
  import { createEventDispatcher, tick } from 'svelte';
  import { X } from 'lucide-svelte';
  import { uiStore } from '$lib/stores/uiStore';

  export let show = false;

  const dispatch = createEventDispatcher();

  let torrc = '';
  let closeButton: HTMLButtonElement | null = null;
  let modalEl: HTMLElement | null = null;
  let previouslyFocused: HTMLElement | null = null;

  $: if (show) {
    previouslyFocused = document.activeElement as HTMLElement;
    torrc = $uiStore.settings.torrcConfig;
    tick().then(() => closeButton && closeButton.focus());
  } else if (previouslyFocused) {
    tick().then(() => previouslyFocused && previouslyFocused.focus());
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      dispatch('close');
    }
  }

  function trapFocus(event: KeyboardEvent) {
    if (event.key !== 'Tab' || !modalEl) return;
    const focusable = Array.from(
      modalEl.querySelectorAll<HTMLElement>('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])')
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function save() {
    uiStore.actions.saveTorrcConfig(torrc);
    dispatch('close');
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

{#if show}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    on:click={() => dispatch('close')}
    tabindex="-1"
  >
    <section
      class="glass-lg rounded-2xl w-[90%] max-w-2xl p-6 flex flex-col"
      on:click|stopPropagation
      on:keydown={trapFocus}
      bind:this={modalEl}
      role="dialog"
      aria-modal="true"
      aria-labelledby="torrc-editor-title"
      tabindex="0"
    >
      <div class="flex justify-between items-center mb-4 shrink-0">
        <h2 id="torrc-editor-title" class="text-2xl font-semibold">Edit torrc</h2>
        <button
          class="text-gray-100 hover:text-white transition-colors"
          on:click={() => dispatch('close')}
          aria-label="Close torrc editor"
          bind:this={closeButton}
        >
          <X size={24} />
        </button>
      </div>
      <textarea
        class="w-full bg-black/50 rounded border border-white/20 p-2 text-sm font-mono flex-grow"
        rows="10"
        bind:value={torrc}
        aria-label="Torrc configuration"
      ></textarea>
      <button
        class="text-sm py-2 px-4 mt-4 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
        on:click={save}
        aria-label="Save torrc configuration"
      >
        Save
      </button>
    </section>
  </div>
{/if}
