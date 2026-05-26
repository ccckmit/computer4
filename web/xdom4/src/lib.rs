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
                    self.expect_char('=')?;
                    self.skip_whitespace();
                    let value = self.parse_attr_value()?;
                    node.attrs.insert(name, value);
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
}
