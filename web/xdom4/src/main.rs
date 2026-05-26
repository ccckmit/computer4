use std::env;
use std::fs;
use std::process;
use xdom4::{parse, query, to_string, Node};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: xdom4 <file.xml> [selector]");
        eprintln!("       xdom4 -q <selector> <file.xml>");
        process::exit(1);
    }
    let (file, selector) = if args[1] == "-q" {
        if args.len() < 4 {
            eprintln!("usage: xdom4 -q <selector> <file.xml>");
            process::exit(1);
        }
        (args[3].as_str(), Some(args[2].as_str()))
    } else if args.len() >= 3 {
        (args[1].as_str(), Some(args[2].as_str()))
    } else {
        (args[1].as_str(), None)
    };

    let content = fs::read_to_string(file).unwrap_or_else(|e| {
        eprintln!("error reading {}: {}", file, e);
        process::exit(1);
    });
    let doc = parse(&content).unwrap_or_else(|e| {
        eprintln!("parse error: {}", e);
        process::exit(1);
    });

    match selector {
        Some(sel) => {
            let nodes = query(&doc, sel).unwrap_or_else(|e| {
                eprintln!("query error: {}", e);
                process::exit(1);
            });
            if nodes.is_empty() {
                println!("(no match)");
            }
            for (i, node) in nodes.iter().enumerate() {
                println!("[{}] {}", i, to_string_node(node));
            }
        }
        None => {
            println!("{}", to_string(&doc));
        }
    }
}

fn to_string_node(node: &Node) -> String {
    let tag = node.tag_name().unwrap_or("?");
    let mut out = format!("<{}", tag);
    for (k, v) in &node.attrs {
        out.push_str(&format!(" {}=\"{}\"", k, v));
    }
    if node.children.is_empty() {
        out.push_str("/>");
    } else {
        let text = node.children.iter()
            .filter_map(|c| c.text())
            .collect::<Vec<_>>()
            .join("");
        if text.is_empty() {
            out.push_str("/>");
        } else {
            out.push_str(&format!(">{}</{}>", text, tag));
        }
    }
    out
}
