use crate::ir::*;

pub fn parse_ir(source: &str) -> Program {
    let lines: Vec<&str> = source.lines().collect();
    let mut globals = Vec::new();
    let mut functions = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        i += 1;
        if line.is_empty() || line.starts_with(';') { continue; }
        if line.starts_with("declare") { i += 0; continue; }
        if line.starts_with("target") || line.starts_with("source") { continue; }
        if line.starts_with('@') {
            if let Some(g) = parse_global(line) { globals.push(g); }
            continue;
        }
        if line.starts_with("define") {
            let mut body = line.to_string();
            let mut depth = line.matches('{').count() - line.matches('}').count();
            while depth > 0 && i < lines.len() {
                let l = lines[i];
                body.push('\n');
                body.push_str(l);
                depth += l.matches('{').count();
                depth -= l.matches('}').count();
                i += 1;
            }
            if let Some(f) = parse_fn_decl(&body) { functions.push(f); }
        }
    }

    Program { globals, functions }
}

fn parse_global(line: &str) -> Option<GlobalVar> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() < 2 { return None; }
    let name = parts[0].trim().trim_start_matches('@').to_string();
    let rest = parts[1].trim();
    let mut s = rest;
    loop {
        if s.starts_with("private") { s = s[7..].trim(); }
        else if s.starts_with("internal") { s = s[8..].trim(); }
        else if s.starts_with("unnamed_addr") { s = s[13..].trim(); }
        else if s.starts_with("constant") { s = s[8..].trim(); }
        else if s.starts_with("global") { s = s[6..].trim(); }
        else if s.starts_with("local_unnamed_addr") { s = s[19..].trim(); }
        else { break; }
    }
    let (ty, s) = parse_llvm_type(s);
    let data = if s.starts_with('c') && s.len() > 2 && s.as_bytes()[1] == b'"' {
        let end = s[2..].find('"').map(|i| i+3).unwrap_or(s.len());
        let raw = &s[2..end-1];
        decode_escapes(raw).into_bytes()
    } else {
        Vec::new()
    };
    Some(GlobalVar { name, ty, data })
}

fn decode_escapes(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut j = 0;
    while j < chars.len() {
        if chars[j] == '\\' && j+1 < chars.len() {
            match chars[j+1] {
                '0' if j+2 < chars.len() && chars[j+2].is_ascii_hexdigit() => {
                    let hex: String = chars[j+1..=j+2].iter().collect();
                    if let Ok(v) = u8::from_str_radix(&hex, 16) {
                        out.push(v as char);
                        j += 3;
                    } else { out.push('\\'); j += 1; }
                }
                '0' => { out.push('\0'); j += 2; }
                'n' => { out.push('\n'); j += 2; }
                't' => { out.push('\t'); j += 2; }
                'r' => { out.push('\r'); j += 2; }
                '"' => { out.push('"'); j += 2; }
                '\\' => { out.push('\\'); j += 2; }
                c => { out.push(c); j += 2; }
            }
        } else { out.push(chars[j]); j += 1; }
    }
    out
}

fn parse_llvm_type(s: &str) -> (LlvmType, &str) {
    let s = s.trim();
    if s.is_empty() { return (LlvmType::I32, s); }
    if s.starts_with("i64") { let rest = s[3..].trim(); return if rest.starts_with('*') { (LlvmType::Pointer(Box::new(LlvmType::I32)), rest[1..].trim()) } else { (LlvmType::I32, rest) }; }
    if s.starts_with("i32") { let rest = s[3..].trim(); return if rest.starts_with('*') { (LlvmType::Pointer(Box::new(LlvmType::I32)), rest[1..].trim()) } else { (LlvmType::I32, rest) }; }
    if s.starts_with("i8")  { let rest = s[2..].trim(); return if rest.starts_with('*') { (LlvmType::Pointer(Box::new(LlvmType::I8)), rest[1..].trim()) } else { (LlvmType::I8, rest) }; }
    if s.starts_with("i1")  { return (LlvmType::I1, s[2..].trim()); }
    if s.starts_with("void") { return (LlvmType::Void, s[4..].trim()); }
    if s.starts_with('[') {
        let after = s[1..].trim();
        let num_end = after.find(|c: char| !c.is_ascii_digit()).unwrap_or(after.len());
        let n: u64 = after[..num_end].parse().unwrap_or(0);
        let rest = after[num_end..].trim();
        let rest = rest.strip_prefix('x').unwrap_or(rest).trim();
        let (inner, rest) = parse_llvm_type(rest);
        let rest = rest.trim_start_matches(']').trim();
        let arr = LlvmType::Array(n, Box::new(inner));
        return if rest.starts_with('*') { (LlvmType::Pointer(Box::new(arr)), rest[1..].trim()) }
        else { (arr, rest) };
    }
    let end = s.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.').unwrap_or(s.len());
    let rest = s[end..].trim();
    (LlvmType::I32, rest)
}

fn parse_fn_decl(body: &str) -> Option<FnDecl> {
    let lines: Vec<&str> = body.lines().collect();
    if lines.is_empty() { return None; }

    let sig = lines[0].trim();
    let s = sig.strip_prefix("define")?.trim();

    let (ret_ty, s) = parse_llvm_type(s);
    let s = s.trim();

    let fn_name = s.strip_prefix('@')?.trim();
    let name_end = fn_name.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.' && c != '-').unwrap_or(fn_name.len());
    let name = fn_name[..name_end].to_string();

    // Parse params from the signature
    let open_paren = sig.find('(')?;
    let close_paren = sig[open_paren..].find(')').map(|i| open_paren + i).unwrap_or(sig.len());
    let param_str = &sig[open_paren+1..close_paren];
    let mut params = Vec::new();
    for p in param_str.split(',') {
        let p = p.trim();
        if p.is_empty() || p == "..." { continue; }
        if let Some((ty, name)) = parse_param_type_name(p) { params.push((name, ty)); }
    }

    // Parse blocks
    let mut blocks = Vec::new();
    let mut cur_label = String::new();
    let mut cur_instrs = Vec::new();

    for &line in lines[1..].iter() {
        let tl = line.trim();
        if tl.is_empty() || tl.starts_with(';') || tl == "{" || tl == "}" {
            if tl == "}" && !cur_instrs.is_empty() {
                blocks.push(BasicBlock { label: if cur_label.is_empty() { "entry".into() } else { cur_label.clone() }, instrs: std::mem::take(&mut cur_instrs) });
            }
            continue;
        }
        if tl.ends_with(':') {
            if !cur_instrs.is_empty() {
                blocks.push(BasicBlock { label: if cur_label.is_empty() { "entry".into() } else { cur_label.clone() }, instrs: std::mem::take(&mut cur_instrs) });
            }
            cur_label = tl.trim_end_matches(':').to_string();
            continue;
        }
        // label: instr on same line
        if let Some(col) = tl.find(':') {
            let before = tl[..col].trim();
            let after = tl[col+1..].trim();
            if !before.is_empty() && before.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') && !after.is_empty() {
                if !cur_instrs.is_empty() {
                    blocks.push(BasicBlock { label: if cur_label.is_empty() { "entry".into() } else { cur_label.clone() }, instrs: std::mem::take(&mut cur_instrs) });
                }
                cur_label = before.to_string();
                if let Some(instr) = parse_instr(after) { cur_instrs.push(instr); }
                continue;
            }
        }
        if let Some(instr) = parse_instr(tl) { cur_instrs.push(instr); }
    }
    if !cur_instrs.is_empty() {
        blocks.push(BasicBlock { label: if cur_label.is_empty() { "entry".into() } else { cur_label.clone() }, instrs: cur_instrs });
    }

    Some(FnDecl { name, params, ret_ty, blocks })
}

fn parse_param_type_name(s: &str) -> Option<(LlvmType, String)> {
    let s = s.trim();
    let (ty, rest) = parse_llvm_type(s);
    let rest = rest.trim();
    let param_name = if rest.starts_with('%') {
        let end = rest[1..].find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.' && c != '-').map(|i| i+1).unwrap_or(rest.len());
        rest[1..end].to_string()
    } else { String::new() };
    Some((ty, param_name))
}

fn parse_instr(s: &str) -> Option<Instruction> {
    let s = s.trim();
    // Extract result if present
    let (result, rest) = if let Some(eq) = s.find('=') {
        let r = s[..eq].trim();
        let after = s[eq+1..].trim();
        if r.starts_with('%') {
            let end = r[1..].find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.').map(|i| i+1).unwrap_or(r.len());
            (Some(r[1..end].to_string()), after)
        } else { (None, s) }
    } else { (None, s) };

    let op_end = rest.find(|c: char| c == ' ').unwrap_or(rest.len());
    let opcode = &rest[..op_end];
    let args = rest[op_end..].trim();

    match opcode {
        "alloca" => {
            let (ty, _) = parse_llvm_type(args);
            Some(Instruction::Alloca { result: result.unwrap_or_default(), ty })
        }
        "store" => {
            let args = skip_type(args);
            let (val, rest) = parse_val(args);
            let rest = trim_comma(rest);
            let rest = skip_type(rest);
            let (ptr, _) = parse_val(rest);
            Some(Instruction::Store { ty: LlvmType::I32, val, ptr })
        }
        "load" => {
            let (ty, rest) = parse_llvm_type(args);
            let rest = trim_comma(rest);
            let rest = skip_type(rest);
            let (ptr, _) = parse_val(rest);
            Some(Instruction::Load { result: result.unwrap_or_default(), ty, ptr })
        }
        "add" | "sub" | "mul" | "sdiv" | "srem" | "and" | "or" | "xor" => {
            let (ty, rest) = parse_llvm_type(args);
            let rest = rest.trim();
            let (lhs, rest) = parse_val(rest);
            let rest = trim_comma(rest);
            let (rhs, _) = parse_val(rest);
            let r = result.unwrap_or_default();
            match opcode {
                "add" => Some(Instruction::Add { result: r, ty, lhs, rhs }),
                "sub" => Some(Instruction::Sub { result: r, ty, lhs, rhs }),
                "mul" => Some(Instruction::Mul { result: r, ty, lhs, rhs }),
                "sdiv" => Some(Instruction::SDiv { result: r, ty, lhs, rhs }),
                "srem" => Some(Instruction::SRem { result: r, ty, lhs, rhs }),
                "and" => Some(Instruction::And { result: r, ty, lhs, rhs }),
                "or" => Some(Instruction::Or { result: r, ty, lhs, rhs }),
                "xor" => Some(Instruction::Xor { result: r, ty, lhs, rhs }),
                _ => None,
            }
        }
        "icmp" => {
            let after_icmp = args.trim();
            let cond_end = after_icmp.find(|c: char| c == ' ').unwrap_or(after_icmp.len());
            let cond_str = &after_icmp[..cond_end];
            let cond = match cond_str {
                "eq" => ICmpCond::Eq, "ne" => ICmpCond::Ne, "slt" => ICmpCond::Slt,
                "sgt" => ICmpCond::Sgt, "sle" => ICmpCond::Sle, "sge" => ICmpCond::Sge,
                _ => return None,
            };
            let rest = after_icmp[cond_end..].trim();
            let (ty, rest) = parse_llvm_type(rest);
            let rest = rest.trim();
            let (lhs, rest) = parse_val(rest);
            let rest = trim_comma(rest);
            let (rhs, _) = parse_val(rest);
            Some(Instruction::ICmp { result: result.unwrap_or_default(), cond, ty, lhs, rhs })
        }
        "call" => {
            let mut r = args;
            // Skip optional calling convention sig like (i32, ...)
            if r.starts_with('(') {
                let paren_end = r.find(')').map(|i| i+1).unwrap_or(r.len());
                r = r[paren_end..].trim();
            }
            let (ret_ty, r) = parse_llvm_type(r);
            let r = r.trim();
            let fn_name = r.strip_prefix('@')?;
            let name_end = fn_name.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.' && c != '-').unwrap_or(fn_name.len());
            let name = fn_name[..name_end].to_string();
            let rest = fn_name[name_end..].trim();
            let args_str = rest.strip_prefix('(').and_then(|s| s.rfind(')').map(|i| &s[..i])).unwrap_or("");
            let mut args = Vec::new();
            for part in args_str.split(',') {
                let part = part.trim();
                if part.is_empty() { continue; }
                let (v, _) = parse_val_skip_type(part); args.push(v);
            }
            Some(Instruction::Call { result, ret_ty, name, args })
        }
        "ret" => {
            let rest = args.trim();
            if rest == "void" { Some(Instruction::Ret { val: None }) }
            else { let (val, _) = parse_val_skip_type(rest); Some(Instruction::Ret { val: Some(val) }) }
        }
        "br" => {
            let rest = args.trim();
            if let Some(rest) = rest.strip_prefix("label") {
                let t = rest.trim().trim_start_matches('%');
                let end = t.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.').unwrap_or(t.len());
                Some(Instruction::Br(t[..end].to_string()))
            } else {
                // br i1 %cond, label %t, label %f
                let rest = rest.strip_prefix("i1").unwrap_or(rest).trim();
                let rest = rest.trim_start_matches('%');
                let end = rest.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.').unwrap_or(rest.len());
                let cond = Operand::Local(rest[..end].to_string());
                let mut r = rest[end..].trim();
                r = r.strip_prefix(',').unwrap_or(r).trim();
                r = r.strip_prefix("label").unwrap_or(r).trim();
                let t = r.trim_start_matches('%');
                let end = t.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.').unwrap_or(t.len());
                let t_label = t[..end].to_string();
                r = t[end..].trim();
                r = r.strip_prefix(',').unwrap_or(r).trim();
                r = r.strip_prefix("label").unwrap_or(r).trim();
                let f = r.trim_start_matches('%');
                let end = f.find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.').unwrap_or(f.len());
                let f_label = f[..end].to_string();
                Some(Instruction::BrCond(cond, t_label, f_label))
            }
        }
        "getelementptr" => {
            let (_, rest) = parse_llvm_type(args);
            let rest = trim_comma(rest);
            let (ptr, mut rest) = parse_val_skip_type(rest);
            let mut indices = Vec::new();
            loop {
                rest = rest.trim();
                if rest.is_empty() { break; }
                rest = trim_comma(rest);
                if rest.is_empty() { break; }
                let prev_len = rest.len();
                let (idx, r) = parse_val_skip_type(rest);
                indices.push(idx);
                if r.len() >= prev_len { break; }
                rest = r;
            }
            Some(Instruction::GetElementPtr { result: result.unwrap_or_default(), ptr, indices })
        }
        _ => None,
    }
}

fn skip_type(s: &str) -> &str {
    let s = s.trim();
    if s.starts_with("i64") || s.starts_with("i32") || s.starts_with("i8") || s.starts_with("i1") || s.starts_with("void") {
        let end = if s.starts_with("i64") { 3 } else if s.starts_with("i32") { 3 } else if s.starts_with("i8") { 2 } else { let e = if s.starts_with("i1") { 2 } else { 4 }; e };
        let rest = s[end..].trim();
        if rest.starts_with('*') { rest[1..].trim() } else { rest }
    } else if s.starts_with('[') {
        let end = s.find(']').map(|i| i+1).unwrap_or(s.len());
        let rest = s[end..].trim();
        if rest.starts_with('*') { rest[1..].trim() } else { rest }
    } else if s.starts_with("label") { s[5..].trim() }
    else { s }
}

fn trim_comma(s: &str) -> &str {
    s.trim().strip_prefix(',').unwrap_or(s).trim()
}

fn parse_val(s: &str) -> (Operand, &str) {
    let s = s.trim();
    if s.is_empty() { return (Operand::Int(0), s); }
    if s.starts_with('%') {
        let end = s[1..].find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.' && c != '-').map(|i| i+1).unwrap_or(s.len());
        (Operand::Local(s[1..end].to_string()), s[end..].trim())
    } else if s.starts_with('@') {
        let end = s[1..].find(|c: char| !c.is_alphanumeric() && c != '_' && c != '.' && c != '-').map(|i| i+1).unwrap_or(s.len());
        (Operand::Global(s[1..end].to_string()), s[end..].trim())
    } else if s.starts_with("true") { (Operand::Bool(true), s[4..].trim()) }
    else if s.starts_with("false") { (Operand::Bool(false), s[5..].trim()) }
    else if s.starts_with("null") || s.starts_with("zeroinitializer") {
        let end = if s.starts_with("null") { 4 } else { 15 };
        (Operand::Int(0), s[end..].trim())
    } else {
        let end = s.find(|c: char| !c.is_ascii_digit() && c != '-').unwrap_or(s.len());
        if end == 0 { (Operand::Int(0), s) }
        else { (Operand::Int(s[..end].parse().unwrap_or(0)), s[end..].trim()) }
    }
}

fn parse_val_skip_type(s: &str) -> (Operand, &str) {
    let s = skip_type(s);
    parse_val(s)
}
