use std::env;
use std::io::IsTerminal;
use std::path::Path;
use std::process::{Command, exit};

fn main() {
    let args: Vec<String> = env::args().collect();

    let builtins: Vec<(&str, fn(&[String]) -> bool)> = vec![
        ("cd", builtin_cd),
        ("exit", builtin_exit),
        ("export", builtin_export),
        ("echo", builtin_echo),
        ("type", builtin_type),
    ];

    if args.len() > 1 {
        // Script mode: run the script file
        let script = std::fs::read_to_string(&args[1]).unwrap_or_else(|e| {
            eprintln!("sh: cannot open '{}': {}", args[1], e);
            exit(1);
        });
        for line in script.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
            execute_line(trimmed, &builtins);
        }
    } else if std::io::stdin().is_terminal() {
        // Interactive REPL mode
        repl(&builtins);
    } else {
        // Piped stdin mode
        let mut input = String::new();
        while std::io::stdin().read_line(&mut input).ok().is_some_and(|n| n > 0) {
            let trimmed = input.trim();
            if !trimmed.is_empty() {
                execute_line(trimmed, &builtins);
            }
            input.clear();
        }
    }
}

fn repl(builtins: &[(&str, fn(&[String]) -> bool)]) {
    use std::io::Write;
    let mut input = String::new();
    loop {
        print!("$ ");
        std::io::stdout().flush().ok();
        input.clear();
        if std::io::stdin().read_line(&mut input).ok().is_none_or(|n| n == 0) {
            println!();
            break;
        }
        let trimmed = input.trim();
        if trimmed.is_empty() { continue; }
        if trimmed == "exit" { break; }
        execute_line(trimmed, builtins);
    }
}

fn execute_line(line: &str, builtins: &[(&str, fn(&[String]) -> bool)]) {
    // Handle variable assignments: VAR=value command
    let parts = tokenize(line);
    if parts.is_empty() { return; }

    // Check for variable assignments before the command
    let mut cmd_start = 0;
    let mut env_vars: Vec<(String, String)> = Vec::new();
    for part in &parts {
        if let Some(eq) = part.find('=') {
            if eq > 0 {
                let name = &part[..eq];
                let value = &part[eq + 1..];
                env_vars.push((name.to_string(), value.to_string()));
                cmd_start += 1;
                continue;
            }
        }
        break;
    }

    if cmd_start >= parts.len() { return; }

    let cmd = &parts[cmd_start];
    let cmd_args: Vec<String> = parts[cmd_start + 1..].to_vec();

    // Check builtins
    for (name, func) in builtins {
        if *name == cmd.as_str() {
            func(&cmd_args);
            return;
        }
    }

    // External command
    let mut child = Command::new(cmd);
    child.args(&cmd_args);

    for (k, v) in &env_vars {
        child.env(k, v);
    }

    match child.status() {
        Ok(status) => { status.code().map(|c| exit(c)); }
        Err(e) => {
            eprintln!("sh: {}: {}", cmd, e);
        }
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut in_dquote = false;
    let mut escape = false;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if escape {
            current.push(c);
            escape = false;
            i += 1;
            continue;
        }
        if c == '\\' && !in_quote {
            escape = true;
            i += 1;
            continue;
        }
        if c == '\'' && !in_dquote {
            in_quote = !in_quote;
            i += 1;
            continue;
        }
        if c == '"' && !in_quote {
            in_dquote = !in_dquote;
            i += 1;
            continue;
        }
        if (c == ' ' || c == '\t') && !in_quote && !in_dquote {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            i += 1;
            continue;
        }
        if c == '$' && !in_quote {
            // Variable expansion
            let mut var = String::new();
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                var.push(chars[i]);
                i += 1;
            }
            if !var.is_empty() {
                let val = env::var(&var).unwrap_or_default();
                current.push_str(&val);
            }
            continue;
        }
        current.push(c);
        i += 1;
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

// ─── Builtins ─────────────────────────────────────────────────────────────

fn builtin_cd(args: &[String]) -> bool {
    let dir = if args.is_empty() {
        env::var("HOME").unwrap_or_else(|_| "/".to_string())
    } else {
        args[0].clone()
    };
    if let Err(e) = env::set_current_dir(Path::new(&dir)) {
        eprintln!("cd: {}: {}", dir, e);
    }
    true
}

fn builtin_exit(args: &[String]) -> bool {
    let code = args.first().and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
    exit(code);
}

fn builtin_export(args: &[String]) -> bool {
    for arg in args {
        if let Some(eq) = arg.find('=') {
            let name = &arg[..eq];
            let value = &arg[eq + 1..];
            env::set_var(name, value);
        }
    }
    true
}

fn builtin_echo(args: &[String]) -> bool {
    println!("{}", args.join(" "));
    true
}

fn builtin_type(args: &[String]) -> bool {
    if args.is_empty() {
        eprintln!("type: usage: type name ...");
        return true;
    }
    for arg in args {
        let builtins = ["cd", "exit", "export", "echo", "type"];
        if builtins.contains(&arg.as_str()) {
            println!("{} is a shell builtin", arg);
        } else if let Ok(path) = which(arg) {
            println!("{} is {}", arg, path);
        } else {
            println!("{}: not found", arg);
        }
    }
    true
}

fn which(name: &str) -> Result<String, ()> {
    let path = env::var("PATH").unwrap_or_default();
    for dir in path.split(':') {
        let full = Path::new(dir).join(name);
        if full.is_file() {
            return Ok(full.to_string_lossy().to_string());
        }
    }
    Err(())
}
