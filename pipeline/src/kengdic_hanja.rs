//! Reviewed corrections to the hanja kengdic itself gives (`data/kengdic_hanja.tsv`). kengdic sometimes merges two
//! words that share a spelling under one hanja (군도 群島 "archipelago; saber": saber is 軍刀) or attaches the wrong
//! homograph's hanja (황천 荒天 "hades": 黃泉). Each kengdic entry whose English mixes meanings while 표준국어대사전 has
//! other words of that spelling was checked independently against stdict's entries: `fix` gives the hanja of the one
//! stdict word all its English belongs to, `drop` removes the hanja (owner rule "if in doubt, no hanja").
//! A row applies only while the entry still has that headword, hanja and English (an upstream change retires it);
//! the entry gets `"hanja_fixed": true`, so the stdict hanja fill leaves it alone.

use anyhow::Result;
use rusqlite::{params, Connection};

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub headword: String,
    pub hanja: String,
    /// the entry's `gloss` column, exactly
    pub gloss: String,
    /// `Some(h)`: replace with h; `None`: remove the hanja
    pub new: Option<String>,
}

pub fn parse(tsv: &str) -> Vec<Row> {
    tsv.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty() && !l.starts_with("headword\t"))
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            match f[..] {
                [hw, h, gloss, "fix", new, _] if !new.is_empty() => Some(Row { headword: hw.into(), hanja: h.into(), gloss: gloss.into(), new: Some(new.into()) }),
                [hw, h, gloss, "drop", _, _] => Some(Row { headword: hw.into(), hanja: h.into(), gloss: gloss.into(), new: None }),
                _ => None,
            }
        })
        .collect()
}

pub fn builtin() -> Vec<Row> {
    parse(include_str!("../data/kengdic_hanja.tsv"))
}

/// Apply to the core pack's kengdic entries; returns how many entries changed.
pub fn apply(core: &Connection, rows: &[Row]) -> Result<usize> {
    let mut n = 0;
    for r in rows {
        let ids: Vec<i64> = {
            let mut q = core.prepare_cached("SELECT id FROM entries WHERE source = 'kengdic' AND headword = ?1 AND hanja = ?2 AND gloss = ?3")?;
            let v = q.query_map(params![r.headword, r.hanja, r.gloss], |x| x.get(0))?.collect::<rusqlite::Result<Vec<i64>>>()?;
            v
        };
        for id in ids {
            crate::hanja_fill::replace_hanja(core, id, r.new.as_deref(), "hanja_fixed")?;
            n += 1;
        }
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fix_and_drop_and_skips_keep() {
        let r = parse("headword\thanja\tgloss\tverdict\tfixed_hanja\treason\n황천\t荒天\tweather; hades\tdrop\t\tmixed\n전뇌\t電腦\tforebrain\tfix\t前腦\tbrain part\n격막\t隔膜\tseptum\tkeep\t\tok\n");
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].new, None);
        assert_eq!(r[1].new.as_deref(), Some("前腦"));
    }

    #[test]
    fn builtin_parses() {
        let rows = include_str!("../data/kengdic_hanja.tsv").lines().filter(|l| !l.starts_with('#') && !l.starts_with("headword\t") && !l.trim().is_empty()).filter(|l| !l.contains("\tkeep\t")).count();
        assert_eq!(builtin().len(), rows, "every fix/drop row parses");
    }
}
