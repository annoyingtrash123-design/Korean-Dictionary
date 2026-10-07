/** Tiny in-memory LRU used to make back-navigation and repeat views instant. */
export class Lru<V> {
  private m = new Map<string, V>();
  constructor(private max = 60) {}
  get(k: string): V | undefined {
    const v = this.m.get(k);
    if (v !== undefined) { this.m.delete(k); this.m.set(k, v); }
    return v;
  }
  set(k: string, v: V): void {
    this.m.delete(k); this.m.set(k, v);
    if (this.m.size > this.max) this.m.delete(this.m.keys().next().value as string);
  }
  clear(): void { this.m.clear(); }
}
