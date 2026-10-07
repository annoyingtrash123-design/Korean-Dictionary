//! Streaming XML helper: yields one small DOM subtree per matching element.
//! Control bytes that are illegal in XML 1.0 (present in a few NIKL files)
//! are dropped at byte level before parsing.

use anyhow::{anyhow, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::{BufRead, BufReader, Read};

#[derive(Debug, Default, Clone)]
pub struct Node {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    /// Concatenated direct text / CDATA content.
    pub text: String,
    pub children: Vec<Node>,
}

impl Node {
    pub fn attr(&self, k: &str) -> Option<&str> {
        self.attrs.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str())
    }
    pub fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }
    pub fn kids<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> + 'a {
        self.children.iter().filter(move |c| c.name == name)
    }
    /// Walk a `/`-separated path of child names, returning every match.
    pub fn find_all<'a>(&'a self, path: &str) -> Vec<&'a Node> {
        let mut cur: Vec<&Node> = vec![self];
        for seg in path.split('/') {
            cur = cur.into_iter().flat_map(|n| n.kids(seg)).collect();
        }
        cur
    }
    /// Trimmed text of the first node at `path` ("" when missing).
    pub fn t(&self, path: &str) -> String {
        self.find_all(path).first().map(|n| n.text.trim().to_string()).unwrap_or_default()
    }
    /// `<feat att="X" val="Y"/>` children as (att, val) pairs.
    pub fn feat(&self, att: &str) -> Option<&str> {
        self.children
            .iter()
            .find(|c| c.name == "feat" && c.attr("att") == Some(att))
            .and_then(|c| c.attr("val"))
    }
}

pub struct CleanReader<R: Read> {
    inner: R,
}

impl<R: Read> CleanReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

#[inline]
fn illegal(b: u8) -> bool {
    matches!(b, 0..=8 | 0x0B | 0x0C | 0x0E..=0x1F)
}

impl<R: Read> Read for CleanReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let n = self.inner.read(buf)?;
            if n == 0 {
                return Ok(0);
            }
            let mut w = 0;
            for r in 0..n {
                let b = buf[r];
                if !illegal(b) {
                    buf[w] = b;
                    w += 1;
                }
            }
            if w > 0 {
                return Ok(w);
            }
        }
    }
}

fn start_node(e: &quick_xml::events::BytesStart) -> Result<Node> {
    let mut n = Node { name: String::from_utf8_lossy(e.name().as_ref()).into_owned(), ..Default::default() };
    for a in e.attributes().with_checks(false) {
        let a = a.map_err(|x| anyhow!("attr: {x}"))?;
        let k = String::from_utf8_lossy(a.key.as_ref()).into_owned();
        let v = a.unescape_value().map(|c| c.into_owned()).unwrap_or_else(|_| String::from_utf8_lossy(&a.value).into_owned());
        n.attrs.push((k, v));
    }
    Ok(n)
}

/// Stream every `tag` element from `reader`, calling `f` with its subtree.
pub fn for_each<R: Read>(reader: R, tag: &str, mut f: impl FnMut(&Node) -> Result<()>) -> Result<()> {
    let br: BufReader<CleanReader<R>> = BufReader::with_capacity(1 << 20, CleanReader::new(reader));
    for_each_buf(br, tag, &mut f)
}

fn for_each_buf<B: BufRead>(br: B, tag: &str, f: &mut dyn FnMut(&Node) -> Result<()>) -> Result<()> {
    let mut rd = Reader::from_reader(br);
    rd.config_mut().check_end_names = false;
    rd.config_mut().expand_empty_elements = true;
    let mut buf = Vec::new();
    let mut stack: Vec<Node> = Vec::new();
    loop {
        match rd.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name();
                if !stack.is_empty() || name.as_ref() == tag.as_bytes() {
                    stack.push(start_node(&e)?);
                }
            }
            Ok(Event::End(_)) => {
                if let Some(n) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(n);
                    } else {
                        f(&n)?;
                    }
                }
            }
            Ok(Event::Text(t)) => {
                if let Some(top) = stack.last_mut() {
                    match t.unescape() {
                        Ok(s) => top.text.push_str(&s),
                        Err(_) => top.text.push_str(&String::from_utf8_lossy(&t)),
                    }
                }
            }
            Ok(Event::CData(t)) => {
                if let Some(top) = stack.last_mut() {
                    top.text.push_str(&String::from_utf8_lossy(&t));
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(anyhow!("xml error at byte {}: {e}", rd.buffer_position())),
        }
        buf.clear();
    }
    Ok(())
}

pub fn for_each_file(path: &std::path::Path, tag: &str, f: impl FnMut(&Node) -> Result<()>) -> Result<()> {
    let fh = std::fs::File::open(path).map_err(|e| anyhow!("open {}: {e}", path.display()))?;
    for_each(fh, tag, f)
}
