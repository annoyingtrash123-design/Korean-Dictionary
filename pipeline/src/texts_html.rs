//! Tiny HTML helpers (no parser dependency) for the OHCHR and korea.kr fetchers.

use crate::texts_wikitext::{decode_entities, remove_element};

struct Tag<'a> {
    end: usize, // index just past '>'
    name: String,
    closing: bool,
    self_closing: bool,
    attrs: &'a str,
}

/// Parse the tag starting at `i` (`html[i] == '<'`).
fn scan_tag(html: &str, i: usize) -> Option<Tag<'_>> {
    let rest = &html[i + 1..];
    let first = rest.chars().next()?;
    let closing = first == '/';
    let body_start = if closing { 1 } else { 0 };
    if !(rest[body_start..].chars().next()?.is_ascii_alphabetic()) {
        return None;
    }
    // find '>' outside quotes
    let b = rest.as_bytes();
    let mut q: Option<u8> = None;
    let mut j = body_start;
    while j < b.len() {
        match (q, b[j]) {
            (None, b'"') | (None, b'\'') => q = Some(b[j]),
            (Some(c), x) if c == x => q = None,
            (None, b'>') => break,
            _ => {}
        }
        j += 1;
    }
    if j >= b.len() {
        return None;
    }
    let inner = &rest[body_start..j];
    let name_end = inner.find(|c: char| c.is_whitespace() || c == '/').unwrap_or(inner.len());
    Some(Tag {
        end: i + 1 + j + 1,
        name: inner[..name_end].to_ascii_lowercase(),
        closing,
        self_closing: inner.ends_with('/'),
        attrs: &inner[name_end..],
    })
}

const VOID: &[&str] = &["br", "img", "meta", "link", "hr", "input", "source", "area", "base", "col", "embed", "wbr"];

/// Value of attribute `name` in an attribute string.
pub fn attr_value(attrs: &str, name: &str) -> Option<String> {
    let lower = attrs.to_ascii_lowercase();
    let mut from = 0;
    while let Some(p) = lower[from..].find(name).map(|x| x + from) {
        let before_ok = p == 0 || !lower.as_bytes()[p - 1].is_ascii_alphanumeric() && lower.as_bytes()[p - 1] != b'-';
        let rest = attrs[p + name.len()..].trim_start();
        if before_ok && rest.starts_with('=') {
            let v = rest[1..].trim_start();
            return Some(match v.chars().next() {
                Some(q @ ('"' | '\'')) => v[1..].split(q).next().unwrap_or("").to_string(),
                _ => v.split(|c: char| c.is_whitespace() || c == '>').next().unwrap_or("").to_string(),
            });
        }
        from = p + name.len();
    }
    None
}

/// `<meta property|name="key" content="…">`.
pub fn meta_content(html: &str, key: &str) -> Option<String> {
    let mut i = 0;
    while let Some(p) = html[i..].find("<meta").map(|x| x + i) {
        if let Some(t) = scan_tag(html, p) {
            let k = attr_value(t.attrs, "property").or_else(|| attr_value(t.attrs, "name")).or_else(|| attr_value(t.attrs, "itemprop"));
            if k.as_deref().map(|k| k.eq_ignore_ascii_case(key)).unwrap_or(false) {
                if let Some(c) = attr_value(t.attrs, "content") {
                    return Some(decode_entities(c.trim()));
                }
            }
            i = t.end;
        } else {
            i = p + 5;
        }
    }
    None
}

pub fn title_tag(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let s = lower.find("<title")?;
    let gt = lower[s..].find('>')? + s + 1;
    let e = lower[gt..].find("</title>")? + gt;
    Some(normalise(&decode_entities(&html[gt..e])))
}

fn matching_close(html: &str, from: usize, name: &str) -> Option<usize> {
    let mut depth = 1;
    let mut i = from;
    while let Some(p) = html[i..].find('<').map(|x| x + i) {
        match scan_tag(html, p) {
            Some(t) => {
                if t.name == name && !t.self_closing {
                    if t.closing {
                        depth -= 1;
                        if depth == 0 {
                            return Some(p);
                        }
                    } else {
                        depth += 1;
                    }
                }
                i = t.end;
            }
            None => i = p + 1,
        }
    }
    None
}

/// Inner HTML of the first element whose `class`, `id` or `itemprop` contains one of `tokens`
/// (tokens are tried in the given order).
pub fn element_by_token(html: &str, tokens: &[&str]) -> Option<String> {
    for tok in tokens {
        let mut i = 0;
        while let Some(p) = html[i..].find('<').map(|x| x + i) {
            let Some(t) = scan_tag(html, p) else {
                i = p + 1;
                continue;
            };
            i = t.end;
            if t.closing || t.self_closing || VOID.contains(&t.name.as_str()) {
                continue;
            }
            let hit = ["class", "id", "itemprop"].iter().any(|a| attr_value(t.attrs, a).map(|v| v.to_ascii_lowercase().contains(tok)).unwrap_or(false));
            if hit {
                if let Some(e) = matching_close(html, t.end, &t.name) {
                    return Some(html[t.end..e].to_string());
                }
            }
        }
    }
    None
}

/// Inner HTML of the first `<name>` element.
pub fn element_by_name(html: &str, name: &str) -> Option<String> {
    let mut i = 0;
    while let Some(p) = html[i..].find('<').map(|x| x + i) {
        let Some(t) = scan_tag(html, p) else {
            i = p + 1;
            continue;
        };
        i = t.end;
        if !t.closing && !t.self_closing && t.name == name {
            return matching_close(html, t.end, name).map(|e| html[t.end..e].to_string());
        }
    }
    None
}

fn normalise(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Visible text of an HTML fragment: paragraphs separated by a blank line, `<br>` as a newline.
pub fn html_to_text(html: &str) -> String {
    let mut s = html.to_string();
    // comments
    while let Some(p) = s.find("<!--") {
        let e = s[p..].find("-->").map(|x| p + x + 3).unwrap_or(s.len());
        s.replace_range(p..e, "");
    }
    for el in ["script", "style", "noscript", "svg", "nav", "header", "footer", "form", "iframe", "button", "select", "template", "aside"] {
        s = remove_element(&s, el);
    }
    const BLOCK: &[&str] = &["p", "div", "h1", "h2", "h3", "h4", "h5", "h6", "li", "tr", "section", "article", "ul", "ol", "table", "blockquote", "dd", "dt", "dl", "pre", "figure", "figcaption", "main"];
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let rest = &s[i..];
        if rest.starts_with("<!") || rest.starts_with("<?") {
            i += rest.find('>').map(|x| x + 1).unwrap_or(rest.len());
            continue;
        }
        if rest.starts_with('<') {
            if let Some(t) = scan_tag(&s, i) {
                if t.name == "br" {
                    out.push('\n');
                } else if BLOCK.contains(&t.name.as_str()) {
                    out.push_str("\n\n");
                } else if t.name == "td" || t.name == "th" {
                    out.push(' ');
                }
                i = t.end;
                continue;
            }
        }
        let ch = rest.chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    let out = decode_entities(&out);
    let mut lines: Vec<String> = Vec::new();
    for l in out.lines() {
        let l = normalise(l);
        if l.is_empty() && lines.last().is_none_or(|x| x.is_empty()) {
            continue;
        }
        lines.push(l);
    }
    while lines.last().map(|l| l.is_empty()).unwrap_or(false) {
        lines.pop();
    }
    lines.join("\n")
}
