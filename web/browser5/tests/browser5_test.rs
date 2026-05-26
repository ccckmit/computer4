use browser5::css;
use browser5::html;

#[test]
fn test_parse_simple_html() {
    let dom = html::parse_html("<html><body><h1>Hello</h1></body></html>");
    assert_eq!(dom.tag_name(), Some("html"));
    let body = dom.children.iter().find(|c| c.tag_name() == Some("body")).unwrap();
    let h1 = &body.children[0];
    assert_eq!(h1.tag_name(), Some("h1"));
}

#[test]
fn test_html_with_text() {
    let dom = html::parse_html("<p>Hello World</p>");
    let body = dom.children.iter().find(|c| c.tag_name() == Some("body")).unwrap();
    let p = &body.children[0];
    assert_eq!(p.tag_name(), Some("p"));
    let text = &p.children[0];
    assert_eq!(text.text(), Some("Hello World"));
}

#[test]
fn test_html_with_attributes() {
    let dom = html::parse_html("<a href=\"test.html\" class=\"link\">click</a>");
    let body = dom.children.iter().find(|c| c.tag_name() == Some("body")).unwrap();
    let a = &body.children[0];
    assert_eq!(a.attrs.get("href"), Some(&"test.html".to_string()));
    assert_eq!(a.attrs.get("class"), Some(&"link".to_string()));
}

#[test]
fn test_extract_scripts() {
    let dom = html::parse_html("<script>let x = 1;</script><p>hello</p><script>let y = 2;</script>");
    let scripts = html::extract_scripts(&dom);
    assert_eq!(scripts.len(), 2);
    assert!(scripts[0].contains("let x = 1;") || scripts[0].contains("let x = 1"));
    assert!(scripts[1].contains("let y = 2;") || scripts[1].contains("let y = 2"));
}

#[test]
fn test_extract_inline_css() {
    let dom = html::parse_html("<style>body { color: red; }</style><p>text</p>");
    let styles = html::extract_inline_css(&dom);
    assert_eq!(styles.len(), 1);
    assert!(styles[0].contains("color: red"));
}

#[test]
fn test_css_parser_simple() {
    let rules = css::parse_css("h1 { color: red; font-size: 24px; }");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].selectors.len(), 1);
    assert_eq!(rules[0].selectors[0], "h1");
    assert_eq!(rules[0].declarations.len(), 2);
}

#[test]
fn test_css_parser_multiple_rules() {
    let rules = css::parse_css("h1 { color: red; }\np { font-size: 16px; }");
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].selectors[0], "h1");
    assert_eq!(rules[1].selectors[0], "p");
}

#[test]
fn test_css_parser_multiple_selectors() {
    let rules = css::parse_css("h1, h2, h3 { color: blue; }");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].selectors.len(), 3);
}

#[test]
fn test_css_parser_with_comments() {
    let rules = css::parse_css("/* comment */ h1 { color: red; } /* another */");
    assert_eq!(rules.len(), 1);
}

#[test]
fn test_parse_color_hex6() {
    let color = css::parse_color("#ff0000");
    assert!(color.is_some());
    assert_eq!(color.unwrap(), eframe::egui::Color32::from_rgb(255, 0, 0));
}

#[test]
fn test_parse_color_named() {
    let color = css::parse_color("blue");
    assert!(color.is_some());
    assert_eq!(color.unwrap(), eframe::egui::Color32::BLUE);
}

#[test]
fn test_parse_color_invalid() {
    let color = css::parse_color("not-a-color");
    assert!(color.is_none());
}

#[test]
fn test_compute_style_simple() {
    let node = xdom4::Node::new_element("h1");
    let rules = css::parse_css("h1 { color: red; font-size: 20px; }");
    let style = css::compute_style(&node, &rules);
    assert_eq!(style.color, Some("red".to_string()));
    assert_eq!(style.font_size, Some(20.0));
}

#[test]
fn test_compute_style_class() {
    let mut node = xdom4::Node::new_element("div");
    node.attrs.insert("class".to_string(), "highlight".to_string());
    let rules = css::parse_css(".highlight { background-color: yellow; color: black; }");
    let style = css::compute_style(&node, &rules);
    assert_eq!(style.background_color, Some("yellow".to_string()));
    assert_eq!(style.color, Some("black".to_string()));
}

#[test]
fn test_compute_style_id() {
    let mut node = xdom4::Node::new_element("div");
    node.attrs.insert("id".to_string(), "main".to_string());
    let rules = css::parse_css("#main { font-size: 18px; font-weight: bold; }");
    let style = css::compute_style(&node, &rules);
    assert_eq!(style.font_size, Some(18.0));
    assert_eq!(style.font_weight, Some("bold".to_string()));
}

#[test]
fn test_compute_style_inline() {
    let mut node = xdom4::Node::new_element("p");
    node.attrs.insert("style".to_string(), "color: green; font-size: 14px;".to_string());
    let rules = css::parse_css("p { color: red; }");
    let style = css::compute_style(&node, &rules);
    assert_eq!(style.color, Some("green".to_string()));
    assert_eq!(style.font_size, Some(14.0));
}

#[test]
fn test_get_text_only() {
    let mut div = xdom4::Node::new_element("div");
    div.children.push(xdom4::Node::new_text("Hello "));
    let mut strong = xdom4::Node::new_element("strong");
    strong.children.push(xdom4::Node::new_text("World"));
    div.children.push(strong);
    assert_eq!(css::get_text_only(&div), "Hello World");
}

#[test]
fn test_js_execution() {
    use browser5::js::JsRuntime;
    let mut rt = JsRuntime::new();
    rt.execute("let x = 42;");
    let val = rt.get_var_string("x");
    assert!(val.is_some(), "variable x should be defined");
    assert_eq!(val.unwrap(), "42");
}

#[test]
fn test_js_execution_multiple() {
    use browser5::js::JsRuntime;
    let mut rt = JsRuntime::new();
    rt.execute("let a = 10; let b = 20;");
    rt.execute("let c = a + b;");
    let val = rt.get_var_string("c");
    assert!(val.is_some(), "variable c should be defined");
    assert_eq!(val.unwrap(), "30");
}

#[test]
fn test_js_execution_functions() {
    use browser5::js::JsRuntime;
    let mut rt = JsRuntime::new();
    rt.execute("function add(a, b) { return a + b; }");
    rt.execute("let result = add(5, 7);");
    let val = rt.get_var_string("result");
    assert!(val.is_some(), "variable result should be defined");
    assert_eq!(val.unwrap(), "12");
}

#[test]
fn test_dom_get_element_by_id() {
    use browser5::html;
    use browser5::js::JsRuntime;
    let dom = html::parse_html(r#"<html><body><div id="display">0</div></body></html>"#);
    let mut rt = JsRuntime::new();
    rt.set_dom(dom);
    rt.execute(r#"
        let el = document.getElementById("display");
        let result = "";
        if (el != null) {
            result = el.innerText;
        } else {
            result = "null";
        }
    "#);
    let val = rt.get_var_string("result").unwrap_or("undefined".to_string());
    assert_eq!(val, r#""0""#, "el.innerText should be '0', got: {}", val);
}

#[test]
fn test_dom_set_inner_text() {
    use browser5::html;
    use browser5::js::JsRuntime;
    use js4::Value;
    let dom = html::parse_html(r#"<html><body><div id="display">0</div></body></html>"#);
    let mut rt = JsRuntime::new();
    rt.set_dom(dom);
    rt.execute(r#"
        let el = document.getElementById("display");
        if (el != null) {
            el.innerText = "hello";
        }
    "#);
    let cached = rt.get_cached_element("display");
    assert!(cached.is_some(), "element should be cached");
    if let Some(Value::Object(m)) = cached {
        let inner = m.borrow().get("innerText").cloned();
        assert!(inner.is_some(), "innerText should exist");
        if let Some(Value::String(s)) = inner {
            assert_eq!(s, "hello", "innerText should be 'hello'");
        } else {
            panic!("innerText should be a string");
        }
    } else {
        panic!("cached element should be an Object");
    }
}
