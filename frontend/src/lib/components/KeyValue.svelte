<script lang="ts">
  export let data: unknown;

  function entries(v: unknown): [string, unknown][] {
    if (v && typeof v === "object" && !Array.isArray(v)) {
      return Object.entries(v as Record<string, unknown>);
    }
    return [];
  }

  function render(v: unknown): string {
    if (v === null || v === undefined) return "—";
    if (typeof v === "object") return JSON.stringify(v, null, 2);
    return String(v);
  }
</script>

<dl class="grid grid-cols-[max-content_1fr] gap-x-6 gap-y-2 text-sm">
  {#each entries(data) as [k, v]}
    <dt class="text-muted">{k}</dt>
    <dd class="font-mono whitespace-pre-wrap">{render(v)}</dd>
  {/each}
</dl>
