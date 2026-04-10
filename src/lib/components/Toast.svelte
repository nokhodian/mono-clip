<script lang="ts">
  interface ToastMessage {
    id: number;
    message: string;
    type: "success" | "info" | "error";
  }

  let toasts = $state<ToastMessage[]>([]);
  let nextId = 0;

  export function show(message: string, type: ToastMessage["type"] = "success", duration = 3000) {
    const id = nextId++;
    toasts = [...toasts, { id, message, type }];
    setTimeout(() => {
      toasts = toasts.filter((t) => t.id !== id);
    }, duration);
  }
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
  {#each toasts as toast (toast.id)}
    <div
      class="animate-fade-up px-4 py-2.5 rounded-xl text-sm font-medium shadow-lg backdrop-blur-xl
             flex items-center gap-2 pointer-events-auto
             {toast.type === 'success' ? 'bg-accent/90 text-white' : ''}
             {toast.type === 'error' ? 'bg-red-500/90 text-white' : ''}
             {toast.type === 'info' ? 'bg-white/10 text-white border border-white/10' : ''}"
    >
      {#if toast.type === "success"}
        <span>✓</span>
      {:else if toast.type === "error"}
        <span>✕</span>
      {/if}
      {toast.message}
    </div>
  {/each}
</div>
