use js4::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;

use crate::Node;

static JS_OUTPUT: Mutex<String> = Mutex::new(String::new());

fn js_console_log(args: Vec<Value>) -> Value {
    let output: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
    if let Ok(mut s) = JS_OUTPUT.lock() {
        s.push_str(&output.join(" "));
        s.push('\n');
    }
    Value::Undefined
}

fn get_text_content(node: &Node) -> String {
    let mut text = String::new();
    match &node.node_type {
        crate::NodeType::Text(t) => text.push_str(t),
        crate::NodeType::Element(_) => {
            for c in &node.children {
                text.push_str(&get_text_content(c));
            }
        }
    }
    text
}

fn find_node_by_id<'a>(node: &'a Node, id: &str) -> Option<&'a Node> {
    match &node.node_type {
        crate::NodeType::Text(_) => None,
        crate::NodeType::Element(_) => {
            if node.attrs.get("id").map_or(false, |v| v == id) {
                return Some(node);
            }
            for c in &node.children {
                if let Some(found) = find_node_by_id(c, id) {
                    return Some(found);
                }
            }
            None
        }
    }
}

fn collect_nodes_by_selector<'a>(node: &'a Node, selector: &str) -> Vec<&'a Node> {
    match node.query(selector) {
        Ok(nodes) => nodes,
        Err(_) => vec![],
    }
}

fn make_element_value(node: &Node) -> Value {
    let tag = node.tag_name().unwrap_or("").to_string();
    let id = node.attrs.get("id").cloned().unwrap_or_default();
    let class = node.attrs.get("class").cloned().unwrap_or_default();
    let text = get_text_content(node);

    let map = Rc::new(RefCell::new(HashMap::new()));
    {
        let mut m = map.borrow_mut();

        m.insert("id".to_string(), Value::String(id));
        m.insert("tagName".to_string(), Value::String(tag.to_uppercase()));
        m.insert("innerText".to_string(), Value::String(text.clone()));
        m.insert("textContent".to_string(), Value::String(text));
        m.insert("className".to_string(), Value::String(class));

        let style_map = Rc::new(RefCell::new(HashMap::new()));
        m.insert("style".to_string(), Value::Object(style_map));

        let val = node.attrs.get("value").cloned().unwrap_or_default();
        m.insert("value".to_string(), Value::String(val));

        let listener_map: Rc<RefCell<HashMap<String, Vec<String>>>> = Rc::new(RefCell::new(HashMap::new()));
        let lm = listener_map.clone();
        m.insert("addEventListener".to_string(), Value::Builtin(Rc::new(move |args| {
            if args.len() >= 2 {
                if let Value::String(event) = &args[0] {
                    if let Value::String(code) = &args[1] {
                        let mut m = lm.borrow_mut();
                        m.entry(event.clone()).or_default().push(code.clone());
                    } else if let Value::Function { .. } = &args[1] {
                        let mut m = lm.borrow_mut();
                        m.entry(event.clone()).or_default().push("[function]".to_string());
                    }
                }
            }
            Value::Undefined
        })));

        m.insert("__listeners".to_string(), Value::Object({
            let lm2 = Rc::new(RefCell::new(HashMap::new()));
            lm2.borrow_mut().insert("_inner".to_string(), Value::Object({
                let inner = Rc::new(RefCell::new(HashMap::new()));
                inner.borrow_mut().insert("data".to_string(), Value::String("listener_storage".to_string()));
                inner
            }));
            lm2
        }));
    }

    Value::Object(map)
}

pub struct JsRuntime {
    pub env: Rc<RefCell<Environment>>,
    pub output: String,
    dom_root: Rc<RefCell<Option<Node>>>,
    element_cache: Rc<RefCell<HashMap<String, Value>>>,
}

impl JsRuntime {
    pub fn new() -> Self {
        if let Ok(mut s) = JS_OUTPUT.lock() {
            s.clear();
        }
        let env = Rc::new(RefCell::new(Environment::new()));
        {
            let console_map = Rc::new(RefCell::new(HashMap::new()));
            console_map.borrow_mut().insert("log".to_string(), Value::Builtin(Rc::new(js_console_log)));
            env.borrow_mut().define("console".to_string(), Value::Object(console_map));
        }
        JsRuntime {
            env,
            output: String::new(),
            dom_root: Rc::new(RefCell::new(None)),
            element_cache: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn set_dom(&mut self, root: Node) {
        *self.dom_root.borrow_mut() = Some(root);
        self.element_cache.borrow_mut().clear();
        self.setup_document();
    }

    fn setup_document(&mut self) {
        let dom_root = self.dom_root.clone();
        let element_cache = self.element_cache.clone();

        let get_elem = move |args: Vec<Value>| -> Value {
            if args.is_empty() { return Value::Null; }
            let id = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Value::Null,
            };
            {
                let cache = element_cache.borrow();
                if let Some(v) = cache.get(&id) {
                    return v.clone();
                }
            }
            let root_guard = dom_root.borrow();
            let root = match &*root_guard {
                Some(r) => r,
                None => return Value::Null,
            };
            if let Some(node) = find_node_by_id(root, &id) {
                let el = make_element_value(node);
                element_cache.borrow_mut().insert(id, el.clone());
                el
            } else {
                Value::Null
            }
        };

        let qs_dom_root = self.dom_root.clone();
        let qs_cache = self.element_cache.clone();
        let query_sel = move |args: Vec<Value>| -> Value {
            if args.is_empty() { return Value::Null; }
            let sel = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Value::Null,
            };
            let root_guard = qs_dom_root.borrow();
            let root = match &*root_guard {
                Some(r) => r,
                None => return Value::Null,
            };
            let nodes = collect_nodes_by_selector(root, &sel);
            if nodes.is_empty() { return Value::Null; }
            let node = nodes[0];
            if sel.starts_with('#') {
                if let Some(id) = sel.strip_prefix('#') {
                    let cache = qs_cache.borrow();
                    if let Some(v) = cache.get(id) {
                        return v.clone();
                    }
                }
            }
            let el = make_element_value(node);
            if let Some(id) = node.attrs.get("id") {
                qs_cache.borrow_mut().insert(id.clone(), el.clone());
            }
            el
        };

        let qsa_dom_root = self.dom_root.clone();
        let qsa_cache = self.element_cache.clone();
        let query_sel_all = move |args: Vec<Value>| -> Value {
            if args.is_empty() { return Value::Array(Rc::new(RefCell::new(vec![]))); }
            let sel = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Value::Array(Rc::new(RefCell::new(vec![]))),
            };
            let root_guard = qsa_dom_root.borrow();
            let root = match &*root_guard {
                Some(r) => r,
                None => return Value::Array(Rc::new(RefCell::new(vec![]))),
            };
            let nodes = collect_nodes_by_selector(root, &sel);
            let mut arr = Vec::new();
            for node in nodes {
                let el = make_element_value(node);
                if let Some(id) = node.attrs.get("id") {
                    qsa_cache.borrow_mut().insert(id.clone(), el.clone());
                }
                arr.push(el);
            }
            Value::Array(Rc::new(RefCell::new(arr)))
        };

        let doc_map = Rc::new(RefCell::new(HashMap::new()));
        doc_map.borrow_mut().insert("getElementById".to_string(), Value::Builtin(Rc::new(get_elem)));
        doc_map.borrow_mut().insert("querySelector".to_string(), Value::Builtin(Rc::new(query_sel)));
        doc_map.borrow_mut().insert("querySelectorAll".to_string(), Value::Builtin(Rc::new(query_sel_all)));
        self.env.borrow_mut().define("document".to_string(), Value::Object(doc_map));
    }

    fn capture_output(&mut self) {
        if let Ok(mut s) = JS_OUTPUT.lock() {
            self.output.push_str(&s);
            s.clear();
        }
    }

    pub fn execute(&mut self, code: &str) {
        let tokens = tokenize(code);
        let mut parser = Parser::new(tokens);
        while *parser.peek() != Token::Eof {
            let stmt = parser.parse_statement();
            let result = Interpreter::eval_stmt(&stmt, &self.env);
            match result {
                Ok(Signal::Return(v)) => {
                    if let Ok(mut s) = JS_OUTPUT.lock() {
                        s.push_str(&format!("=> {}\n", v));
                    }
                }
                Err(e) => {
                    if let Ok(mut s) = JS_OUTPUT.lock() {
                        s.push_str(&format!("Error: {}\n", e));
                    }
                }
                _ => {}
            }
        }
        self.capture_output();
    }

    pub fn eval_expr(&mut self, expr_code: &str) -> String {
        let tokens = tokenize(expr_code);
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expression();
        match Interpreter::eval_expr(&expr, &self.env) {
            Ok(val) => {
                if let Ok(mut s) = JS_OUTPUT.lock() {
                    s.push_str(&format!("=> {}\n", val));
                }
                let result = format!("{}", val);
                self.capture_output();
                result
            }
            Err(e) => {
                let msg = format!("Error: {}", e);
                if let Ok(mut s) = JS_OUTPUT.lock() {
                    s.push_str(&msg);
                    s.push('\n');
                }
                self.capture_output();
                msg
            }
        }
    }

    pub fn get_var(&self, name: &str) -> Option<Value> {
        self.env.borrow().get(name)
    }

    pub fn get_var_string(&self, name: &str) -> Option<String> {
        self.env.borrow().get(name).map(|v| format!("{}", v))
    }

    pub fn get_cached_element(&self, id: &str) -> Option<Value> {
        self.element_cache.borrow().get(id).cloned()
    }
}
