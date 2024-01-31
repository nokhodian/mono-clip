<script lang="ts">
  interface Props {
    open?: boolean;
    onclose?: () => void;
  }
  let { open = $bindable(false), onclose }: Props = $props();
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
      <!-- Header -->
      <div class="flex items-center justify-between mb-5">
        <h2 class="text-sm font-semibold text-white/90">Help</h2>
        <button
          class="text-white/40 hover:text-white/70 transition-colors"
          onclick={() => { open = false; onclose?.(); }}
        >✕</button>
      </div>

      <!-- Keyboard Shortcuts -->
      <section class="mb-5">
        <h3 class="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">Keyboard Shortcuts</h3>
        <div class="space-y-2">
          {#each [
            ["Open / Close", "⌘ ⇧ V"],
            ["Search", "⌘ F  or  /"],
            ["Dismiss / Close", "Esc"],
          ] as [action, keys]}
            <div class="flex items-center justify-between">
              <span class="text-sm text-white/65">{action}</span>
              <kbd class="font-mono text-xs bg-white/8 border border-white/10 rounded-md px-2 py-0.5 text-white/70">{keys}</kbd>
            </div>
          {/each}
        </div>
      </section>

      <!-- Using the inbox -->
      <section class="mb-5">
        <h3 class="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">Using MonoClip</h3>
        <ul class="space-y-2 text-sm text-white/60 list-none">
          <li class="flex gap-2"><span class="text-white/30 shrink-0">→</span>Everything you copy lands in <strong class="text-white/80">Inbox</strong> automatically.</li>
          <li class="flex gap-2"><span class="text-white/30 shrink-0">→</span>Click any clip to copy it back to your clipboard.</li>
          <li class="flex gap-2"><span class="text-white/30 shrink-0">→</span>Right-click a folder to rename or delete it.</li>
          <li class="flex gap-2"><span class="text-white/30 shrink-0">→</span>Images are captured and shown as thumbnails.</li>
          <li class="flex gap-2"><span class="text-white/30 shrink-0">→</span>Copying files or folders captures the full path.</li>
          <li class="flex gap-2"><span class="text-white/30 shrink-0">→</span>Pin a clip 📌 to keep it from being auto-cleaned.</li>
        </ul>
      </section>

      <!-- CLI -->
      <section class="mb-5">
        <h3 class="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">CLI — mclip</h3>
        <p class="text-xs text-white/40 mb-3">
          Install via <span class="font-mono text-white/60">Settings → Install mclip CLI</span>, then add
          <span class="font-mono text-white/60">~/.local/bin</span> to your <span class="font-mono text-white/60">$PATH</span>.
        </p>
        <div class="bg-black/30 rounded-xl p-3 font-mono text-xs space-y-1.5 text-white/65">
          {#each [
            ["mclip list", "List recent inbox items"],
            ["mclip list --folder Work", "List a specific folder"],
            ["mclip list --search query", "Search clips"],
            ["mclip add \"text\"", "Add a clip to inbox"],
            ["mclip add \"text\" --folder Work", "Add to a folder"],
            ["mclip remove <id>", "Delete a clip"],
            ["mclip pin <id>", "Pin a clip"],
            ["mclip get <id>", "Print raw content (pipeable)"],
            ["mclip folder list", "List all folders"],
            ["mclip folder add \"Name\"", "Create a folder"],
            ["mclip folder remove \"Name\"", "Delete a folder"],
            ["mclip context", "Print AI context block to paste into chat"],
            ["mclip mcp", "Start MCP stdio server (Claude Desktop etc.)"],
          ] as [cmd, desc]}
            <div class="flex gap-3">
              <span class="text-[#6366f1]/90 shrink-0 w-44">{cmd}</span>
              <span class="text-white/35"># {desc}</span>
            </div>
          {/each}
        </div>
      </section>

      <!-- AI Integration -->
      <section class="mb-5">
        <h3 class="text-xs font-medium text-white/40 uppercase tracking-wider mb-3">AI Integration</h3>
        <ul class="space-y-2 text-sm text-white/60 list-none">
          <li class="flex gap-2">
            <span class="text-white/30 shrink-0">→</span>
            <span>Run <code class="font-mono text-white/70">mclip context</code> and paste the output into any AI chat so it knows all available commands.</span>
          </li>
          <li class="flex gap-2">
            <span class="text-white/30 shrink-0">→</span>
            <span>For Claude Desktop / Cursor, add <code class="font-mono text-white/70">mclip mcp</code> as an MCP stdio server — the AI can then call your clipboard directly.</span>
          </li>
        </ul>
        <div class="bg-black/30 rounded-xl p-3 font-mono text-xs mt-3 text-white/50">
          <span class="text-white/30"># ~/.config/claude/claude_desktop_config.json</span><br/>
          <span class="text-[#6366f1]/90">&#123;"mcpServers": &#123;"mclip": &#123;"command": "mclip", "args": ["mcp"]&#125;&#125;&#125;</span>
        </div>
      </section>

      <!-- Version -->
      <p class="text-xs text-white/20 text-center">MonoClip v0.1.0</p>
    </div>
  </div>
{/if}
