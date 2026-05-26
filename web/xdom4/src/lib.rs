use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Element(String),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub node_type: NodeType,
    pub attrs: HashMap<String, String>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new_element(name: &str) -> Self {
        Node {
            node_type: NodeType::Element(name.to_string()),
            attrs: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn new_text(content: &str) -> Self {
        Node {
            node_type: NodeType::Text(content.to_string()),
            attrs: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn tag_name(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Element(name) => Some(name),
            NodeType::Text(_) => None,
        }
    }

    pub fn text(&self) -> Option<&str> {
        match &self.node_type {
            NodeType::Element(_) => None,
            NodeType::Text(s) => Some(s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub root: Node,
}

impl Document {
    pub fn new(root: Node) -> Self {
        Document { root }
    }

    pub fn query(&self, selector: &str) -> Result<Vec<&Node>, String> {
        self.root.query(selector)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Combinator {
    Descendant,
    Child,
}

#[derive(Debug, Clone, PartialEq)]
enum AttrCheck {
    Has(String),
    Equals(String, String),
}

#[derive(Debug, Clone, PartialEq)]
struct Compound {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
    attr_checks: Vec<AttrCheck>,
}

impl Compound {
    fn matches(&self, node: &Node) -> bool {
        let name = match &node.node_type {
            NodeType::Text(_) => return false,
            NodeType::Element(n) => n,
        };
        if let Some(ref t) = self.tag {
            if name != t {
                return false;
            }
        }
        if let Some(ref id) = self.id {
            match node.attrs.get("id") {
                Some(v) if v == id => {}
                _ => return false,
            }
        }
        for cls in &self.classes {
            match node.attrs.get("class") {
                Some(v) => {
                    if !v.split_ascii_whitespace().any(|c| c == cls) {
                        return false;
                    }
                }
                None => return false,
            }
        }
        for check in &self.attr_checks {
            match check {
                AttrCheck::Has(name) => {
                    if !node.attrs.contains_key(name) {
                        return false;
                    }
                }
                AttrCheck::Equals(name, val) => {
                    match node.attrs.get(name) {
                        Some(v) if v == val => {}
                        _ => return false,
                    }
                }
            }
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
struct QueryStep {
    combinator: Combinator,
    selector: Compound,
}

struct SelectorParser {
    input: String,
    pos: usize,
}

impl SelectorParser {
    fn new(input: &str) -> Self {
        SelectorParser {
            input: input.to_string(),
            pos: 0,
        }
    }

    fn done(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn skip_ws(&mut self) {
        while self.pos < self.input.len() {
            let b = self.input.as_bytes()[self.pos];
            if b.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn parse_name(&mut self) -> Result<String, String> {
        let start = self.pos;
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos] as char;
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err("expected name in selector".to_string());
        }
        Ok(self.input[start..self.pos].to_string())
    }

    fn parse_attr_value(&mut self) -> Result<String, String> {
        self.skip_ws();
        if self.done() {
            return Err("unexpected EOF in attribute value".to_string());
        }
        let c = self.input.as_bytes()[self.pos] as char;
        if c == '"' || c == '\'' {
            self.pos += 1;
            let start = self.pos;
            while self.pos < self.input.len() {
                if (self.input.as_bytes()[self.pos] as char) == c {
                    let val = self.input[start..self.pos].to_string();
                    self.pos += 1;
                    return Ok(val);
                }
                self.pos += 1;
            }
            return Err("unterminated attribute value".to_string());
        }
        let start = self.pos;
        while self.pos < self.input.len() {
            let c = self.input.as_bytes()[self.pos] as char;
            if c == ']' || c.is_ascii_whitespace() {
                break;
            }
            self.pos += 1;
        }
        if self.pos == start {
            return Err("empty attribute value".to_string());
        }
        Ok(self.input[start..self.pos].to_string())
    }

    fn parse_compound(&mut self) -> Result<Compound, String> {
        let mut cpd = Compound {
            tag: None,
            id: None,
            classes: Vec::new(),
            attr_checks: Vec::new(),
        };
        if !self.done() {
            let c = self.input.as_bytes()[self.pos] as char;
            if c.is_ascii_alphabetic() || c == '*' || c == '_' {
                let tag = self.parse_name()?;
                if tag != "*" {
                    cpd.tag = Some(tag);
                }
            }
        }
        loop {
            if self.done() {
                break;
            }
            match self.input.as_bytes()[self.pos] as char {
                '#' => {
                    self.pos += 1;
                    cpd.id = Some(self.parse_name()?);
                }
                '.' => {
                    self.pos += 1;
                    cpd.classes.push(self.parse_name()?);
                }
                '[' => {
                    self.pos += 1;
                    let name = self.parse_name()?;
                    self.skip_ws();
                    if !self.done() && self.input.as_bytes()[self.pos] as char == '=' {
                        self.pos += 1;
                        let val = self.parse_attr_value()?;
                        cpd.attr_checks.push(AttrCheck::Equals(name, val));
                    } else {
                        cpd.attr_checks.push(AttrCheck::Has(name));
                    }
                    self.skip_ws();
                    if self.done() || self.input.as_bytes()[self.pos] as char != ']' {
                        return Err("expected ']' in attribute selector".to_string());
                    }
                    self.pos += 1;
                }
                _ => break,
            }
        }
        Ok(cpd)
    }

    fn parse(&mut self) -> Result<Vec<QueryStep>, String> {
        let mut steps = Vec::new();
        let mut combinator = Combinator::Descendant;
        loop {
            self.skip_ws();
            if self.done() {
                break;
            }
            if self.input.as_bytes()[self.pos] as char == '>' {
                combinator = Combinator::Child;
                self.pos += 1;
                continue;
            }
            let cpd = self.parse_compound()?;
            steps.push(QueryStep { combinator, selector: cpd });
            combinator = Combinator::Descendant;
            self.skip_ws();
            if self.done() {
                break;
            }
            if self.input.as_bytes()[self.pos] as char == '>' {
                combinator = Combinator::Child;
                self.pos += 1;
            }
        }
        if steps.is_empty() {
            return Err("empty selector".to_string());
        }
        Ok(steps)
    }
}

fn parse_selector(input: &str) -> Result<Vec<QueryStep>, String> {
    SelectorParser::new(input).parse()
}

fn collect_all<'a>(node: &'a Node, out: &mut Vec<&'a Node>) {
    if matches!(node.node_type, NodeType::Element(_)) {
        out.push(node);
    }
    for child in &node.children {
        collect_all(child, out);
    }
}

fn collect_descendants<'a>(node: &'a Node, out: &mut Vec<&'a Node>) {
    for child in &node.children {
        if matches!(child.node_type, NodeType::Element(_)) {
            out.push(child);
            collect_descendants(child, out);
        }
    }
}

impl Node {
    pub fn query(&self, selector: &str) -> Result<Vec<&Node>, String> {
        let steps = parse_selector(selector)?;
        if steps.is_empty() {
            return Ok(vec![]);
        }
        let all: Vec<&Node> = {
            let mut v = Vec::new();
            if matches!(self.node_type, NodeType::Element(_)) {
                v.push(self);
            }
            for child in &self.children {
                collect_all(child, &mut v);
            }
            v
        };
        let mut results: Vec<&Node> = all.into_iter()
            .filter(|n| steps[0].selector.matches(n))
            .collect();
        for step in &steps[1..] {
            let candidates: Vec<&Node> = match step.combinator {
                Combinator::Child => {
                    results.iter()
                        .flat_map(|n| n.children.iter())
                        .filter(|n| matches!(n.node_type, NodeType::Element(_)))
                        .collect()
                }
                Combinator::Descendant => {
                    results.iter()
                        .flat_map(|n| {
                            let mut v = Vec::new();
                            collect_descendants(n, &mut v);
                            v
                        })
                        .collect()
                }
            };
            results = candidates.into_iter()
                .filter(|n| step.selector.matches(n))
                .collect();
        }
        Ok(results)
    }
}

pub fn query<'a>(doc: &'a Document, selector: &str) -> Result<Vec<&'a Node>, String> {
    doc.query(selector)
}

pub struct Parser {
    input: Vec<char>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        Parser {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Document, String> {
        self.skip_whitespace();
        let root = self.parse_element()?;
        Ok(Document::new(root))
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            let ch = self.input[self.pos];
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        match self.advance() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(format!(
                "expected '{}', found '{}' at pos {}",
                expected, ch, self.pos
            )),
            None => Err(format!("unexpected EOF, expected '{}'", expected)),
        }
    }

    fn parse_element(&mut self) -> Result<Node, String> {
        self.expect_char('<')?;
        let name = self.parse_name()?;
        let mut node = Node::new_element(&name);
        self.parse_attributes(&mut node)?;
        if self.peek() == Some('/') {
            self.advance();
            self.expect_char('>')?;
            return Ok(node);
        }
        self.expect_char('>')?;
        self.parse_children(&mut node)?;
        self.expect_char('<')?;
        self.expect_char('/')?;
        let close_name = self.parse_name()?;
        self.skip_whitespace();
        self.expect_char('>')?;
        if close_name != name {
            return Err(format!(
                "mismatched tag: </{}> does not match <{}>",
                close_name, name
            ));
        }
        Ok(node)
    }

    fn parse_name(&mut self) -> Result<String, String> {
        let mut name = String::new();
        match self.peek() {
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
                name.push(self.advance().unwrap());
            }
            Some(ch) => {
                return Err(format!(
                    "unexpected char '{}' in tag name at pos {}",
                    ch, self.pos
                ))
            }
            None => return Err("unexpected EOF in tag name".to_string()),
        }
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == ':' || ch == '.' {
                name.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        Ok(name)
    }

    fn parse_attributes(&mut self, node: &mut Node) -> Result<(), String> {
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('>') | Some('/') => break,
                Some(_) => {
                    let name = self.parse_name()?;
                    self.skip_whitespace();
                    if self.peek() == Some('=') {
                        self.advance();
                        self.skip_whitespace();
                        let value = self.parse_attr_value()?;
                        node.attrs.insert(name, value);
                    } else {
                        node.attrs.insert(name, String::new());
                    }
                }
                None => return Err("unexpected EOF in attributes".to_string()),
            }
        }
        Ok(())
    }

    fn parse_attr_value(&mut self) -> Result<String, String> {
        let quote = match self.advance() {
            Some(ch) if ch == '"' || ch == '\'' => ch,
            Some(ch) => return Err(format!("expected quote, found '{}'", ch)),
            None => return Err("unexpected EOF in attribute value".to_string()),
        };
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch == quote {
                self.advance();
                return Ok(value);
            }
            value.push(self.advance().unwrap());
        }
        Err("unterminated attribute value".to_string())
    }

    fn parse_children(&mut self, node: &mut Node) -> Result<(), String> {
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => return Err("unexpected EOF in children".to_string()),
                Some('<') => {
                    if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '/' {
                        break;
                    }
                    node.children.push(self.parse_element()?);
                }
                Some(_) => {
                    node.children.push(self.parse_text()?);
                }
            }
        }
        Ok(())
    }

    fn parse_text(&mut self) -> Result<Node, String> {
        let mut content = String::new();
        while let Some(ch) = self.peek() {
            if ch == '<' {
                break;
            }
            content.push(self.advance().unwrap());
        }
        Ok(Node::new_text(&content))
    }
}

pub fn parse(input: &str) -> Result<Document, String> {
    let mut p = Parser::new(input);
    p.parse()
}

pub fn format_node(node: &Node, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    match &node.node_type {
        NodeType::Text(t) => {
            let trimmed = t.trim();
            if trimmed.is_empty() {
                String::new()
            } else {
                format!("{}{}", pad, trimmed)
            }
        }
        NodeType::Element(name) => {
            let mut out = format!("{}<{}", pad, name);
            for (k, v) in &node.attrs {
                out.push_str(&format!(" {}=\"{}\"", k, v));
            }
            if node.children.is_empty() {
                out.push_str("/>");
                return out;
            }
            if node.children.len() == 1 {
                if let NodeType::Text(ref t) = node.children[0].node_type {
                    out.push('>');
                    out.push_str(&format!("{}</{}>", t, name));
                    return out;
                }
            }
            out.push('>');
            out.push('\n');
            for child in &node.children {
                let s = format_node(child, indent + 1);
                if !s.is_empty() {
                    out.push_str(&s);
                    out.push('\n');
                }
            }
            out.push_str(&format!("{}</{}>", pad, name));
            out
        }
    }
}

pub fn to_string(doc: &Document) -> String {
    let s = format_node(&doc.root, 0);
    if s.contains('\n') {
        s
    } else {
        format!("{}\n", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_element() {
        let doc = parse("<br/>").unwrap();
        assert_eq!(doc.root.tag_name(), Some("br"));
    }

    #[test]
    fn test_text_content() {
        let doc = parse("<p>hello</p>").unwrap();
        assert_eq!(doc.root.children.len(), 1);
        assert_eq!(doc.root.children[0].text(), Some("hello"));
    }

    #[test]
    fn test_nested() {
        let doc = parse("<a><b><c/></b></a>").unwrap();
        assert_eq!(doc.root.children[0].children[0].tag_name(), Some("c"));
    }

    #[test]
    fn test_attributes() {
        let doc = parse("<x id=\"42\" ok=\"yes\"/>").unwrap();
        assert_eq!(doc.root.attrs.get("id"), Some(&"42".to_string()));
        assert_eq!(doc.root.attrs.get("ok"), Some(&"yes".to_string()));
    }

    #[test]
    fn test_mismatch_error() {
        assert!(parse("<a></b>").is_err());
    }

    #[test]
    fn test_siblings() {
        let doc = parse("<r><a/><b/><c/></r>").unwrap();
        assert_eq!(doc.root.children.len(), 3);
    }

    #[test]
    fn test_mixed_content() {
        let doc = parse("<p>before<b>bold</b>after</p>").unwrap();
        assert_eq!(doc.root.children.len(), 3);
        assert_eq!(doc.root.children[0].text(), Some("before"));
        assert_eq!(doc.root.children[1].tag_name(), Some("b"));
        assert_eq!(doc.root.children[1].children[0].text(), Some("bold"));
        assert_eq!(doc.root.children[2].text(), Some("after"));
    }

    #[test]
    fn test_single_quote_attr() {
        let doc = parse("<x val='abc'/>").unwrap();
        assert_eq!(doc.root.attrs.get("val"), Some(&"abc".to_string()));
    }

    #[test]
    fn test_roundtrip() {
        let xml = "<root><item id=\"1\">hello</item></root>";
        let doc = parse(xml).unwrap();
        let out = to_string(&doc);
        let doc2 = parse(&out).unwrap();
        assert_eq!(doc, doc2);
    }

    #[test]
    fn test_query_tag() {
        let doc = parse("<a><b id='x'/><b id='y'/><c/></a>").unwrap();
        let res = doc.query("b").unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].attrs.get("id").unwrap(), "x");
        assert_eq!(res[1].attrs.get("id").unwrap(), "y");
    }

    #[test]
    fn test_query_id() {
        let doc = parse("<r><x id='a'/><y id='b'/></r>").unwrap();
        let res = doc.query("#b").unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].tag_name(), Some("y"));
    }

    #[test]
    fn test_query_class() {
        let doc = parse("<r><x class='foo bar'/><y class='baz'/></r>").unwrap();
        let res = doc.query(".foo").unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].tag_name(), Some("x"));
    }

    #[test]
    fn test_query_attr_equals() {
        let doc = parse("<r><a type='text'/><b type='code'/></r>").unwrap();
        let res = doc.query("[type=\"text\"]").unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].tag_name(), Some("a"));
    }

    #[test]
    fn test_query_attr_exists() {
        let doc = parse("<r><a hidden/><b/></r>").unwrap();
        let res = doc.query("[hidden]").unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].tag_name(), Some("a"));
    }

    #[test]
    fn test_query_child() {
        let doc = parse("<ul><li>a</li><li>b</li></ul>").unwrap();
        let res = doc.query("ul > li").unwrap();
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn test_query_descendant() {
        let doc = parse("<div><p><span>hi</span></p></div>").unwrap();
        let res = doc.query("div span").unwrap();
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn test_query_no_match() {
        let doc = parse("<a><b/></a>").unwrap();
        let res = doc.query("c").unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn test_query_child_vs_descendant() {
        let doc = parse("<a><b><c/></b></a>").unwrap();
        let child = doc.query("a > c").unwrap();
        assert!(child.is_empty());
        let desc = doc.query("a c").unwrap();
        assert_eq!(desc.len(), 1);
    }

    #[test]
    fn test_query_compound() {
        let doc = parse("<r><x id='m' class='c' data='v'/><x id='n' class='c'/><x id='m'/></r>").unwrap();
        let res = doc.query("x#m.c[data]").unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].attrs.get("data").unwrap(), "v");
    }

    #[test]
    fn test_query_invalid_selector() {
        let doc = parse("<a/>").unwrap();
        assert!(doc.query("").is_err());
    }

    #[test]
    fn test_node_query() {
        let doc = parse("<ul><li>a</li><li>b</li></ul>").unwrap();
        let res = doc.root.query("li").unwrap();
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn test_query_toplevel_fn() {
        let doc = parse("<root><item/><item/></root>").unwrap();
        let res = query(&doc, "item").unwrap();
        assert_eq!(res.len(), 2);
    }
}
