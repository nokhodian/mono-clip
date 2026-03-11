<script lang="ts">
  import { foldersStore } from "$lib/stores/folders.svelte";
  import { clipsStore } from "$lib/stores/clips.svelte";
  import FolderModal from "./FolderModal.svelte";
  import type { Folder } from "$lib/api/tauri";
  import { deleteFolder } from "$lib/api/tauri";

  interface Props {
    onSettingsClick?: () => void;
    onHelpClick?: () => void;
  }
  let { onSettingsClick, onHelpClick }: Props = $props();

  let showNewFolder = $state(false);
  let editingFolder = $state<Folder | null>(null);
  let showEditFolder = $state(false);
  let contextMenu = $state<{ folder: Folder; x: number; y: number } | null>(null);

  async function selectFolder(id: number) {
    foldersStore.setActive(id);
    await clipsStore.load(id, clipsStore.searchQuery || undefined);
  }

  function openContextMenu(e: MouseEvent, folder: Folder) {
    if (folder.id === 1) return; // No context menu for Inbox
    e.preventDefault();
    contextMenu = { folder, x: e.clientX, y: e.clientY };
  }

  async function handleDeleteFolder() {
    if (!contextMenu) return;
    const { folder } = contextMenu;
    contextMenu = null;
    try {
      await deleteFolder(folder.id);
      foldersStore.removeFolder(folder.id);
      if (foldersStore.activeId === folder.id) {
        foldersStore.setActive(1);
        await clipsStore.load(1);
      }
    } catch (e) {
      console.error(e);
    }
  }

  function startEditFolder() {
    if (!contextMenu) return;
    editingFolder = contextMenu.folder;
    showEditFolder = true;
    contextMenu = null;
  }

  function closeEditFolder() {
    showEditFolder = false;
    editingFolder = null;
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<aside class="w-48 flex-shrink-0 flex flex-col border-r border-white/5 py-2">
  <!-- Folder list -->
  <nav class="flex-1 overflow-y-auto px-2 space-y-0.5">
    {#each foldersStore.items as folder (folder.id)}
      <button
        class="w-full flex items-center gap-2.5 px-2.5 py-2 rounded-lg text-left text-sm
               transition-all duration-100 group
               {foldersStore.activeId === folder.id
                 ? 'bg-white/10 text-white/95'
                 : 'text-white/55 hover:text-white/80 hover:bg-white/5'}"
        onclick={() => selectFolder(folder.id)}
        oncontextmenu={(e) => openContextMenu(e, folder)}
      >
        <!-- Color dot -->
        <span
          class="w-2 h-2 rounded-full flex-shrink-0 transition-transform duration-150
                 {foldersStore.activeId === folder.id ? 'scale-125' : ''}"
          style="background-color: {folder.color};"
        ></span>
        <span class="truncate flex-1">{folder.icon} {folder.name}</span>
      </button>
    {/each}
  </nav>

  <!-- Bottom actions -->
  <div class="px-2 pt-2 border-t border-white/5 space-y-0.5">
    <button
      class="w-full flex items-center gap-2 px-2.5 py-2 rounded-lg text-sm
             text-white/40 hover:text-white/70 hover:bg-white/5 transition-colors"
      onclick={() => { showNewFolder = true; }}
    >
      <span class="text-base">+</span>
      <span>New Folder</span>
    </button>
    <button
      class="w-full flex items-center gap-2 px-2.5 py-2 rounded-lg text-sm
             text-white/40 hover:text-white/70 hover:bg-white/5 transition-colors"
      onclick={onHelpClick}
    >
      <span>?</span>
      <span>Help</span>
    </button>
    <button
      class="w-full flex items-center gap-2 px-2.5 py-2 rounded-lg text-sm
             text-white/40 hover:text-white/70 hover:bg-white/5 transition-colors"
      onclick={onSettingsClick}
    >
      <span>⚙</span>
      <span>Settings</span>
    </button>
  </div>
</aside>

<!-- Context menu -->
{#if contextMenu}
  <div
    class="fixed z-50 bg-[#2c2c2e]/95 backdrop-blur-xl rounded-xl shadow-2xl
           border border-white/10 overflow-hidden py-1 w-40"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
  >
    <button
      class="w-full px-3 py-2 text-sm text-left text-white/80 hover:bg-white/10
             flex items-center gap-2"
      onclick={startEditFolder}
    >
      ✏️ Edit
    </button>
    <button
      class="w-full px-3 py-2 text-sm text-left text-red-400 hover:bg-red-500/10
             flex items-center gap-2"
      onclick={handleDeleteFolder}
    >
      🗑️ Delete
    </button>
  </div>
  <!-- Dismiss backdrop -->
  <div
    class="fixed inset-0 z-40"
    onclick={() => { contextMenu = null; }}
  ></div>
{/if}

<!-- Modals -->
<FolderModal bind:open={showNewFolder} />

<FolderModal
  open={showEditFolder}
  editFolder={editingFolder}
  onclose={closeEditFolder}
/>
