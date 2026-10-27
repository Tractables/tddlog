//! The XML that GeNIe's `.xdsl` networks and Ace's `.inst` evidence files
//! are written in, read into a tree of elements: their names, attributes,
//! children and text. Comments, processing instructions and a document
//! type are skipped, character and the predefined entity references are
//! read, and the document is ISO-8859-1 where its declaration says so and
//! UTF-8 otherwise. Namespaces and entities a document type declares are
//! not read.

/// An element: its name, its attributes in order, its child elements and
/// the text directly inside it.
pub(super) struct Element {
    pub name: String,
    attributes: Vec<(String, String)>,
    pub children: Vec<Element>,
    pub text: String,
}

impl Element {
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// The first child of that name.
    pub fn child(&self, name: &str) -> Option<&Element> {
        self.children.iter().find(|c| c.name == name)
    }
}

/// The root element of the document `data`.
pub(super) fn parse(data: &[u8]) -> Result<Element, String> {
    let text = decoded(data)?;
    // Line ends read as one newline (XML 1.0, 2.11).
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut reader = Reader { s: &text, i: 0 };
    reader.skip_misc()?;
    if reader.rest().is_empty() {
        return Err("xml: no root element".into());
    }
    let root = reader.element()?;
    reader.skip_misc()?;
    if !reader.rest().is_empty() {
        return Err("xml: text after the root element".into());
    }
    Ok(root)
}

/// The document's text: ISO-8859-1 where its declaration names it, else
/// UTF-8 (after a byte order mark, if any).
fn decoded(data: &[u8]) -> Result<String, String> {
    let data = data.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(data);
    let declaration = data
        .strip_prefix(b"<?xml")
        .and_then(|rest| rest.split(|&b| b == b'>').next())
        .map(|d| String::from_utf8_lossy(d).to_ascii_lowercase())
        .unwrap_or_default();
    let latin1 = ["iso-8859-1", "latin-1", "latin1", "iso8859-1"]
        .iter()
        .any(|e| {
            declaration.contains(&format!("encoding=\"{e}\""))
                || declaration.contains(&format!("encoding='{e}'"))
        });
    if latin1 {
        Ok(data.iter().map(|&b| char::from(b)).collect())
    } else {
        String::from_utf8(data.to_vec()).map_err(|_| "xml: not UTF-8".to_string())
    }
}

struct Reader<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Reader<'a> {
    fn rest(&self) -> &'a str {
        &self.s[self.i..]
    }

    /// Past the first `end` from here.
    fn skip_past(&mut self, end: &str, what: &str) -> Result<(), String> {
        match self.rest().find(end) {
            Some(k) => {
                self.i += k + end.len();
                Ok(())
            }
            None => Err(format!("xml: unterminated {what}")),
        }
    }

    /// Past white space, comments, processing instructions and a document
    /// type, outside the root element.
    fn skip_misc(&mut self) -> Result<(), String> {
        loop {
            let rest = self.rest();
            let trimmed = rest.trim_start();
            self.i += rest.len() - trimmed.len();
            if trimmed.starts_with("<?") {
                self.skip_past("?>", "processing instruction")?;
            } else if trimmed.starts_with("<!--") {
                self.skip_past("-->", "comment")?;
            } else if trimmed.starts_with("<!DOCTYPE") {
                // An internal subset in brackets may hold `>`.
                let rest = self.rest();
                let end = match (rest.find('['), rest.find('>')) {
                    (Some(b), Some(g)) if b < g => rest[b..]
                        .find("]")
                        .and_then(|c| rest[b + c..].find('>').map(|g| b + c + g)),
                    (_, g) => g,
                };
                match end {
                    Some(g) => self.i += g + 1,
                    None => return Err("xml: unterminated document type".into()),
                }
            } else {
                return Ok(());
            }
        }
    }

    /// The element whose start tag begins here, with what it holds.
    fn element(&mut self) -> Result<Element, String> {
        let mut open: Vec<Element> = Vec::new();
        loop {
            let rest = self.rest();
            if rest.is_empty() {
                return Err("xml: unterminated element".into());
            }
            if let Some(after) = rest.strip_prefix("</") {
                let end = after.find('>').ok_or("xml: unterminated end tag")?;
                let name = after[..end].trim_end();
                self.i += 2 + end + 1;
                let done = open.pop().ok_or("xml: an end tag without its start")?;
                if done.name != name {
                    return Err(format!("xml: <{}> ended by </{name}>", done.name));
                }
                match open.last_mut() {
                    Some(parent) => parent.children.push(done),
                    None => return Ok(done),
                }
            } else if rest.starts_with("<!--") {
                self.skip_past("-->", "comment")?;
            } else if rest.starts_with("<?") {
                self.skip_past("?>", "processing instruction")?;
            } else if let Some(after) = rest.strip_prefix("<![CDATA[") {
                let end = after.find("]]>").ok_or("xml: unterminated CDATA section")?;
                let parent = open
                    .last_mut()
                    .ok_or("xml: text outside the root element")?;
                parent.text.push_str(&after[..end]);
                self.i += 9 + end + 3;
            } else if rest.starts_with('<') {
                let (element, empty) = self.start_tag()?;
                if empty {
                    match open.last_mut() {
                        Some(parent) => parent.children.push(element),
                        None => return Ok(element),
                    }
                } else {
                    open.push(element);
                }
            } else {
                let end = rest.find('<').unwrap_or(rest.len());
                let parent = open
                    .last_mut()
                    .ok_or("xml: text outside the root element")?;
                parent.text.push_str(&unescaped(&rest[..end])?);
                self.i += end;
            }
        }
    }

    /// A start tag `<name a="v" ...>` or `<name ... />`: the element, and
    /// whether the tag is empty.
    fn start_tag(&mut self) -> Result<(Element, bool), String> {
        let tag = &self.rest()[1..];
        let name_end = tag
            .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
            .ok_or("xml: unterminated start tag")?;
        let name = &tag[..name_end];
        if name.is_empty() {
            return Err("xml: a tag without a name".into());
        }
        let mut attributes = Vec::new();
        let mut a = tag[name_end..].trim_start();
        let empty = loop {
            if let Some(after) = a.strip_prefix("/>") {
                a = after;
                break true;
            }
            if let Some(after) = a.strip_prefix('>') {
                a = after;
                break false;
            }
            let eq = a.find('=').ok_or("xml: unterminated start tag")?;
            let key = a[..eq].trim();
            let value = a[eq + 1..].trim_start();
            let quote = value
                .chars()
                .next()
                .filter(|&q| q == '"' || q == '\'')
                .ok_or("xml: an attribute value without quotes")?;
            let close = value[1..]
                .find(quote)
                .ok_or("xml: an attribute value without its closing quote")?;
            // White space in a value reads as spaces (XML 1.0, 3.3.3).
            let raw = value[1..1 + close].replace(['\n', '\t'], " ");
            attributes.push((key.to_string(), unescaped(&raw)?));
            a = value[1 + close + 1..].trim_start();
        };
        self.i = self.s.len() - a.len();
        Ok((
            Element {
                name: name.to_string(),
                attributes,
                children: Vec::new(),
                text: String::new(),
            },
            empty,
        ))
    }
}

/// `s` with its character and predefined entity references read.
fn unescaped(s: &str) -> Result<String, String> {
    if !s.contains('&') {
        return Ok(s.to_string());
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp + 1..];
        let semi = after.find(';').ok_or("xml: an unterminated reference")?;
        let entity = &after[..semi];
        let c = match entity {
            "lt" => '<',
            "gt" => '>',
            "amp" => '&',
            "quot" => '"',
            "apos" => '\'',
            _ => {
                let code = if let Some(hex) = entity.strip_prefix("#x") {
                    u32::from_str_radix(hex, 16).ok()
                } else if let Some(dec) = entity.strip_prefix('#') {
                    dec.parse().ok()
                } else {
                    None
                };
                code.and_then(char::from_u32)
                    .ok_or_else(|| format!("xml: an unknown reference &{entity};"))?
            }
        };
        out.push(c);
        rest = &after[semi + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_document_reads_as_its_tree() {
        let doc = b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?>\r\n<!-- a comment -->\r\n\
            <smile id=\"n\"><nodes>\r\n<cpt id=\"a&amp;b\"><state id='x' />\r\n<!-- skipped -->\
            <probabilities>0.3 0.7</probabilities></cpt></nodes><extensions><x/></extensions></smile>";
        let root = parse(doc).unwrap();
        assert_eq!(root.name, "smile");
        assert_eq!(root.attribute("id"), Some("n"));
        let nodes = root.child("nodes").unwrap();
        assert_eq!(nodes.children.len(), 1);
        let cpt = &nodes.children[0];
        assert_eq!(cpt.attribute("id"), Some("a&b"));
        assert_eq!(cpt.children.len(), 2);
        assert_eq!(cpt.child("state").unwrap().attribute("id"), Some("x"));
        assert_eq!(cpt.child("probabilities").unwrap().text, "0.3 0.7");
        assert!(parse(b"<a><b></a>").is_err());
        assert!(parse(b"<a></a><b/>").is_err());
        let latin = parse(b"<?xml version='1.0' encoding='ISO-8859-1'?><s id='\xe9'/>").unwrap();
        assert_eq!(latin.attribute("id"), Some("\u{e9}"));
    }
}
