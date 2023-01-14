import type { Settings, SettingsPatch } from "$lib/api/tauri";
import { getSettings, updateSettings } from "$lib/api/tauri";

class SettingsStore {
  data = $state<Settings | null>(null);
  isLoading = $state(false);

  async load() {
    this.isLoading = true;
    try {
      this.data = await getSettings();
    } finally {
      this.isLoading = false;
    }
  }

  async update(patch: SettingsPatch) {
    this.data = await updateSettings(patch);
  }

  get theme() {
    return this.data?.theme ?? "system";
  }
}

export const settingsStore = new SettingsStore();
