import { createStore } from 'idb-keyval';
let store: ReturnType<typeof createStore> | undefined;
/** IndexedDB store for user data (bookmarks, history). Lazily created. */
export const userDb = () => (store ??= createStore('kd-user', 'kv'));
