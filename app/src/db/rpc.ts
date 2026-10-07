// Tiny message RPC between the UI thread and the DB worker.
export interface Req { id: number; method: string; args: unknown[] }
export interface Res { id: number; result?: unknown; error?: string }
export interface Evt { event: string; payload: unknown }
