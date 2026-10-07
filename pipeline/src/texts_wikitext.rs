//! Wikitext -> plain text for the Reader texts.
//!
//! Output conventions: paragraphs are separated by a blank line (`"\n\n"`), section headings are
//! lines starting with `"## "`, and verse keeps its line breaks (`Layout::Verse`, `<poem>` and
//! `<br>` always do). Templates, references, categories, files, navigation boxes and the
//! reference/see-also sections are dropped.

use serde_json::{Map, Value};

/// Private-use marker for a *kept* line break; turned into `\n` at the very end.
const LB: char = '\u{E000}';

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layout {
    /// Consecutive non-blank lines form one paragraph (as MediaWiki renders them).
    Prose,
    /// Every source line is kept as its own line; blank lines separate stanzas.
    Verse,
}

#[derive(Debug, Clone, Default)]
pub struct Cleaned {
    pub text: String,
    /// Source/edition info taken from the page header template (and `<pages index=…>`), if any.
    pub edition: Option<Value>,
}

/// `nospace`: join the lines of a prose paragraph without a space (hanmun).
pub fn clean(wikitext: &str, layout: Layout, nospace: bool) -> Cleaned {
    let s = strip_comments(wikitext);
    let s = s.replace("\r\n", "\n");
    let edition = extract_edition(&s);
    let mut s = s;
    for el in ["noinclude", "ref", "gallery", "references", "pages", "section", "math", "templatedata", "rt", "rp", "syntaxhighlight", "timeline", "imagemap", "score"] {
        s = remove_element(&s, el);
    }
    let s = process_poems(&s);
    let s = process_templates(&s, 0);
    let s = process_tables(&s);
    let text = assemble(&s, layout, nospace);
    Cleaned { text, edition }
}

// ---------------------------------------------------------------- low-level helpers

fn strip_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(p) = s[i..].find("<!--") {
        out.push_str(&s[i..i + p]);
        match s[i + p..].find("-->") {
            Some(e) => i = i + p + e + 3,
            None => {
                i = s.len();
                break;
            }
        }
    }
    out.push_str(&s[i..]);
    out
}

/// Remove `<name …>…</name>` and `<name …/>` (ASCII case-insensitive).
pub fn remove_element(s: &str, name: &str) -> String {
    let lower = s.to_ascii_lowercase();
    let open = format!("<{name}");
    let close = format!("</{name}>");
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(p) = lower[i..].find(&open).map(|x| x + i) {
        let after = lower.as_bytes().get(p + open.len()).copied();
        if !matches!(after, Some(b' ') | Some(b'>') | Some(b'/') | Some(b'\n') | Some(b'\t')) {
            out.push_str(&s[i..p + open.len()]);
            i = p + open.len();
            continue;
        }
        out.push_str(&s[i..p]);
        let Some(gt) = lower[p..].find('>').map(|x| x + p) else {
            i = s.len();
            break;
        };
        if lower[..gt].ends_with('/') {
            i = gt + 1;
            continue;
        }
        match lower[gt..].find(&close) {
            Some(c) => i = gt + c + close.len(),
            None => {
                i = s.len();
                break;
            }
        }
    }
    out.push_str(&s[i..]);
    out
}

/// `<poem>…</poem>`: keep the line breaks inside stanzas.
fn process_poems(s: &str) -> String {
    let lower = s.to_ascii_lowercase();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(p) = lower[i..].find("<poem").map(|x| x + i) {
        out.push_str(&s[i..p]);
        let Some(gt) = lower[p..].find('>').map(|x| x + p) else {
            i = p;
            break;
        };
        let (inner_end, next) = match lower[gt..].find("</poem>") {
            Some(c) => (gt + c, gt + c + 7),
            None => (s.len(), s.len()),
        };
        out.push('\n');
        out.push_str(&keep_line_breaks(&s[gt + 1..inner_end]));
        out.push('\n');
        i = next;
    }
    out.push_str(&s[i..]);
    out
}

fn keep_line_breaks(inner: &str) -> String {
    let lines: Vec<&str> = inner.trim_matches('\n').split('\n').collect();
    let mut out = String::new();
    for (n, l) in lines.iter().enumerate() {
        out.push_str(l.trim_end());
        if let Some(next) = lines.get(n + 1) {
            if l.trim().is_empty() || next.trim().is_empty() {
                out.push('\n');
            } else {
                out.push(LB);
            }
        }
    }
    out
}

/// Index just past the `}}` matching the `{{` at `i`.
fn match_braces(s: &str, i: usize) -> Option<usize> {
    let b = s.as_bytes();
    let mut depth = 0i32;
    let mut j = i;
    while j + 1 < b.len() {
        if b[j] == b'{' && b[j + 1] == b'{' {
            depth += 1;
            j += 2;
        } else if b[j] == b'}' && b[j + 1] == b'}' {
            depth -= 1;
            j += 2;
            if depth == 0 {
                return Some(j);
            }
        } else {
            j += 1;
        }
    }
    None
}

/// Split at top-level `sep` (not inside `{{ }}` or `[[ ]]`).
fn split_top(s: &str, sep: u8) -> Vec<&str> {
    let b = s.as_bytes();
    let (mut parts, mut start, mut depth, mut j) = (Vec::new(), 0usize, 0i32, 0usize);
    while j < b.len() {
        let two = b.get(j + 1).copied();
        match (b[j], two) {
            (b'{', Some(b'{')) | (b'[', Some(b'[')) => {
                depth += 1;
                j += 2;
                continue;
            }
            (b'}', Some(b'}')) | (b']', Some(b']')) => {
                depth -= 1;
                j += 2;
                continue;
            }
            (c, _) if c == sep && depth <= 0 => {
                parts.push(&s[start..j]);
                start = j + 1;
            }
            _ => {}
        }
        j += 1;
    }
    parts.push(&s[start..]);
    parts
}

fn is_named_param(p: &str) -> Option<(&str, &str)> {
    let eq = p.find('=')?;
    let k = p[..eq].trim();
    if !k.is_empty() && k.chars().count() <= 24 && !k.contains(['[', '{', '<']) {
        Some((k, p[eq + 1..].trim()))
    } else {
        None
    }
}

// ---------------------------------------------------------------- templates

const KEEP_FIRST: &[&str] = &[
    "center", "centre", "가운데", "중앙", "larger", "smaller", "큰글씨", "작은글씨", "nowrap", "bold", "굵게", "small", "big", "한자", "hanja", "ruby", "루비", "underline", "밑줄", "u", "indent", "들여쓰기", "right", "오른쪽", "block center", "sc", "ib", "overstrike", "strike", "blockquote", "인용문", "quote", "저자:", "텍스트", "글",
];
const KEEP_POEM: &[&str] = &["poem", "시", "verse", "운문", "시구"];
const KEEP_LAST: &[&str] = &["lang", "언어", "font", "폰트", "linktext", "rubi"];

fn process_templates(s: &str, depth: usize) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        if rest.starts_with("{{") {
            if let Some(end) = match_braces(s, i) {
                let inner = &s[i + 2..end - 2];
                if depth < 24 {
                    out.push_str(&expand_template(inner, depth));
                }
                i = end;
                continue;
            }
            i += 2;
            continue;
        }
        let ch = rest.chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn expand_template(inner: &str, depth: usize) -> String {
    let parts = split_top(inner, b'|');
    let name = parts[0].trim().to_lowercase();
    let name = name.trim_start_matches("subst:").trim_start_matches("safesubst:").trim().to_string();
    if name.starts_with('#') || name.contains(':') && !name.ends_with(':') {
        return String::new();
    }
    let mut positional = Vec::new();
    let mut named = Vec::new();
    for p in &parts[1..] {
        match is_named_param(p) {
            Some((k, v)) => named.push((k.to_lowercase(), v.to_string())),
            None => positional.push(p.trim().to_string()),
        }
    }
    let pick_named = || named.iter().find(|(k, _)| matches!(k.as_str(), "1" | "text" | "content" | "내용")).map(|(_, v)| v.clone());
    let arg = if KEEP_FIRST.contains(&name.as_str()) || KEEP_POEM.contains(&name.as_str()) {
        positional.first().cloned().or_else(pick_named)
    } else if KEEP_LAST.contains(&name.as_str()) {
        positional.last().cloned().or_else(pick_named)
    } else {
        None
    };
    let Some(arg) = arg else { return String::new() };
    let mut v = process_templates(&arg, depth + 1);
    if KEEP_POEM.contains(&name.as_str()) {
        v = format!("\n{}\n", keep_line_breaks(&v));
    }
    v
}

// ---------------------------------------------------------------- edition info

const EDITION_KEYS: &[&str] = &[
    "원문", "출처", "저자", "번역자", "편집자", "판", "판본", "source", "author", "translator", "editor", "edition", "publisher", "year", "연도", "원본", "스캔", "scan", "출판사", "발행", "발행처", "날짜", "제목", "title", "volume", "권",
];
const NAV_KEYS: &[&str] = &["previous", "next", "prev", "이전", "다음", "section", "섹션", "override_author", "override_translator", "noauthor", "notes_hidden"];
const HEADER_NAMES: &[&str] = &["header", "헤더", "제목", "textinfo", "text info", "저자 정보", "판본", "출처", "book", "도서", "고전"];

/// Pull edition/source info from the first header-like template and any scan index references.
pub fn extract_edition(s: &str) -> Option<Value> {
    let head_end = s.char_indices().nth(6000).map(|(i, _)| i).unwrap_or(s.len());
    let head = &s[..head_end];
    let mut found: Option<Map<String, Value>> = None;
    let mut i = 0;
    while let Some(p) = head[i..].find("{{").map(|x| x + i) {
        let Some(end) = match_braces(s, p) else { break };
        let inner = &s[p + 2..end - 2];
        let parts = split_top(inner, b'|');
        let name = parts[0].trim().to_string();
        let lname = name.to_lowercase();
        let mut params = Map::new();
        let mut has_key = false;
        for part in &parts[1..] {
            if let Some((k, v)) = is_named_param(part) {
                let lk = k.to_lowercase();
                if NAV_KEYS.contains(&lk.as_str()) {
                    continue;
                }
                let val = inline(&process_templates(v, 1));
                if val.is_empty() {
                    continue;
                }
                if EDITION_KEYS.contains(&lk.as_str()) {
                    has_key = true;
                }
                params.insert(k.to_string(), Value::String(val.chars().take(400).collect()));
            }
        }
        let header_like = HEADER_NAMES.iter().any(|h| lname.contains(h));
        if (header_like || has_key) && !params.is_empty() {
            let mut m = Map::new();
            m.insert("template".into(), Value::String(name));
            m.insert("params".into(), Value::Object(params));
            found = Some(m);
            break;
        }
        i = end;
    }
    // scan index references
    let mut index: Vec<String> = Vec::new();
    let lower = s.to_ascii_lowercase();
    let mut j = 0;
    while let Some(p) = lower[j..].find("<pages").map(|x| x + j) {
        let gt = lower[p..].find('>').map(|x| x + p).unwrap_or(s.len());
        let tag = &s[p..gt];
        if let Some(a) = tag.find("index=") {
            let v = tag[a + 6..].trim_start_matches(['"', '\'']);
            let v: String = v.chars().take_while(|c| !matches!(c, '"' | '\'' | ' ' | '/' | '>') || *c == '/' && false).collect();
            if !v.is_empty() && !index.contains(&v) {
                index.push(v);
            }
        }
        j = gt.max(p + 6);
    }
    for pre in ["[[Index:", "[[인덱스:", "[[index:"] {
        let mut k = 0;
        while let Some(p) = s[k..].find(pre).map(|x| x + k) {
            let end = s[p..].find([']', '|']).map(|x| x + p).unwrap_or(s.len());
            let v = s[p + 2..end].to_string();
            if !index.contains(&v) {
                index.push(v);
            }
            k = end;
        }
    }
    if found.is_none() && index.is_empty() {
        return None;
    }
    let mut m = found.unwrap_or_default();
    if !index.is_empty() {
        m.insert("index".into(), Value::Array(index.into_iter().map(Value::String).collect()));
    }
    Some(Value::Object(m))
}

// ---------------------------------------------------------------- tables

const SKIP_TABLE: &[&str] = &["navbox", "toc", "infobox", "metadata", "noprint", "navigation", "sidebar", "vertical-navbox", "header", "ambox"];

fn strip_cell_attrs(cell: &str) -> &str {
    let parts = split_top(cell, b'|');
    if parts.len() >= 2 && parts[0].contains('=') && !parts[0].contains("[[") {
        let off = parts[0].len() + 1;
        return cell[off..].trim();
    }
    cell.trim()
}

fn flush_row(row: &mut Vec<String>, out: &mut Vec<String>) {
    let cells: Vec<&str> = row.iter().map(|c| c.trim()).filter(|c| !c.is_empty()).collect();
    if !cells.is_empty() {
        out.push(cells.join(" "));
        out.push(String::new());
    }
    row.clear();
}

fn process_tables(s: &str) -> String {
    if !s.contains("{|") {
        return s.to_string();
    }
    let mut out: Vec<String> = Vec::new();
    let (mut depth, mut skipping) = (0usize, false);
    let mut row: Vec<String> = Vec::new();
    for line in s.lines() {
        let t = line.trim();
        if t.starts_with("{|") {
            depth += 1;
            if depth == 1 {
                let attrs = t[2..].to_lowercase();
                skipping = SKIP_TABLE.iter().any(|k| attrs.contains(k));
            }
            continue;
        }
        if depth == 0 {
            out.push(line.to_string());
            continue;
        }
        if t.starts_with("|}") {
            depth -= 1;
            flush_row(&mut row, &mut out);
            if depth == 0 {
                skipping = false;
            }
            continue;
        }
        if skipping {
            continue;
        }
        if t.starts_with("|-") {
            flush_row(&mut row, &mut out);
        } else if let Some(cap) = t.strip_prefix("|+") {
            flush_row(&mut row, &mut out);
            out.push(strip_cell_attrs(cap).to_string());
            out.push(String::new());
        } else if t.starts_with('|') || t.starts_with('!') {
            let body = &t[1..];
            for c in body.split("||").flat_map(|x| x.split("!!")) {
                row.push(strip_cell_attrs(c).to_string());
            }
        } else if !t.is_empty() {
            match row.last_mut() {
                Some(last) => {
                    last.push(' ');
                    last.push_str(t);
                }
                None => row.push(t.to_string()),
            }
        }
    }
    flush_row(&mut row, &mut out);
    out.join("\n")
}

// ---------------------------------------------------------------- lines, headings, inline

const DROP_SECTIONS: &[&str] = &["각주", "주석", "주", "외부 링크", "외부링크", "같이 보기", "함께 보기", "관련 문서", "관련 항목", "참고 문헌", "참고문헌", "references", "notes", "footnotes", "external links", "see also", "각주 및 참고 문헌"];
const MAGIC: &[&str] = &["__NOTOC__", "__TOC__", "__FORCETOC__", "__NOEDITSECTION__", "__NOTITLECONVERT__", "__NOCONTENTCONVERT__", "__NOINDEX__", "__INDEX__"];

enum Line {
    Blank,
    Heading(usize, String),
    Text(String),
}

fn classify(raw: &str) -> Line {
    let t = raw.trim();
    if t.is_empty() {
        return Line::Blank;
    }
    if t.len() >= 4 && t.bytes().all(|b| b == b'-') {
        return Line::Blank;
    }
    if t.len() >= 3 && t.starts_with('=') && t.ends_with('=') && !t.bytes().all(|b| b == b'=') {
        let lead = t.bytes().take_while(|b| *b == b'=').count();
        let trail = t.bytes().rev().take_while(|b| *b == b'=').count();
        let title = inline(t.trim_matches('=').trim());
        return if title.is_empty() { Line::Blank } else { Line::Heading(lead.min(trail).min(6), title) };
    }
    let t = t.trim_start_matches(['*', '#', ':', ';']).trim();
    let v = inline(t);
    if v.trim_matches(LB).trim().is_empty() {
        Line::Blank
    } else {
        Line::Text(v)
    }
}

fn assemble(s: &str, layout: Layout, nospace: bool) -> String {
    let sep: String = match layout {
        Layout::Verse => LB.to_string(),
        Layout::Prose if nospace => String::new(),
        Layout::Prose => " ".to_string(),
    };
    let mut blocks: Vec<String> = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    let mut drop_level: Option<usize> = None;
    macro_rules! flush {
        () => {
            if !cur.is_empty() {
                blocks.push(cur.join(&sep));
                cur.clear();
            }
        };
    }
    for raw in s.lines() {
        match classify(raw) {
            Line::Heading(level, title) => {
                if let Some(dl) = drop_level {
                    if level > dl {
                        continue;
                    }
                    drop_level = None;
                }
                flush!();
                if DROP_SECTIONS.contains(&title.to_lowercase().as_str()) {
                    drop_level = Some(level);
                    continue;
                }
                blocks.push(format!("## {title}"));
            }
            _ if drop_level.is_some() => {}
            Line::Blank => flush!(),
            Line::Text(t) => cur.push(t),
        }
    }
    flush!();
    let text = blocks.join("\n\n").replace(LB, "\n");
    // tidy: trim every line, at most one blank line in a row
    let mut out = String::with_capacity(text.len());
    let mut blank = 0;
    for l in text.lines() {
        let l = l.trim_matches(|c: char| c.is_whitespace());
        if l.is_empty() {
            blank += 1;
            if blank == 1 && !out.is_empty() {
                out.push('\n');
            }
        } else {
            if blank > 0 || out.is_empty() {
                // blank line(s) already emitted
            } else {
                out.push('\n');
            }
            blank = 0;
            out.push_str(l);
        }
    }
    out.trim().to_string()
}

/// Inline markup cleanup for one line: links, bold/italic, tags, entities, whitespace.
pub fn inline(t: &str) -> String {
    let mut s = t.to_string();
    for m in MAGIC {
        s = s.replace(m, "");
    }
    let s2 = clean_links(&s);
    let s2 = clean_ext_links(&s2);
    let s2 = s2.replace("'''''", "").replace("'''", "").replace("''", "");
    let s2 = strip_tags(&s2);
    let s2 = decode_entities(&s2);
    normalise_ws(&s2)
}

fn normalise_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = true;
    for c in s.chars() {
        let c = if c == '\u{a0}' || c == '\t' { ' ' } else { c };
        if c == ' ' {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            prev_space = false;
            out.push(c);
        }
    }
    out.trim_end().trim_start_matches('\u{3000}').to_string()
}

const MEDIA_PREFIXES: &[&str] = &["file:", "image:", "media:", "category:", "파일:", "그림:", "이미지:", "미디어:", "분류:", "index:", "인덱스:", "page:", "쪽:", "특수:", "special:"];
const LANG_PREFIXES: &[&str] = &["en:", "zh:", "ja:", "fr:", "de:", "ru:", "es:", "vi:", "it:", "pt:", "ar:", "la:", "ko:"];

fn clean_links(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(p) = s[i..].find("[[").map(|x| x + i) {
        out.push_str(&s[i..p]);
        // find matching ]]
        let b = s.as_bytes();
        let (mut depth, mut j, mut end) = (0i32, p, None);
        while j + 1 < b.len() {
            if b[j] == b'[' && b[j + 1] == b'[' {
                depth += 1;
                j += 2;
            } else if b[j] == b']' && b[j + 1] == b']' {
                depth -= 1;
                j += 2;
                if depth == 0 {
                    end = Some(j);
                    break;
                }
            } else {
                j += 1;
            }
        }
        let Some(end) = end else {
            out.push_str("[[");
            i = p + 2;
            continue;
        };
        let inner = &s[p + 2..end - 2];
        let parts = split_top(inner, b'|');
        let target = parts[0].trim();
        let ltarget = target.to_lowercase();
        let is_media = MEDIA_PREFIXES.iter().any(|m| ltarget.starts_with(m)) && !target.starts_with(':');
        let is_lang = LANG_PREFIXES.iter().any(|m| ltarget.starts_with(m)) && !target.starts_with(':');
        if !is_media && !is_lang {
            let label = if parts.len() > 1 { parts[parts.len() - 1].trim() } else { target.trim_start_matches(':').trim_start_matches('#') };
            let label = if label.is_empty() { target.trim_start_matches(':') } else { label };
            out.push_str(&clean_links(label));
        }
        i = end;
    }
    out.push_str(&s[i..]);
    out
}

fn clean_ext_links(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while let Some(p) = s[i..].find('[').map(|x| x + i) {
        let rest = &s[p + 1..];
        if rest.starts_with("http://") || rest.starts_with("https://") || rest.starts_with("//") {
            if let Some(e) = rest.find(']') {
                out.push_str(&s[i..p]);
                let body = &rest[..e];
                if let Some(sp) = body.find(' ') {
                    out.push_str(body[sp + 1..].trim());
                }
                i = p + 1 + e + 1;
                continue;
            }
        }
        out.push_str(&s[i..p + 1]);
        i = p + 1;
    }
    out.push_str(&s[i..]);
    out
}

fn strip_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        if rest.starts_with('<') {
            let nxt = rest[1..].chars().next();
            let tagish = matches!(nxt, Some(c) if c.is_ascii_alphabetic() || c == '/' || c == '!');
            if tagish {
                if let Some(gt) = rest.find('>') {
                    let tag = &rest[1..gt];
                    if gt < 600 && !tag.contains('<') {
                        let name: String = tag.trim_start_matches('/').chars().take_while(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_lowercase();
                        if name == "br" {
                            out.push(LB);
                        }
                        i += gt + 1;
                        continue;
                    }
                }
            }
        }
        let ch = rest.chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

pub fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        if rest.starts_with('&') {
            if let Some(semi) = rest[..rest.len().min(12)].find(';') {
                let ent = &rest[1..semi];
                let rep: Option<String> = match ent {
                    "nbsp" => Some(" ".into()),
                    "amp" => Some("&".into()),
                    "lt" => Some("<".into()),
                    "gt" => Some(">".into()),
                    "quot" => Some("\"".into()),
                    "apos" => Some("'".into()),
                    _ => ent
                        .strip_prefix("#x")
                        .or_else(|| ent.strip_prefix("#X"))
                        .and_then(|h| u32::from_str_radix(h, 16).ok())
                        .or_else(|| ent.strip_prefix('#').and_then(|d| d.parse().ok()))
                        .and_then(char::from_u32)
                        .map(|c| c.to_string()),
                };
                if let Some(r) = rep {
                    out.push_str(&r);
                    i += semi + 1;
                    continue;
                }
            }
        }
        let ch = rest.chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}
