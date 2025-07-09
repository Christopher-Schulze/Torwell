<script lang="ts">
  import { createEventDispatcher, tick } from "svelte";
  import { X } from "lucide-svelte";
  import { uiStore } from "$lib/stores/uiStore";

  export let show = false;

  const dispatch = createEventDispatcher();
  const docURL = new URL("../../docs/Todo-fuer-User.md", import.meta.url).href;

  let content = "";
  let filePicker: HTMLInputElement | null = null;
  let closeButton: HTMLButtonElement | null = null;
  let modalEl: HTMLElement | null = null;
  let previouslyFocused: HTMLElement | null = null;
  $: importProgress = $uiStore.importProgress;

  $: if (show) {
    previouslyFocused = document.activeElement as HTMLElement;
    fetch(docURL)
      .then((r) => r.text())
      .then((t) => (content = t));
    tick().then(() => closeButton && closeButton.focus());
  } else if (previouslyFocused) {
    tick().then(() => previouslyFocused && previouslyFocused.focus());
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      dispatch("close");
    }
  }

  function trapFocus(event: KeyboardEvent) {
    if (event.key !== "Tab" || !modalEl) return;
    const focusable = Array.from(
      modalEl.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
      ),
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

  async function importFile(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;
    const text = await input.files[0].text();
    await uiStore.actions.importWorkersFromText(text);
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

{#if show}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    role="button"
    tabindex="0"
    aria-label="Close worker setup"
    on:click={() => dispatch("close")}
    on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && dispatch('close')}
    on:keydown={trapFocus}
  >
    <section
      class="glass-lg rounded-2xl w-[90%] max-w-2xl p-6 flex flex-col"
      on:pointerdown|stopPropagation
      bind:this={modalEl}
      role="dialog"
      aria-modal="true"
      aria-labelledby="worker-setup-title"
      >
      <div class="flex justify-between items-center mb-4 shrink-0">
        <h2 id="worker-setup-title" class="text-2xl font-semibold">
          Worker Setup
        </h2>
        <button
          class="text-gray-100 hover:text-white transition-colors"
          on:click={() => dispatch("close")}
          aria-label="Close worker setup"
          bind:this={closeButton}
        >
          <X size={24} />
        </button>
      </div>
      <pre
        class="overflow-y-auto whitespace-pre-wrap text-sm flex-grow">{content}</pre>
      <input
        type="file"
        accept="text/*"
        class="hidden"
        bind:this={filePicker}
        on:change={importFile}
      />
      <button
        class="text-sm py-2 px-4 mt-4 rounded-xl border-transparent font-medium flex items-center justify-center gap-2 cursor-pointer transition-all w-auto bg-blue-500/20 text-blue-400 hover:bg-blue-500/30"
        on:click={() => filePicker && filePicker.click()}
      >
        Import Worker List
      </button>
      {#if importProgress !== null}
        <p class="text-xs mt-2 text-center">{importProgress}%</p>
      {/if}
    </section>
  </div>
{/if}
