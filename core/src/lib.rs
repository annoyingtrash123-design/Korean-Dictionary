//! `kdict-core`: the engine of the offline Korean-English dictionary PWA.
//!
//! * [`hangul`] - syllable compose/decompose and batchim helpers;
//! * [`deconjugate`] - inflected word -> dictionary-form candidates, grammar hints;
//! * [`sql`] - small SQLite abstraction (rusqlite natively, sqlite-wasm-rs in the browser);
//! * [`search`] - script-aware search and all query logic (plain Rust, natively testable);
//! * `db`, `import` (wasm only) - OPFS sahpool storage and streaming pack import;
//! * the `Engine` class below - the wasm-bindgen API used by the web worker.

pub mod deconjugate;
pub mod hangul;
pub mod packfile;
pub mod search;
pub mod sql;

#[cfg(target_arch = "wasm32")]
mod db;
#[cfg(target_arch = "wasm32")]
mod import;

#[cfg(target_arch = "wasm32")]
mod wasm_api {
    use crate::db::Store;
    use crate::import::Importers;
    use crate::search;
    use serde::{Deserialize, Serialize};
    use std::cell::RefCell;
    use wasm_bindgen::prelude::*;

    #[derive(Deserialize)]
    struct SearchOpts {
        #[serde(default)]
        packs: Vec<String>,
        #[serde(default)]
        limit: Option<usize>,
    }

    fn js_err(e: impl std::fmt::Display) -> JsError {
        JsError::new(&e.to_string())
    }

    /// Plain JS objects (not Maps), `None` as `null`.
    fn to_js<T: Serialize>(v: &T) -> Result<JsValue, JsError> {
        v.serialize(&serde_wasm_bindgen::Serializer::json_compatible()).map_err(js_err)
    }

    #[derive(Default)]
    struct State {
        store: Option<Store>,
        imports: Importers,
    }

    /// The dictionary engine. One instance per worker; call `init()` first.
    #[wasm_bindgen]
    pub struct Engine {
        state: RefCell<State>,
    }

    impl Engine {
        fn with_store<T>(&self, f: impl FnOnce(&Store) -> Result<T, JsError>) -> Result<T, JsError> {
            let st = self.state.borrow();
            match st.store.as_ref() {
                Some(s) => f(s),
                None => Err(JsError::new("engine not initialised: call init() first")),
            }
        }
    }

    #[wasm_bindgen]
    impl Engine {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Engine {
            console_error_panic_hook::set_once();
            Engine { state: RefCell::new(State::default()) }
        }

        /// Install the OPFS sahpool VFS and open the packs already stored. Idempotent.
        pub async fn init(&self) -> Result<(), JsError> {
            if self.state.borrow().store.is_some() {
                return Ok(());
            }
            let store = Store::open().await.map_err(js_err)?;
            let mut st = self.state.borrow_mut();
            if st.store.is_none() {
                st.store = Some(store);
            }
            Ok(())
        }

        /// `{id, version, bytes}[]`
        #[wasm_bindgen(js_name = installedPacks)]
        pub async fn installed_packs(&self) -> Result<JsValue, JsError> {
            self.with_store(|s| to_js(&s.installed_packs()))
        }

        #[wasm_bindgen(js_name = beginImport)]
        pub async fn begin_import(&self, pack_id: String, total_bytes: f64) -> Result<(), JsError> {
            if !(total_bytes >= 0.0 && total_bytes.fract() == 0.0) {
                return Err(JsError::new("totalBytes must be a non-negative integer"));
            }
            let mut st = self.state.borrow_mut();
            let State { store, imports } = &mut *st;
            let store = store.as_mut().ok_or_else(|| JsError::new("engine not initialised: call init() first"))?;
            imports.begin(store, &pack_id, total_bytes as u64).map_err(js_err)
        }

        #[wasm_bindgen(js_name = writeChunk)]
        pub async fn write_chunk(&self, pack_id: String, bytes: js_sys::Uint8Array) -> Result<(), JsError> {
            self.state.borrow_mut().imports.write(&pack_id, &bytes).map_err(js_err)
        }

        #[wasm_bindgen(js_name = finishImport)]
        pub async fn finish_import(&self, pack_id: String, version: String, sha256: Option<String>) -> Result<(), JsError> {
            let mut st = self.state.borrow_mut();
            let State { store, imports } = &mut *st;
            let store = store.as_mut().ok_or_else(|| JsError::new("engine not initialised: call init() first"))?;
            imports.finish(store, &pack_id, &version, sha256.as_deref()).map_err(js_err)
        }

        #[wasm_bindgen(js_name = deletePack)]
        pub async fn delete_pack(&self, pack_id: String) -> Result<(), JsError> {
            let mut st = self.state.borrow_mut();
            let State { store, imports } = &mut *st;
            let store = store.as_mut().ok_or_else(|| JsError::new("engine not initialised: call init() first"))?;
            imports.abort(&pack_id);
            store.remove_pack(&pack_id).map_err(js_err)
        }

        /// Run background warm-up step `step` over the given packs; resolves to whether more steps remain.
        pub async fn warm(&self, step: u32, packs: JsValue) -> Result<bool, JsError> {
            let ids: Vec<String> = serde_wasm_bindgen::from_value(packs).unwrap_or_default();
            self.with_store(|s| {
                let packs = if ids.is_empty() { s.all() } else { s.select(&ids) };
                Ok(search::warm_step(&packs, step as usize))
            })
        }

        /// Diagnostics log (startup file discovery, pack swaps) for troubleshooting.
        pub fn diagnostics(&self) -> String {
            crate::db::diag_dump()
        }

        /// `search(query, {packs, limit?}) -> SearchResult`
        pub async fn search(&self, query: String, opts: JsValue) -> Result<JsValue, JsError> {
            let opts: SearchOpts = if opts.is_undefined() || opts.is_null() {
                SearchOpts { packs: vec![], limit: None }
            } else {
                serde_wasm_bindgen::from_value(opts).map_err(js_err)?
            };
            self.with_store(|s| {
                let packs = if opts.packs.is_empty() { s.all() } else { s.select(&opts.packs) };
                to_js(&search::search(&packs, &query, opts.limit).map_err(js_err)?)
            })
        }

        #[wasm_bindgen(js_name = entriesByHeadword)]
        pub async fn entries_by_headword(&self, hw: String, packs: Vec<String>) -> Result<JsValue, JsError> {
            self.with_store(|s| to_js(&search::entries_by_headword(&s.select(&packs), &hw).map_err(js_err)?))
        }

        /// `Entry | null`
        pub async fn entry(&self, source: String, id: f64) -> Result<JsValue, JsError> {
            self.with_store(|s| to_js(&search::entry(&s.all(), &source, id as i64).map_err(js_err)?))
        }

        #[wasm_bindgen(js_name = hanjaChar)]
        pub async fn hanja_char(&self, ch: String) -> Result<JsValue, JsError> {
            self.with_store(|s| to_js(&search::hanja_char(&s.all(), &ch).map_err(js_err)?))
        }

        #[wasm_bindgen(js_name = wordsWithHanja)]
        pub async fn words_with_hanja(&self, ch: String, limit: u32, offset: u32) -> Result<JsValue, JsError> {
            self.with_store(|s| {
                to_js(&search::words_with_hanja(&s.all(), &ch, limit as usize, offset as usize).map_err(js_err)?)
            })
        }

        pub async fn sentences(&self, text: String, limit: u32) -> Result<JsValue, JsError> {
            self.with_store(|s| to_js(&search::sentences(&s.all(), &text, limit as usize).map_err(js_err)?))
        }

        #[wasm_bindgen(js_name = grammarList)]
        pub async fn grammar_list(&self) -> Result<JsValue, JsError> {
            self.with_store(|s| to_js(&search::grammar_list(&s.all()).map_err(js_err)?))
        }

        #[wasm_bindgen(js_name = wordOfDay)]
        pub async fn word_of_day(&self, date: String) -> Result<JsValue, JsError> {
            self.with_store(|s| to_js(&search::word_of_day(&s.all(), &date).map_err(js_err)?))
        }

        /// Version of the bundled SQLite (diagnostics).
        #[wasm_bindgen(js_name = sqliteVersion)]
        pub async fn sqlite_version(&self) -> Result<String, JsError> {
            let c = crate::sql::Conn::open_memory().map_err(js_err)?;
            let rows = c
                .query("SELECT sqlite_version(), (SELECT group_concat(compile_options) FROM pragma_compile_options WHERE compile_options LIKE 'ENABLE_FTS%')", &[])
                .map_err(js_err)?;
            Ok(format!("{} [{}]", rows[0].string(0), rows[0].string(1)))
        }

        /// Deconjugation candidates for a word (diagnostics / UI hints): `{lemma, rule}[]`.
        pub async fn deconjugate(&self, word: String) -> Result<JsValue, JsError> {
            to_js(&crate::deconjugate::deconjugate(&word))
        }
    }

    impl Default for Engine {
        fn default() -> Self {
            Engine::new()
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm_api::Engine;
