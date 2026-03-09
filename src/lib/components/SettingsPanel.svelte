<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { runAutoCleanup } from "$lib/api/tauri";

  interface Props {
    open?: boolean;
    onclose?: () => void;
  }
  let { open = $bindable(false), onclose }: Props = $props();

  let cleanupResult = $state<number | null>(null);

  async function handleCleanup() {
    const count = await runAutoCleanup();
    cleanupResult = count;
    setTimeout(() => { cleanupResult = null; }, 3000);
  }
</script>

{#if open}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 bg-black/40 z-40 flex items-end justify-stretch"
    onclick={() => { open = false; onclose?.(); }}
  >
    <div
      class="flex-1 bg-[#1c1c1e]/95 backdrop-blur-2xl border-t border-white/10
             rounded-t-2xl p-5 animate-slide-in z-50 max-h-[80%] overflow-y-auto"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
    >
      <div class="flex items-center justify-between mb-5">
        <h2 class="text-sm font-semibold text-white/90">Settings</h2>
        <button
          class="text-white/40 hover:text-white/70 transition-colors"
          onclick={() => { open = false; onclose?.(); }}
        >✕</button>
      </div>

      {#if settingsStore.data}
        {@const s = settingsStore.data}

        <!-- General -->
        <section class="mb-5">
          <h3 class="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">General</h3>

          <div class="space-y-3">
            <label class="flex items-center justify-between">
              <span class="text-sm text-white/75">Launch at Login</span>
              <input
                type="checkbox"
                checked={s.launchAtLogin}
                onchange={(e) => settingsStore.update({ launchAtLogin: (e.target as HTMLInputElement).checked })}
                class="w-4 h-4 accent-[#6366f1]"
              />
            </label>

            <label class="flex items-center justify-between">
              <span class="text-sm text-white/75">Paste on Click</span>
              <input
                type="checkbox"
                checked={s.pasteOnClick}
                onchange={(e) => settingsStore.update({ pasteOnClick: (e.target as HTMLInputElement).checked })}
                class="w-4 h-4 accent-[#6366f1]"
              />
            </label>

            <div class="flex items-center justify-between">
              <span class="text-sm text-white/75">Theme</span>
              <select
                value={s.theme}
                onchange={(e) => settingsStore.update({ theme: (e.target as HTMLSelectElement).value as "light" | "dark" | "system" })}
                class="bg-white/10 border border-white/10 rounded-lg px-2 py-1 text-sm text-white/80 outline-none"
              >
                <option value="system">System</option>
                <option value="dark">Dark</option>
                <option value="light">Light</option>
              </select>
            </div>
          </div>
        </section>

        <!-- Clipboard -->
        <section class="mb-5">
          <h3 class="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">Clipboard</h3>
          <div class="space-y-3">
            <div>
              <div class="flex items-center justify-between mb-1.5">
                <span class="text-sm text-white/75">Max History</span>
                <span class="text-sm text-white/50">{s.maxHistoryItems} items</span>
              </div>
              <input
                type="range"
                min="50"
                max="2000"
                step="50"
                value={s.maxHistoryItems}
                oninput={(e) => settingsStore.update({ maxHistoryItems: Number((e.target as HTMLInputElement).value) })}
                class="w-full accent-[#6366f1]"
              />
            </div>
          </div>
        </section>

        <!-- Auto Cleanup -->
        <section class="mb-5">
          <h3 class="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">Auto Cleanup</h3>
          <div class="space-y-3">
            <label class="flex items-center justify-between">
              <span class="text-sm text-white/75">Auto Clean</span>
              <input
                type="checkbox"
                checked={s.autoCleanEnabled}
                onchange={(e) => settingsStore.update({ autoCleanEnabled: (e.target as HTMLInputElement).checked })}
                class="w-4 h-4 accent-[#6366f1]"
              />
            </label>

            {#if s.autoCleanEnabled}
              <div>
                <div class="flex items-center justify-between mb-1.5">
                  <span class="text-sm text-white/75">Keep for</span>
                  <span class="text-sm text-white/50">{s.autoCleanDays} days</span>
                </div>
                <input
                  type="range"
                  min="1"
                  max="365"
                  value={s.autoCleanDays}
                  oninput={(e) => settingsStore.update({ autoCleanDays: Number((e.target as HTMLInputElement).value) })}
                  class="w-full accent-[#6366f1]"
                />
              </div>
            {/if}

            <button
              class="w-full py-2 rounded-lg text-sm text-white/70 border border-white/10
                     hover:bg-white/5 transition-colors"
              onclick={handleCleanup}
            >
              {cleanupResult !== null ? `Removed ${cleanupResult} clips` : "Clean Now"}
            </button>
          </div>
        </section>

        <!-- Shortcuts -->
        <section>
          <h3 class="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">Master Shortcut</h3>
          <input
            type="text"
            value={s.masterShortcut}
            onblur={(e) => settingsStore.update({ masterShortcut: (e.target as HTMLInputElement).value })}
            class="w-full bg-white/5 border border-white/10 rounded-lg px-3 py-2
                   text-sm text-white/90 font-mono outline-none focus:border-accent/50"
          />
          <p class="text-xs text-white/30 mt-1.5">
            Format: CmdOrCtrl+Shift+V
          </p>
        </section>
      {/if}
    </div>
  </div>
{/if}
