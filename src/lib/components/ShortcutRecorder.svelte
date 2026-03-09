<script lang="ts">
  interface Props {
    value?: string;
    onchange?: (value: string) => void;
  }
  let { value = $bindable(""), onchange }: Props = $props();

  let recording = $state(false);
  let inputEl: HTMLInputElement;

  // Map browser key names → Tauri shortcut format
  const KEY_MAP: Record<string, string> = {
    " ": "Space",
    "ArrowUp": "Up",
    "ArrowDown": "Down",
    "ArrowLeft": "Left",
    "ArrowRight": "Right",
    "Enter": "Return",
    "Backspace": "Backspace",
    "Delete": "Delete",
    "Tab": "Tab",
    "Escape": "Escape",
    "Home": "Home",
    "End": "End",
    "PageUp": "PageUp",
    "PageDown": "PageDown",
  };

  function formatShortcut(e: KeyboardEvent): string | null {
    // Ignore bare modifier key presses
    if (["Meta", "Alt", "Shift", "Control"].includes(e.key)) return null;

    const parts: string[] = [];
    if (e.metaKey || e.ctrlKey) parts.push("CmdOrCtrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");

    // Require at least one non-shift modifier for safety
    if (!e.metaKey && !e.ctrlKey && !e.altKey) return null;

    const rawKey = e.key;
    const mappedKey = KEY_MAP[rawKey]
      ?? (rawKey.length === 1 ? rawKey.toUpperCase() : rawKey);

    parts.push(mappedKey);
    return parts.join("+");
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();

    const shortcut = formatShortcut(e);
    if (shortcut) {
      value = shortcut;
      onchange?.(shortcut);
      recording = false;
      inputEl.blur();
    }
  }

  function startRecording() {
    recording = true;
  }

  function stopRecording() {
    recording = false;
  }

  function clear(e: MouseEvent) {
    e.stopPropagation();
    value = "";
    onchange?.("");
  }
</script>

<div class="relative">
  <input
    bind:this={inputEl}
    type="text"
    readonly
    value={recording ? "" : value}
    placeholder={recording ? "Press shortcut…" : value ? value : "Click to record…"}
    class="w-full rounded-lg px-3 py-2 text-sm font-mono outline-none
           cursor-pointer select-none transition-all duration-150
           {recording
             ? 'bg-accent/10 border border-accent/60 text-accent placeholder-accent/50 ring-2 ring-accent/20'
             : 'bg-white/5 border border-white/10 text-white/90 placeholder-white/30 hover:border-white/20'}"
    onfocus={startRecording}
    onblur={stopRecording}
    onkeydown={handleKeydown}
  />

  <!-- Pulse dot while recording -->
  {#if recording}
    <span class="absolute left-3 top-1/2 -translate-y-1/2 w-1.5 h-1.5 rounded-full bg-accent animate-pulse"></span>
    <span class="absolute left-3 top-1/2 -translate-y-1/2 w-1.5 h-1.5 rounded-full bg-accent/40 animate-ping"></span>
  {/if}

  <!-- Clear button -->
  {#if value && !recording}
    <!-- svelte-ignore a11y_consider_explicit_label -->
    <button
      class="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 flex items-center justify-center
             rounded text-white/30 hover:text-white/70 hover:bg-white/10 transition-colors text-[10px]"
      onmousedown={clear}
    >✕</button>
  {/if}
</div>
