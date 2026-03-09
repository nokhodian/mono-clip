<script lang="ts">
  import type { ClipItem } from "$lib/api/tauri";
  import { copyToClipboard, deleteClip, pinClip, unpinClip } from "$lib/api/tauri";
  import { relativeTime } from "$lib/utils/time";
  import { clipsStore } from "$lib/stores/clips.svelte";

  interface Props {
    clip: ClipItem;
    index?: number;
    onCopy?: (id: number) => void;
  }

  let { clip, index = 0, onCopy }: Props = $props();

  let isHovered = $state(false);
  let isFlashing = $derived(clipsStore.flashingId === clip.id);

  async function handleCopy(e: MouseEvent) {
    e.stopPropagation();
    try {
      await copyToClipboard(clip.id);
      clipsStore.setFlashing(clip.id);
      onCopy?.(clip.id);
    } catch (err) {
      console.error("Copy failed:", err);
    }
  }

  async function handlePin(e: MouseEvent) {
    e.stopPropagation();
    try {
      if (clip.isPinned) {
        await unpinClip(clip.id);
        clipsStore.updateItem({ ...clip, isPinned: false });
      } else {
        await pinClip(clip.id);
        clipsStore.updateItem({ ...clip, isPinned: true });
      }
    } catch (err) {
      console.error("Pin failed:", err);
    }
  }

  async function handleDelete(e: MouseEvent) {
    e.stopPropagation();
    try {
      await deleteClip(clip.id);
      clipsStore.removeItem(clip.id);
    } catch (err) {
      console.error("Delete failed:", err);
    }
  }

  const typeIcon: Record<string, string> = {
    url: "🔗",
    email: "📧",
    color: "🎨",
    code: "</>",
    text: "",
  };

  const animDelay = `${Math.min(index * 30, 300)}ms`;
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="group relative rounded-xl p-3 cursor-pointer transition-all duration-150
         border border-transparent
         {isFlashing
           ? 'animate-copy-flash bg-accent/20 border-accent/30'
           : 'bg-white/5 hover:bg-white/10 hover:border-white/10'}
         opacity-0"
  style="animation: fade-up 200ms ease-out {animDelay} forwards;"
  onmouseenter={() => (isHovered = true)}
  onmouseleave={() => (isHovered = false)}
  onclick={handleCopy}
  role="button"
  tabindex="0"
  onkeydown={(e) => e.key === "Enter" && handleCopy(e as unknown as MouseEvent)}
>
  <!-- Pin indicator -->
  {#if clip.isPinned}
    <div class="absolute top-2 right-2 text-xs opacity-60">📌</div>
  {/if}

  <!-- Content type icon -->
  {#if typeIcon[clip.contentType]}
    <span class="text-xs opacity-40 mb-1 block font-mono">
      {typeIcon[clip.contentType]}
    </span>
  {/if}

  <!-- Content preview -->
  <p
    class="text-sm leading-relaxed line-clamp-4 selectable
           {clip.contentType === 'code' ? 'font-mono text-xs text-green-300/80' : 'text-white/85'}
           {clip.contentType === 'url' ? 'text-blue-300/80 underline-offset-2' : ''}"
  >
    {clip.preview || clip.content}
  </p>

  <!-- Color swatch for color type -->
  {#if clip.contentType === "color"}
    <div
      class="w-full h-6 rounded mt-2 border border-white/10"
      style="background-color: {clip.content};"
    ></div>
  {/if}

  <!-- Footer -->
  <div class="flex items-center justify-between mt-2">
    <span class="text-xs text-white/30">{relativeTime(clip.updatedAt)}</span>

    <!-- Hover actions -->
    <div
      class="flex items-center gap-1 transition-opacity duration-100
             {isHovered ? 'opacity-100' : 'opacity-0'}"
    >
      <button
        class="p-1 rounded-md hover:bg-white/15 text-white/50 hover:text-white/90 text-xs transition-colors"
        onclick={handlePin}
        title={clip.isPinned ? "Unpin" : "Pin"}
      >
        {clip.isPinned ? "📌" : "📍"}
      </button>
      <button
        class="p-1 rounded-md hover:bg-red-500/20 text-white/50 hover:text-red-400 text-xs transition-colors"
        onclick={handleDelete}
        title="Delete"
      >
        ✕
      </button>
    </div>
  </div>
</div>
