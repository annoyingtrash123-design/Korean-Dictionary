import { createStore } from 'idb-keyval';
let store: ReturnType<typeof createStore> | undefined;
/** IndexedDB store for user data (lists, history). Lazily created. */
export const userDb = () => (store ??= createStore('kd-user', 'kv'));
