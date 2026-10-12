//! 국사편찬위원회 한국사데이터베이스 (db.history.go.kr): the original-language text of public-domain
//! documents (매천야록, 한국독립운동사자료, 동학농민혁명자료총서, 유길준 자료 …).
//!
//! Only the original text is taken: never the DB's 국역 (translation) items (`…r` ids), its
//! editorial headings, page markers or source-citation lines. Leaf page:
//! `/diachronic/level.do?levelId=<id>` → `#section-read .txt-wrap` blocks, one paragraph each.

use crate::texts_html::{element_by_token, elements_by_class_with_attrs, html_to_text};
use crate::texts_wikitext::remove_element;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Hanyang PUA (한양 PUA, the old-Hangul private-use encoding used by the DB) → Unicode.
fn pua_table() -> &'static HashMap<char, String> {
    static T: OnceLock<HashMap<char, String>> = OnceLock::new();
    T.get_or_init(|| {
        include_str!("../data/hanyang_pua.tsv")
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .filter_map(|l| {
                let (a, b) = l.split_once('\t')?;
                let k = char::from_u32(u32::from_str_radix(a, 16).ok()?)?;
                let v: String = b.split_whitespace().filter_map(|x| u32::from_str_radix(x, 16).ok().and_then(char::from_u32)).collect();
                Some((k, v))
            })
            .collect()
    })
}

/// Replace Hanyang-PUA code points with standard old-Hangul jamo sequences.
pub fn from_hanyang_pua(s: &str) -> String {
    let t = pua_table();
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match t.get(&c) {
            Some(v) => out.push_str(v),
            None => out.push(c),
        }
    }
    out
}

pub const BASE: &str = "https://db.history.go.kr";

pub fn leaf_url(level_id: &str) -> String {
    format!("{BASE}/diachronic/level.do?levelId={level_id}")
}

/// Is `id` a translation item (`sa_001r_…`)? Those are 국역 — never fetched.
pub fn is_translation(id: &str) -> bool {
    id.split('_').any(|p| p.len() > 1 && p.ends_with('r') && p[..p.len() - 1].chars().all(|c| c.is_ascii_digit()))
}

/// A source-citation line such as "韓國獨立運動文類 第一集 二〇-二一面" or "… 第一輯 九-一四面".
fn is_citation(p: &str) -> bool {
    let t = p.trim();
    t.chars().count() < 60 && t.ends_with('面') && (t.contains('第') || t.contains('輯') || t.contains('集'))
}

/// A page marker "<406>" / "＜12-18＞" left as its own line.
fn is_page_marker(line: &str) -> bool {
    strip_page_markers(line).trim().is_empty() && !line.trim().is_empty()
}

/// Remove page markers "<406>", "＜9＞", "＜12-18＞" wherever they occur.
fn strip_page_markers(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        if c == '<' || c == '＜' {
            let close = if c == '<' { '>' } else { '＞' };
            let rest = &s[i + c.len_utf8()..];
            if let Some(end) = rest.find(close) {
                let inner = &rest[..end];
                if !inner.is_empty() && inner.len() <= 12 && inner.chars().all(|x| x.is_ascii_digit() || x == '-') {
                    let skip = inner.chars().count() + 1;
                    for _ in 0..skip {
                        it.next();
                    }
                    continue;
                }
            }
        }
        out.push(c);
    }
    out
}

/// Cut one document out of a longer text: from the first line containing `from` (inclusive) to
/// the first later line containing `to` (exclusive). `None` when `from` is not found.
pub fn cut(text: &str, from: Option<&str>, to: Option<&str>) -> Option<String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let a = match from {
        Some(f) => lines.iter().position(|l| l.contains(f))?,
        None => 0,
    };
    let b = to.and_then(|t| lines[a + 1..].iter().position(|l| l.contains(t)).map(|k| a + 1 + k)).unwrap_or(lines.len());
    Some(lines[a..b].join("\n").trim().to_string())
}

/// Paragraphs of original text in a leaf page (empty when the page has no reading section).
pub fn extract(html: &str) -> Vec<String> {
    let Some(read) = element_by_token(html, &["section-read"]) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (attrs, block) in elements_by_class_with_attrs(&read, "txt-wrap") {
        let low = block.to_ascii_lowercase();
        let indented = attrs.replace(' ', "").contains("padding-left:15px");
        // editorial sub-headings are set in the DB's blue
        if low.contains("#4863a0") {
            continue;
        }
        // the DB's own item title (<strong>…</strong> alone in a block) is not part of the text
        let bare = html_to_text(&remove_element(&block, "strong"));
        if low.contains("<strong") && bare.trim().is_empty() {
            continue;
        }
        // <h6> holds the printed source citation ("韓國獨立運動文類 第一集 二◯-二一面")
        let block = remove_element(&block, "h6");
        let text = strip_page_markers(&from_hanyang_pua(&html_to_text(&block)));
        let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty() && !is_page_marker(l)).collect();
        // a page marker or <br> inside a sentence is not a paragraph break; unspaced
        // manuscript text (東學史 草稿本) is joined without inventing spaces
        let chars: usize = lines.iter().map(|l| l.chars().count()).sum();
        let spaces: usize = lines.iter().map(|l| l.chars().filter(|c| *c == ' ').count()).sum();
        let sep = if chars > 0 && spaces * 50 < chars { "" } else { " " };
        let para = lines.join(sep);
        if para.is_empty() || (out.is_empty() && is_citation(&para)) {
            continue;
        }
        // a short indented first block is the DB editor's title for the item (倡義文「內譯」)
        if out.is_empty() && indented && para.chars().count() < 30 {
            continue;
        }
        out.push(para);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_original_drops_citation_markers_and_editorial_headings() {
        let html = r#"<html><body><section class="section-top"><div class="title">2. 京軍의 해산</div></section>
<section class="section-read" id="section-read">
<div class="txt-wrap">韓國獨立運動文類 第一集 二〇-二一面</div>
<div class="txt-wrap"><span style="color: #4863a0;">國債報償斷煙會의 創設</span></div>
<div class="txt-wrap">京軍解散 七賊等, 恐<span title="이름">軍</span>情激變。<div align="center"><small>&lt;406&gt;</small></div>二十三日, 招各隊長…</div>
<div class="txt-wrap"><small>七月以後爲隆熙元年</small></div>
</section><section class="section-remark" id="section-remark">편집자 주</section></body></html>"#;
        let p = extract(html);
        assert_eq!(p.len(), 2, "{p:?}");
        assert!(p[0].starts_with("京軍解散 七賊等, 恐軍情激變。"));
        assert!(!p[0].contains("406"));
        assert!(p[0].contains("二十三日"));
        assert_eq!(p[1], "七月以後爲隆熙元年");
        assert!(extract("<html>no reading section</html>").is_empty());
        let kd = r#"<section id="section-read"><div class="txt-wrap"><strong>5. 臨時政府 對日宣戰聲明書</strong></div>
<div class="txt-wrap"><h6>韓國獨立運動文類 第一集 二◯-二一面</h6> 大韓民國臨時政府<span title="기타">對日宣戰聲明書</span></div>
<div class="txt-wrap"> 吾人代表三千萬韓人</div></section>"#;
        assert_eq!(extract(kd), vec!["大韓民國臨時政府對日宣戰聲明書", "吾人代表三千萬韓人"]);
        let prd = "<section id=\"section-read\"><div class=\"txt-wrap\" style=\"padding-left: 15px;\">倡義文「內譯」</div><div class=\"txt-wrap\" style=\"padding-left: 0px;\">世上에서사람을貴타함은<br/>人倫이라\u{ea7f}문이다</div></section>";
        let p = extract(prd);
        assert_eq!(p.len(), 1);
        assert!(p[0].starts_with("世上에서사람을貴타함은人倫이라"), "{p:?}");
        assert!(!p[0].chars().any(|c| ('\u{e000}'..='\u{f8ff}').contains(&c)), "{p:?}");
    }

    #[test]
    fn page_markers_and_cuts() {
        assert_eq!(strip_page_markers("天下＜9＞之事<406>也"), "天下之事也");
        assert!(is_page_marker("＜12-18＞"));
        assert!(!is_page_marker("<b>"));
        let t = "會報\n\n大韓自强會趣旨書\n\n本文\n\n大韓自强會 規則\n\n第一條";
        assert_eq!(cut(t, Some("趣旨書"), Some("規則")).unwrap(), "大韓自强會趣旨書\n\n本文");
        assert!(cut(t, Some("없음"), None).is_none());
    }

    #[test]
    fn translation_items_are_recognised() {
        assert!(is_translation("sa_001r_0050_0090_0270"));
        assert!(!is_translation("sa_001_0050_0090_0270"));
        assert!(!is_translation("kd_002_0010_0010_0110"));
    }
}
