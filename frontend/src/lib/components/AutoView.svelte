<script lang="ts">
  import DataTable from "./DataTable.svelte";
  import KeyValue from "./KeyValue.svelte";

  export let data: unknown;

  function kind(v: unknown): "table" | "object" | "scalar" | "empty" {
    if (v === null || v === undefined) return "empty";
    if (Array.isArray(v)) {
      if (v.length === 0) return "empty";
      if (v.every((x) => x && typeof x === "object" && !Array.isArray(x)))
        return "table";
      return "scalar";
    }
    if (typeof v === "object") return "object";
    return "scalar";
  }

  $: k = kind(data);
  $: tableRows = (Array.isArray(data) ? data : []) as Record<string, unknown>[];
</script>

{#if k === "table"}
  <DataTable rows={tableRows} />
{:else if k === "object"}
  <KeyValue {data} />
{:else if k === "empty"}
  <div class="text-muted italic">No data.</div>
{:else}
  <pre
    class="rounded border border-border bg-panel p-3 font-mono text-sm overflow-x-auto">{JSON.stringify(
      data,
      null,
      2
    )}</pre>
{/if}
