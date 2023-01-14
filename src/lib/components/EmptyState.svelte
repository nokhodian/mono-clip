<script lang="ts">
  interface Props {
    query?: string;
    folderName?: string;
  }
  let { query, folderName }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center h-full gap-4 opacity-60 select-none">
  <div class="text-6xl">{query ? "🔍" : "📋"}</div>
  <div class="text-center">
    <p class="text-sm font-medium text-white/70">
      {#if query}
        No clips match "<span class="text-white/90 font-semibold">{query}</span>"
      {:else if folderName}
        {folderName} is empty
      {:else}
        Nothing saved yet
      {/if}
    </p>
    {#if !query}
      <p class="text-xs text-white/40 mt-1">
        Copy something and it'll appear here
      </p>
    {/if}
  </div>
</div>
