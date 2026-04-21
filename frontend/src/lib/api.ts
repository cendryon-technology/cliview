// Typed client for the cliview HTTP API.

export interface MetaItem {
  id: string;
  title: string;
  input_schema?: JsonSchema | null;
}

export interface Meta {
  name: string;
  pages: MetaItem[];
  actions: MetaItem[];
  streams: MetaItem[];
}

// Loose JSON Schema shape — we only read a handful of fields for AutoForm.
export interface JsonSchema {
  type?: string | string[];
  properties?: Record<string, JsonSchema>;
  required?: string[];
  description?: string;
  title?: string;
  enum?: unknown[];
  items?: JsonSchema;
  format?: string;
  default?: unknown;
}

async function jsonOrThrow(res: Response) {
  if (!res.ok) {
    let msg = `${res.status} ${res.statusText}`;
    try {
      const body = await res.json();
      if (body?.error) msg = body.error;
    } catch {
      /* ignore */
    }
    throw new Error(msg);
  }
  return res.json();
}

export async function getMeta(): Promise<Meta> {
  return jsonOrThrow(await fetch("/api/meta"));
}

export async function getPage(id: string): Promise<unknown> {
  return jsonOrThrow(await fetch(`/api/pages/${encodeURIComponent(id)}`));
}

export async function runAction(
  id: string,
  input: unknown
): Promise<unknown> {
  return jsonOrThrow(
    await fetch(`/api/actions/${encodeURIComponent(id)}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(input ?? null)
    })
  );
}

/**
 * Subscribe to an SSE stream. Returns an unsubscribe function.
 * `onEvent` receives each JSON-decoded event payload.
 */
export function subscribeStream(
  id: string,
  onEvent: (value: unknown) => void,
  onError?: (err: Event) => void
): () => void {
  const es = new EventSource(`/api/streams/${encodeURIComponent(id)}`);
  es.onmessage = (e) => {
    try {
      onEvent(JSON.parse(e.data));
    } catch {
      onEvent(e.data);
    }
  };
  es.onerror = (e) => {
    onError?.(e);
    es.close();
  };
  return () => es.close();
}
