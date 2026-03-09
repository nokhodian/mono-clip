import type { Folder } from "$lib/api/tauri";
import { getFolders } from "$lib/api/tauri";

class FoldersStore {
  items = $state<Folder[]>([]);
  activeId = $state<number>(1);
  isLoading = $state(false);

  async load() {
    this.isLoading = true;
    try {
      this.items = await getFolders();
    } finally {
      this.isLoading = false;
    }
  }

  setActive(id: number) {
    this.activeId = id;
  }

  addFolder(folder: Folder) {
    this.items = [...this.items, folder];
  }

  updateFolder(folder: Folder) {
    this.items = this.items.map((f) => (f.id === folder.id ? folder : f));
  }

  removeFolder(id: number) {
    this.items = this.items.filter((f) => f.id !== id);
  }

  get active() {
    return this.items.find((f) => f.id === this.activeId) ?? null;
  }
}

export const foldersStore = new FoldersStore();
