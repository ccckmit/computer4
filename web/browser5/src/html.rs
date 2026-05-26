use scraper::{Html, Node as ScraperNode};
use xdom4::Node;

pub fn parse_html(html_text: &str) -> Node {
    let document = Html::parse_document(html_text);
    let root = document.tree.root();
    convert_node(root).unwrap_or_else(|| Node::new_element("html"))
}

fn convert_node(node: ego_tree::NodeRef<ScraperNode>) -> Option<Node> {
    match node.value() {
        ScraperNode::Text(text) => {
            let t = text.text.to_string();
            if t.trim().is_empty() {
                return None;
            }
            Some(Node::new_text(&t))
        }
        ScraperNode::Element(el) => {
            let tag = el.name().to_lowercase();
            let mut n = Node::new_element(&tag);
            for (k, v) in el.attrs() {
                n.attrs.insert(k.to_string(), v.to_string());
            }
            for child in node.children() {
                if let Some(c) = convert_node(child) {
                    n.children.push(c);
                }
            }
            Some(n)
        }
        ScraperNode::Document => {
            for child in node.children() {
                if let Some(c) = convert_node(child) {
                    return Some(c);
                }
            }
            None
        }
        _ => None,
    }
}

pub fn extract_scripts(node: &Node) -> Vec<String> {
    let mut scripts = Vec::new();
    match &node.node_type {
        xdom4::NodeType::Element(tag) => {
            if tag == "script" {
                for c in &node.children {
                    if let xdom4::NodeType::Text(t) = &c.node_type {
                        scripts.push(t.clone());
                    }
                }
            } else {
                for c in &node.children {
                    scripts.extend(extract_scripts(c));
                }
            }
        }
        _ => {}
    }
    scripts
}

pub fn extract_inline_css(node: &Node) -> Vec<String> {
    let mut styles = Vec::new();
    match &node.node_type {
        xdom4::NodeType::Element(tag) => {
            if tag == "style" {
                for c in &node.children {
                    if let xdom4::NodeType::Text(t) = &c.node_type {
                        styles.push(t.clone());
                    }
                }
            } else {
                for c in &node.children {
                    styles.extend(extract_inline_css(c));
                }
            }
        }
        _ => {}
    }
    styles
}

pub fn extract_link_css(node: &Node, base_url: &str) -> Vec<String> {
    let mut urls = Vec::new();
    match &node.node_type {
        xdom4::NodeType::Element(tag) => {
            if tag == "link" {
                let rel = node.attrs.get("rel").map(|s| s.as_str()).unwrap_or("");
                let href = node.attrs.get("href").map(|s| s.as_str()).unwrap_or("");
                if rel == "stylesheet" && !href.is_empty() {
                    let url = if href.starts_with("http") || href.starts_with("file://") || base_url.is_empty() {
                        href.to_string()
                    } else {
                        let base = base_url.trim_end_matches('/');
                        format!("{}/{}", base, href.trim_start_matches('/'))
                    };
                    urls.push(url);
                }
            }
            for c in &node.children {
                urls.extend(extract_link_css(c, base_url));
            }
        }
        _ => {}
    }
    urls
}
