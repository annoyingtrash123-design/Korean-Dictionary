//! 국사편찬위원회 한국사데이터베이스 (db.history.go.kr): the original-language text of public-domain
//! documents (매천야록, 한국독립운동사자료, 동학농민혁명자료총서, 유길준 자료 …).
//!
//! Only the original text is taken: never the DB's 국역 (translation) items (`…r` ids), its
//! editorial headings, page markers or source-citation lines. Leaf page:
//! `/diachronic/level.do?levelId=<id>` → `#section-read .txt-wrap` blocks, one paragraph each.

use crate::texts_html::{element_by_token, elements_by_class, html_to_text};

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

/// A page marker "<406>" left as its own line.
fn is_page_marker(line: &str) -> bool {
    let t = line.trim();
    t.starts_with('<') && t.ends_with('>') && t[1..t.len() - 1].chars().all(|c| c.is_ascii_digit()) && t.len() > 2
}

/// Paragraphs of original text in a leaf page (empty when the page has no reading section).
pub fn extract(html: &str) -> Vec<String> {
    let Some(read) = element_by_token(html, &["section-read"]) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for block in elements_by_class(&read, "txt-wrap") {
        // editorial sub-headings are set in the DB's blue
        if block.to_ascii_lowercase().contains("#4863a0") {
            continue;
        }
        let text = html_to_text(&block);
        let lines: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty() && !is_page_marker(l)).collect();
        // a page marker or <br> inside a sentence is not a paragraph break
        let para = lines.join(" ");
        if para.is_empty() || (out.is_empty() && is_citation(&para)) {
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
    }

    #[test]
    fn translation_items_are_recognised() {
        assert!(is_translation("sa_001r_0050_0090_0270"));
        assert!(!is_translation("sa_001_0050_0090_0270"));
        assert!(!is_translation("kd_002_0010_0010_0110"));
    }
}
