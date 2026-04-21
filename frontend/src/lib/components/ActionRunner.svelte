<script lang="ts">
  import { runAction, type MetaItem } from "$lib/api";
  import AutoForm from "./AutoForm.svelte";
  import AutoView from "./AutoView.svelte";

  export let action: MetaItem;

  let input: Record<string, unknown> = {};
  let result: unknown = undefined;
  let error: string | null = null;
  let busy = false;

  async function run() {
    busy = true;
    error = null;
    try {
      result = await runAction(action.id, input);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="space-y-3 rounded border border-border bg-panel/40 p-4">
  <h3 class="font-semibold">{action.title}</h3>
  <AutoForm schema={action.input_schema} bind:value={input} />
  <div class="flex items-center gap-2">
    <button
      class="rounded bg-accent px-3 py-1 text-sm font-medium text-bg disabled:opacity-50"
      on:click={run}
      disabled={busy}>{busy ? "Running…" : "Run"}</button
    >
    {#if error}<span class="text-sm text-red-400">{error}</span>{/if}
  </div>
  {#if result !== undefined}
    <div>
      <div class="mb-1 text-xs uppercase tracking-wide text-muted">Result</div>
      <AutoView data={result} />
    </div>
  {/if}
</div>
