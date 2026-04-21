<script lang="ts">
  import { onMount } from "svelte";
  import { getMeta, getPage, type Meta, type MetaItem } from "$lib/api";
  import AutoView from "$lib/components/AutoView.svelte";
  import ActionRunner from "$lib/components/ActionRunner.svelte";
  import LogStream from "$lib/components/LogStream.svelte";

  type NavItem =
    | { kind: "page"; item: MetaItem }
    | { kind: "stream"; item: MetaItem };

  let meta: Meta | null = null;
  let loadError: string | null = null;
  let selected: NavItem | null = null;
  let pageData: unknown = undefined;
  let pageError: string | null = null;
  let pageLoading = false;

  onMount(async () => {
    try {
      meta = await getMeta();
      const first =
        meta.pages[0] ?? meta.streams[0] ?? null;
      if (first) {
        selected = meta.pages[0]
          ? { kind: "page", item: meta.pages[0] }
          : { kind: "stream", item: meta.streams[0] };
      }
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    }
  });

  async function select(n: NavItem) {
    selected = n;
    pageData = undefined;
    pageError = null;
    if (n.kind === "page") {
      pageLoading = true;
      try {
        pageData = await getPage(n.item.id);
      } catch (e) {
        pageError = e instanceof Error ? e.message : String(e);
      } finally {
        pageLoading = false;
      }
    }
  }

  // Actions scoped to the active page by dotted-prefix convention:
  // action id "users.delete" attaches to page "users".
  $: scopedActions =
    meta && selected?.kind === "page"
      ? meta.actions.filter(
          (a) =>
            a.id.startsWith(selected!.item.id + ".") || a.id === selected!.item.id
        )
      : meta?.actions ?? [];
</script>

<div class="flex min-h-screen">
  <aside class="w-60 shrink-0 border-r border-border bg-panel/40 p-4">
    <div class="mb-4">
      <div class="text-xs uppercase tracking-wide text-muted">cliview</div>
      <div class="text-lg font-semibold">{meta?.name ?? "…"}</div>
    </div>

    {#if loadError}
      <div class="text-sm text-red-400">Failed to load: {loadError}</div>
    {/if}

    {#if meta}
      {#if meta.pages.length}
        <div class="mb-2 mt-4 text-xs uppercase text-muted">Pages</div>
        <ul class="space-y-0.5">
          {#each meta.pages as p}
            <li>
              <button
                class="w-full rounded px-2 py-1 text-left text-sm hover:bg-panel"
                class:bg-panel={selected?.kind === "page" &&
                  selected.item.id === p.id}
                on:click={() => select({ kind: "page", item: p })}
              >
                {p.title}
              </button>
            </li>
          {/each}
        </ul>
      {/if}

      {#if meta.streams.length}
        <div class="mb-2 mt-4 text-xs uppercase text-muted">Streams</div>
        <ul class="space-y-0.5">
          {#each meta.streams as s}
            <li>
              <button
                class="w-full rounded px-2 py-1 text-left text-sm hover:bg-panel"
                class:bg-panel={selected?.kind === "stream" &&
                  selected.item.id === s.id}
                on:click={() => select({ kind: "stream", item: s })}
              >
                {s.title}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  </aside>

  <main class="flex-1 p-6">
    {#if !selected}
      <div class="text-muted">Select something from the sidebar.</div>
    {:else if selected.kind === "page"}
      <div class="mb-4">
        <h1 class="text-xl font-semibold">{selected.item.title}</h1>
        <div class="text-xs text-muted">/api/pages/{selected.item.id}</div>
      </div>

      {#if pageLoading}
        <div class="text-muted">Loading…</div>
      {:else if pageError}
        <div class="text-red-400">{pageError}</div>
      {:else}
        <AutoView data={pageData} />
      {/if}

      {#if scopedActions.length}
        <div class="mt-8 space-y-4">
          <h2 class="text-sm uppercase tracking-wide text-muted">Actions</h2>
          {#each scopedActions as a (a.id)}
            <ActionRunner action={a} />
          {/each}
        </div>
      {/if}
    {:else}
      <div class="mb-4">
        <h1 class="text-xl font-semibold">{selected.item.title}</h1>
        <div class="text-xs text-muted">/api/streams/{selected.item.id}</div>
      </div>
      <LogStream streamId={selected.item.id} />
    {/if}
  </main>
</div>
