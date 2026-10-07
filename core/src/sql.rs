//! A tiny SQLite connection abstraction with two backends:
//!
//! * native (tests, tooling): `rusqlite` with the same bundled SQLite as the pipeline;
//! * `wasm32-unknown-unknown`: raw FFI to `sqlite-wasm-rs` (SQLite 3.5x compiled to wasm with FTS5),
//!   opened on the OPFS sahpool VFS or in memory.
//!
//! Only what the engine needs is exposed: run a statement with positional parameters and collect
//! all rows. All search logic is written against this type so it can be tested natively.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    Null,
    Int(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl From<i64> for Val {
    fn from(v: i64) -> Self {
        Val::Int(v)
    }
}
impl From<i32> for Val {
    fn from(v: i32) -> Self {
        Val::Int(v as i64)
    }
}
impl From<usize> for Val {
    fn from(v: usize) -> Self {
        Val::Int(v as i64)
    }
}
impl From<&str> for Val {
    fn from(v: &str) -> Self {
        Val::Text(v.to_string())
    }
}
impl From<String> for Val {
    fn from(v: String) -> Self {
        Val::Text(v)
    }
}
impl<T: Into<Val>> From<Option<T>> for Val {
    fn from(v: Option<T>) -> Self {
        v.map_or(Val::Null, Into::into)
    }
}

/// One result row.
#[derive(Debug, Clone, Default)]
pub struct Row(pub Vec<Val>);

impl Row {
    pub fn int(&self, i: usize) -> Option<i64> {
        match self.0.get(i)? {
            Val::Int(v) => Some(*v),
            Val::Real(v) => Some(*v as i64),
            Val::Text(s) => s.parse().ok(),
            _ => None,
        }
    }
    pub fn real(&self, i: usize) -> Option<f64> {
        match self.0.get(i)? {
            Val::Int(v) => Some(*v as f64),
            Val::Real(v) => Some(*v),
            _ => None,
        }
    }
    pub fn text(&self, i: usize) -> Option<String> {
        match self.0.get(i)? {
            Val::Text(s) => Some(s.clone()),
            Val::Int(v) => Some(v.to_string()),
            Val::Real(v) => Some(v.to_string()),
            _ => None,
        }
    }
    /// Text or empty string.
    pub fn string(&self, i: usize) -> String {
        self.text(i).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct SqlError(pub String);

impl fmt::Display for SqlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for SqlError {}

pub type Result<T> = std::result::Result<T, SqlError>;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use rusqlite::types::ValueRef;

    pub struct Conn {
        db: rusqlite::Connection,
    }

    fn err(e: rusqlite::Error) -> SqlError {
        SqlError(e.to_string())
    }

    impl Conn {
        pub fn open_memory() -> Result<Conn> {
            Ok(Conn { db: rusqlite::Connection::open_in_memory().map_err(err)? })
        }

        pub fn open_path(path: &str, read_only: bool) -> Result<Conn> {
            use rusqlite::OpenFlags;
            let flags = if read_only {
                OpenFlags::SQLITE_OPEN_READ_ONLY
            } else {
                OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
            };
            Ok(Conn { db: rusqlite::Connection::open_with_flags(path, flags).map_err(err)? })
        }

        pub fn exec(&self, sql: &str) -> Result<()> {
            self.db.execute_batch(sql).map_err(err)
        }

        /// Pager cache misses (pages read from the file) since the last reset; the native
        /// proxy for the number of OPFS reads a query costs in the browser.
        pub fn cache_misses(&self, reset: bool) -> i64 {
            let (mut cur, mut hi) = (0i32, 0i32);
            unsafe {
                // SQLITE_DBSTATUS_CACHE_MISS = 8
                rusqlite::ffi::sqlite3_db_status(self.db.handle(), 8, &mut cur, &mut hi, reset as i32);
            }
            cur as i64
        }

        pub fn query(&self, sql: &str, params: &[Val]) -> Result<Vec<Row>> {
            let mut stmt = self.db.prepare_cached(sql).map_err(err)?;
            let ncol = stmt.column_count();
            let bound: Vec<rusqlite::types::Value> = params
                .iter()
                .map(|p| match p {
                    Val::Null => rusqlite::types::Value::Null,
                    Val::Int(v) => rusqlite::types::Value::Integer(*v),
                    Val::Real(v) => rusqlite::types::Value::Real(*v),
                    Val::Text(s) => rusqlite::types::Value::Text(s.clone()),
                    Val::Blob(b) => rusqlite::types::Value::Blob(b.clone()),
                })
                .collect();
            let mut rows = stmt.query(rusqlite::params_from_iter(bound)).map_err(err)?;
            let mut out = Vec::new();
            while let Some(r) = rows.next().map_err(err)? {
                let mut vals = Vec::with_capacity(ncol);
                for i in 0..ncol {
                    vals.push(match r.get_ref(i).map_err(err)? {
                        ValueRef::Null => Val::Null,
                        ValueRef::Integer(v) => Val::Int(v),
                        ValueRef::Real(v) => Val::Real(v),
                        ValueRef::Text(t) => Val::Text(String::from_utf8_lossy(t).into_owned()),
                        ValueRef::Blob(b) => Val::Blob(b.to_vec()),
                    });
                }
                out.push(Row(vals));
            }
            Ok(out)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::Conn;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use sqlite_wasm_rs as ffi;
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_int, c_void};
    use std::ptr;

    pub struct Conn {
        db: *mut ffi::sqlite3,
    }

    impl Drop for Conn {
        fn drop(&mut self) {
            unsafe {
                ffi::sqlite3_close(self.db);
            }
        }
    }

    fn errmsg(db: *mut ffi::sqlite3, code: c_int) -> SqlError {
        let msg = unsafe {
            if db.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ffi::sqlite3_errmsg(db)).to_string_lossy().into_owned()
            }
        };
        SqlError(format!("sqlite error {code}: {msg}"))
    }

    impl Conn {
        /// Open `name` on the given VFS (None = default in-memory VFS).
        pub fn open(name: &str, vfs: Option<&str>, read_only: bool) -> Result<Conn> {
            let cname = CString::new(name).map_err(|e| SqlError(e.to_string()))?;
            let cvfs = match vfs {
                Some(v) => Some(CString::new(v).map_err(|e| SqlError(e.to_string()))?),
                None => None,
            };
            let flags = if read_only {
                ffi::SQLITE_OPEN_READONLY
            } else {
                ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE
            };
            let mut db: *mut ffi::sqlite3 = ptr::null_mut();
            let rc = unsafe {
                ffi::sqlite3_open_v2(
                    cname.as_ptr() as *const c_char,
                    &mut db,
                    flags,
                    cvfs.as_ref().map_or(ptr::null(), |c| c.as_ptr() as *const c_char),
                )
            };
            if rc != ffi::SQLITE_OK {
                let e = errmsg(db, rc);
                unsafe { ffi::sqlite3_close(db) };
                return Err(e);
            }
            Ok(Conn { db })
        }

        pub fn open_memory() -> Result<Conn> {
            Conn::open(":memory:", None, false)
        }

        pub fn exec(&self, sql: &str) -> Result<()> {
            let c = CString::new(sql).map_err(|e| SqlError(e.to_string()))?;
            let mut msg: *mut c_char = ptr::null_mut();
            let rc = unsafe { ffi::sqlite3_exec(self.db, c.as_ptr() as *const c_char, None, ptr::null_mut(), &mut msg) };
            if rc != ffi::SQLITE_OK {
                let text = unsafe {
                    if msg.is_null() {
                        String::new()
                    } else {
                        let s = CStr::from_ptr(msg).to_string_lossy().into_owned();
                        ffi::sqlite3_free(msg as *mut c_void);
                        s
                    }
                };
                return Err(SqlError(format!("sqlite error {rc}: {text}")));
            }
            Ok(())
        }

        pub fn query(&self, sql: &str, params: &[Val]) -> Result<Vec<Row>> {
            let c = CString::new(sql).map_err(|e| SqlError(e.to_string()))?;
            let mut stmt: *mut ffi::sqlite3_stmt = ptr::null_mut();
            let rc = unsafe {
                ffi::sqlite3_prepare_v2(self.db, c.as_ptr() as *const c_char, -1, &mut stmt, ptr::null_mut())
            };
            if rc != ffi::SQLITE_OK {
                return Err(errmsg(self.db, rc));
            }
            struct Guard(*mut ffi::sqlite3_stmt);
            impl Drop for Guard {
                fn drop(&mut self) {
                    unsafe {
                        ffi::sqlite3_finalize(self.0);
                    }
                }
            }
            let _g = Guard(stmt);
            for (i, p) in params.iter().enumerate() {
                let idx = (i + 1) as c_int;
                let rc = unsafe {
                    match p {
                        Val::Null => ffi::sqlite3_bind_null(stmt, idx),
                        Val::Int(v) => ffi::sqlite3_bind_int64(stmt, idx, *v),
                        Val::Real(v) => ffi::sqlite3_bind_double(stmt, idx, *v),
                        Val::Text(s) => ffi::sqlite3_bind_text(
                            stmt,
                            idx,
                            s.as_ptr() as *const c_char,
                            s.len() as c_int,
                            ffi::SQLITE_TRANSIENT(),
                        ),
                        Val::Blob(b) => ffi::sqlite3_bind_blob(
                            stmt,
                            idx,
                            b.as_ptr() as *const c_void,
                            b.len() as c_int,
                            ffi::SQLITE_TRANSIENT(),
                        ),
                    }
                };
                if rc != ffi::SQLITE_OK {
                    return Err(errmsg(self.db, rc));
                }
            }
            let ncol = unsafe { ffi::sqlite3_column_count(stmt) } as usize;
            let mut out = Vec::new();
            loop {
                let rc = unsafe { ffi::sqlite3_step(stmt) };
                if rc == ffi::SQLITE_DONE {
                    break;
                }
                if rc != ffi::SQLITE_ROW {
                    return Err(errmsg(self.db, rc));
                }
                let mut vals = Vec::with_capacity(ncol);
                for i in 0..ncol {
                    let i = i as c_int;
                    let v = unsafe {
                        match ffi::sqlite3_column_type(stmt, i) {
                            ffi::SQLITE_INTEGER => Val::Int(ffi::sqlite3_column_int64(stmt, i)),
                            ffi::SQLITE_FLOAT => Val::Real(ffi::sqlite3_column_double(stmt, i)),
                            ffi::SQLITE_TEXT => {
                                let p = ffi::sqlite3_column_text(stmt, i);
                                let n = ffi::sqlite3_column_bytes(stmt, i) as usize;
                                if p.is_null() {
                                    Val::Text(String::new())
                                } else {
                                    Val::Text(String::from_utf8_lossy(std::slice::from_raw_parts(p, n)).into_owned())
                                }
                            }
                            ffi::SQLITE_BLOB => {
                                let p = ffi::sqlite3_column_blob(stmt, i) as *const u8;
                                let n = ffi::sqlite3_column_bytes(stmt, i) as usize;
                                if p.is_null() {
                                    Val::Blob(Vec::new())
                                } else {
                                    Val::Blob(std::slice::from_raw_parts(p, n).to_vec())
                                }
                            }
                            _ => Val::Null,
                        }
                    };
                    vals.push(v);
                }
                out.push(Row(vals));
            }
            Ok(out)
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::Conn;
