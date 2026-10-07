//! Pack file naming and checksum helpers shared by the browser storage layer (`db`, `import`)
//! and testable natively.
//!
//! The sahpool VFS publishes an import atomically under a *new* name and has no rename, so each
//! pack owns two alternating file slots, `<id>.sqlite3` (A) and `<id>.b.sqlite3` (B). A new
//! import is written into the slot that is not live; the install record (`installed.file`) is
//! switched to it only after it has been fully written, checksummed and opened. Until then the
//! old pack stays installed and usable, and a failed or aborted import leaves it untouched.

pub const EXT: &str = ".sqlite3";

pub const CORRUPT_MSG: &str = "Downloaded data is corrupt (checksum mismatch) — please retry";

/// `core`, `stdict`, ... (lower-case ascii, digits, `_`, `-`).
pub fn valid_pack_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 32
        && id.as_bytes()[0].is_ascii_alphanumeric()
        && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// File name of slot 0 (`<id>.sqlite3`) or 1 (`<id>.b.sqlite3`).
pub fn slot_file(pack: &str, slot: u8) -> String {
    if slot == 0 {
        format!("{pack}{EXT}")
    } else {
        format!("{pack}.b{EXT}")
    }
}

/// `(pack id, slot)` of a pool file name, or `None` for anything that is not a pack file.
pub fn parse_file(name: &str) -> Option<(String, u8)> {
    let stem = name.strip_suffix(EXT)?;
    let (id, slot) = match stem.strip_suffix(".b") {
        Some(id) => (id, 1),
        None => (stem, 0),
    };
    valid_pack_id(id).then(|| (id.to_string(), slot))
}

/// The slot an import of `pack` should be written to, given the file that is currently live.
pub fn import_slot(pack: &str, live_file: Option<&str>) -> u8 {
    match live_file.and_then(parse_file) {
        Some((id, 0)) if id == pack => 1,
        _ => 0,
    }
}

/// Lower-case hex of a digest.
pub fn hex(digest: &[u8]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Compare the computed digest with the manifest's `sha256` (absent / empty = not checked).
pub fn verify_sha256(expected: Option<&str>, digest: &[u8]) -> Result<(), String> {
    match expected.map(str::trim).filter(|s| !s.is_empty()) {
        Some(want) if !want.eq_ignore_ascii_case(&hex(digest)) => Err(CORRUPT_MSG.to_string()),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn slots_alternate_and_parse() {
        assert_eq!(slot_file("core", 0), "core.sqlite3");
        assert_eq!(slot_file("core", 1), "core.b.sqlite3");
        assert_eq!(parse_file("core.sqlite3"), Some(("core".into(), 0)));
        assert_eq!(parse_file("core.b.sqlite3"), Some(("core".into(), 1)));
        assert_eq!(parse_file("_installed.sqlite3"), None);
        assert_eq!(parse_file("core.sqlite3-journal"), None);
        // nothing live: slot A; live A: B; live B: A; unrelated live file: A
        assert_eq!(import_slot("core", None), 0);
        assert_eq!(import_slot("core", Some("core.sqlite3")), 1);
        assert_eq!(import_slot("core", Some("core.b.sqlite3")), 0);
        assert_eq!(import_slot("core", Some("stdict.sqlite3")), 0);
        // the new slot never equals the live file
        for live in [slot_file("core", 0), slot_file("core", 1)] {
            assert_ne!(slot_file("core", import_slot("core", Some(&live))), live);
        }
    }

    #[test]
    fn incremental_sha_matches_and_mismatch_is_rejected() {
        let data: Vec<u8> = (0..100_000u32).map(|i| (i * 7 % 251) as u8).collect();
        let mut h = Sha256::new();
        for part in data.chunks(777) {
            h.update(part);
        }
        let digest = h.finalize();
        let want = hex(&digest);
        assert_eq!(want, hex(&Sha256::digest(&data)));
        assert!(verify_sha256(Some(&want), &digest).is_ok());
        assert!(verify_sha256(Some(&want.to_uppercase()), &digest).is_ok());
        assert!(verify_sha256(None, &digest).is_ok());
        assert!(verify_sha256(Some(""), &digest).is_ok());
        let mut bad = want.clone();
        bad.replace_range(0..1, if want.starts_with('0') { "1" } else { "0" });
        assert_eq!(verify_sha256(Some(&bad), &digest).unwrap_err(), CORRUPT_MSG);
    }
}
