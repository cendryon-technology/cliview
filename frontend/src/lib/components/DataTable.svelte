<script lang="ts">
  export let rows: Record<string, unknown>[] = [];

  $: columns = rows.length
    ? Array.from(
        rows.reduce((set, r) => {
          Object.keys(r).forEach((k) => set.add(k));
          return set;
        }, new Set<string>())
      )
    : [];

  function renderCell(v: unknown): string {
    if (v === null || v === undefined) return "—";
    if (typeof v === "object") return JSON.stringify(v);
    return String(v);
  }
</script>

{#if rows.length === 0}
  <div class="text-muted italic">No rows.</div>
{:else}
  <div class="overflow-x-auto rounded border border-border">
    <table class="w-full text-sm">
      <thead class="bg-panel text-left">
        <tr>
          {#each columns as col}
            <th class="px-3 py-2 font-medium text-muted">{col}</th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each rows as row}
          <tr class="border-t border-border hover:bg-panel/60">
            {#each columns as col}
              <td class="px-3 py-2 font-mono">{renderCell(row[col])}</td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}
