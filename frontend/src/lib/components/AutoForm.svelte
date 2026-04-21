<script lang="ts">
  import type { JsonSchema } from "$lib/api";

  export let schema: JsonSchema | null | undefined;
  export let value: Record<string, unknown> = {};

  // Resolve $ref style or nested "definitions" — schemars wraps in
  // { "$ref": "#/definitions/Foo", "definitions": { "Foo": {...} } }.
  // We only need to handle the top-level case.
  function resolve(s: JsonSchema | null | undefined): JsonSchema {
    if (!s) return {};
    const anyS = s as JsonSchema & {
      $ref?: string;
      definitions?: Record<string, JsonSchema>;
    };
    if (anyS.$ref && anyS.definitions) {
      const name = anyS.$ref.split("/").pop();
      if (name && anyS.definitions[name]) return anyS.definitions[name];
    }
    return s;
  }

  $: resolved = resolve(schema);
  $: props = resolved.properties ?? {};
  $: required = new Set(resolved.required ?? []);

  function fieldType(p: JsonSchema): "string" | "number" | "boolean" | "json" {
    const t = Array.isArray(p.type) ? p.type[0] : p.type;
    if (t === "integer" || t === "number") return "number";
    if (t === "boolean") return "boolean";
    if (t === "string") return "string";
    return "json";
  }

  function setField(name: string, v: unknown) {
    value = { ...value, [name]: v };
  }
</script>

{#if Object.keys(props).length === 0}
  <div class="text-muted text-sm italic">No input required.</div>
{:else}
  <div class="space-y-3">
    {#each Object.entries(props) as [name, prop]}
      {@const t = fieldType(prop)}
      <label class="block text-sm">
        <span class="mb-1 block text-muted">
          {prop.title ?? name}
          {#if required.has(name)}<span class="text-red-400">*</span>{/if}
        </span>
        {#if t === "boolean"}
          <input
            type="checkbox"
            checked={Boolean(value[name])}
            on:change={(e) => setField(name, e.currentTarget.checked)}
          />
        {:else if t === "number"}
          <input
            type="number"
            class="w-full rounded border border-border bg-panel px-2 py-1 font-mono"
            value={value[name] ?? ""}
            on:input={(e) =>
              setField(
                name,
                e.currentTarget.value === ""
                  ? undefined
                  : Number(e.currentTarget.value)
              )}
          />
        {:else if t === "string"}
          <input
            type="text"
            class="w-full rounded border border-border bg-panel px-2 py-1 font-mono"
            value={value[name] ?? ""}
            on:input={(e) => setField(name, e.currentTarget.value)}
          />
        {:else}
          <textarea
            class="w-full rounded border border-border bg-panel px-2 py-1 font-mono text-xs"
            rows="3"
            value={value[name] ? JSON.stringify(value[name]) : ""}
            on:input={(e) => {
              try {
                setField(name, JSON.parse(e.currentTarget.value));
              } catch {
                /* keep prior */
              }
            }}
          />
        {/if}
        {#if prop.description}
          <span class="mt-0.5 block text-xs text-muted">{prop.description}</span>
        {/if}
      </label>
    {/each}
  </div>
{/if}
