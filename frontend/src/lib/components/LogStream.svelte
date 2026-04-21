<script lang="ts">
  import { onDestroy } from "svelte";
  import { subscribeStream } from "$lib/api";

  export let streamId: string;

  let lines: string[] = [];
  let error: string | null = null;
  let running = false;
  let unsub: (() => void) | null = null;

  function start() {
    stop();
    lines = [];
    error = null;
    running = true;
    unsub = subscribeStream(
      streamId,
      (v) => {
        lines = [...lines, typeof v === "string" ? v : JSON.stringify(v)];
      },
      () => {
        running = false;
        error = "stream ended";
      }
    );
  }

  function stop() {
    unsub?.();
    unsub = null;
    running = false;
  }

  onDestroy(stop);
</script>

<div class="space-y-2">
  <div class="flex items-center gap-2">
    <button
      class="rounded bg-accent px-3 py-1 text-sm font-medium text-bg disabled:opacity-50"
      on:click={start}
      disabled={running}>Start</button
    >
    <button
      class="rounded border border-border px-3 py-1 text-sm disabled:opacity-50"
      on:click={stop}
      disabled={!running}>Stop</button
    >
    {#if error}
      <span class="text-xs text-muted">{error}</span>
    {/if}
  </div>
  <pre
    class="h-64 overflow-auto rounded border border-border bg-panel p-3 font-mono text-xs">{lines.join(
      "\n"
    ) || "(no output yet)"}</pre>
</div>
