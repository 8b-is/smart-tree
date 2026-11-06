//! Brzozowski-derivative GBNF / JSON-Schema constrained sampler automaton.
//! 
//! Compiles MCP/OpenAI JSON Schema to GBNF, and parses GBNF into a DAG for 
//! O(1) Brzozowski byte-derivative transitions during speculative decoding.

use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

// ============================================================================
// 1. JSON-Schema to GBNF Compiler
// ============================================================================

pub fn compile_json_schema_to_gbnf(schema: &Value) -> Result<String, String> {
    let mut rules: Vec<(String, String)> = Vec::new();
    let mut rule_counter = 0usize;

    let root_expr = build_schema_rule(schema, "root", &mut rules, &mut rule_counter)?;

    let mut out = String::new();
    out.push_str(&format!("root ::= ws {root_expr} ws\n"));
    for (name, body) in rules {
        out.push_str(&format!("{name} ::= {body}\n"));
    }

    out.push_str("ws ::= [ \\t\\n\\r]*\n");
    out.push_str("string ::= \"\\\"\" char* \"\\\"\"\n");
    out.push_str("char ::= [^\"\\\\] | \"\\\\\" [\"\\\\/bfnrt]\n");
    out.push_str("int ::= \"-\"? ([0-9] | [1-9] [0-9]*)\n");
    out.push_str("number ::= int (\".\" [0-9]+)? ([eE] [+-]? [0-9]+)?\n");
    out.push_str("boolean ::= \"true\" | \"false\"\n");
    out.push_str("null ::= \"null\"\n");

    Ok(out)
}

fn build_schema_rule(
    schema: &Value,
    hint: &str,
    rules: &mut Vec<(String, String)>,
    counter: &mut usize,
) -> Result<String, String> {
    if let Some(enum_vals) = schema.get("enum").and_then(|v| v.as_array()) {
        if enum_vals.is_empty() {
            return Err("JSON schema 'enum' array cannot be empty".into());
        }
        let mut alts = Vec::with_capacity(enum_vals.len());
        for val in enum_vals {
            let serialized = serde_json::to_string(val).map_err(|e| e.to_string())?;
            alts.push(escape_gbnf_literal(&serialized));
        }
        let rule_name = next_rule_name(hint, counter);
        rules.push((rule_name.clone(), alts.join(" | ")));
        return Ok(rule_name);
    }

    let schema_type = schema
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("object");

    match schema_type {
        "string" => Ok("string".to_string()),
        "integer" => Ok("int".to_string()),
        "number" => Ok("number".to_string()),
        "boolean" => Ok("boolean".to_string()),
        "null" => Ok("null".to_string()),
        "array" => {
            let item_rule = if let Some(items_schema) = schema.get("items") {
                build_schema_rule(items_schema, "item", rules, counter)?
            } else {
                "string".to_string()
            };
            let rule_name = next_rule_name(hint, counter);
            let body = format!(
                "\"[\" ws ({item_rule} (ws \",\" ws {item_rule})*)? ws \"]\""
            );
            rules.push((rule_name.clone(), body));
            Ok(rule_name)
        }
        "object" => {
            let Some(props) = schema.get("properties").and_then(|p| p.as_object()) else {
                return Ok("\"{\" ws \"}\"".to_string());
            };
            if props.is_empty() {
                return Ok("\"{\" ws \"}\"".to_string());
            }

            let required_set: Vec<&str> = schema
                .get("required")
                .and_then(|r| r.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();

            let mut prop_rules = Vec::with_capacity(props.len());
            for (prop_idx, (prop_key, prop_schema)) in props.iter().enumerate() {
                let val_rule = build_schema_rule(prop_schema, prop_key, rules, counter)?;
                let key_lit = escape_gbnf_literal(&format!("\"{prop_key}\""));
                let kv_expr = format!("{key_lit} ws \":\" ws {val_rule}");
                let is_req = required_set.contains(&prop_key.as_str()) || required_set.is_empty();

                let field_rule = next_rule_name(&format!("field_{prop_idx}"), counter);
                rules.push((field_rule.clone(), kv_expr));
                prop_rules.push((field_rule, is_req));
            }

            let mut seq_parts = Vec::with_capacity(prop_rules.len());
            for (idx, (f_rule, is_req)) in prop_rules.into_iter().enumerate() {
                if idx == 0 {
                    if is_req {
                        seq_parts.push(f_rule);
                    } else {
                        seq_parts.push(format!("{f_rule}?"));
                    }
                } else if is_req {
                    seq_parts.push(format!("ws \",\" ws {f_rule}"));
                } else {
                    seq_parts.push(format!("(ws \",\" ws {f_rule})?"));
                }
            }

            let obj_rule = next_rule_name(hint, counter);
            let body = format!("\"{{\" ws {} ws \"}}\"", seq_parts.join(" "));
            rules.push((obj_rule.clone(), body));
            Ok(obj_rule)
        }
        other => Err(format!("Unsupported JSON Schema type: {other}")),
    }
}

fn next_rule_name(hint: &str, counter: &mut usize) -> String {
    *counter += 1;
    let clean: String = hint
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("rule-{clean}-{}", *counter)
}

fn escape_gbnf_literal(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 4);
    out.push('"');
    for ch in raw.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ============================================================================
// 2. GBNF AST & Cloneable Brzozowski Byte-Derivative Automaton
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GbnfExpr {
    Empty,
    Fail,
    Literal(Vec<u8>),
    CharClass { ranges: Vec<(u8, u8)>, negated: bool },
    RuleRef(usize),
    Seq(Vec<GbnfExpr>),
    Alt(Vec<GbnfExpr>),
    Star(Box<GbnfExpr>),
}

#[derive(Clone, Debug)]
pub struct GbnfGrammar {
    pub rules: Vec<GbnfExpr>,
    pub root_id: usize,
}

#[derive(Clone, Debug)]
pub struct GrammarState {
    pub grammar: Arc<GbnfGrammar>,
    pub current: GbnfExpr,
}

impl GbnfGrammar {
    pub fn parse(gbnf_src: &str) -> Result<Arc<Self>, String> {
        let mut raw_rules: Vec<(String, String)> = Vec::new();
        let mut name_to_id: HashMap<String, usize> = HashMap::new();

        for raw_line in gbnf_src.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((lhs, rhs)) = line.split_once("::=") {
                let name = lhs.trim().to_string();
                let body = rhs.trim().to_string();
                if !name_to_id.contains_key(&name) {
                    let id = raw_rules.len();
                    name_to_id.insert(name.clone(), id);
                    raw_rules.push((name, body));
                }
            }
        }

        if raw_rules.is_empty() {
            return Err("GBNF grammar contains no '::=' rules".into());
        }

        let root_id = name_to_id.get("root").copied().unwrap_or(0);
        let mut compiled_rules = Vec::with_capacity(raw_rules.len());

        for (_name, body) in &raw_rules {
            let expr = parse_rule_rhs(body, &name_to_id)?;
            compiled_rules.push(expr);
        }

        Ok(Arc::new(Self {
            rules: compiled_rules,
            root_id,
        }))
    }
}

impl GrammarState {
    pub fn new(grammar: Arc<GbnfGrammar>) -> Self {
        let root_expr = grammar.rules[grammar.root_id].clone();
        Self {
            grammar,
            current: root_expr,
        }
    }

    #[inline]
    pub fn is_complete(&self) -> bool {
        is_nullable(&self.current, &self.grammar.rules, 0)
    }

    pub fn advance_bytes(&self, bytes: &[u8]) -> Option<Self> {
        let mut expr = self.current.clone();
        for &b in bytes {
            expr = derive_byte(&expr, &self.grammar.rules, b, 0);
            if expr == GbnfExpr::Fail {
                return None;
            }
        }
        Some(Self {
            grammar: Arc::clone(&self.grammar),
            current: expr,
        })
    }

    #[inline]
    pub fn accepts_bytes(&self, bytes: &[u8]) -> bool {
        self.advance_bytes(bytes).is_some()
    }
}

fn is_nullable(expr: &GbnfExpr, rules: &[GbnfExpr], depth: usize) -> bool {
    if depth > 64 { return false; }
    match expr {
        GbnfExpr::Empty | GbnfExpr::Star(_) => true,
        GbnfExpr::Fail | GbnfExpr::CharClass { .. } => false,
        GbnfExpr::Literal(bytes) => bytes.is_empty(),
        GbnfExpr::RuleRef(id) => is_nullable(&rules[*id], rules, depth + 1),
        GbnfExpr::Seq(items) => items.iter().all(|e| is_nullable(e, rules, depth + 1)),
        GbnfExpr::Alt(choices) => choices.iter().any(|e| is_nullable(e, rules, depth + 1)),
    }
}

fn derive_byte(expr: &GbnfExpr, rules: &[GbnfExpr], b: u8, depth: usize) -> GbnfExpr {
    if depth > 64 { return GbnfExpr::Fail; }
    match expr {
        GbnfExpr::Empty | GbnfExpr::Fail => GbnfExpr::Fail,
        GbnfExpr::Literal(bytes) => match bytes.split_first() {
            Some((&first, rest)) if first == b => {
                if rest.is_empty() { GbnfExpr::Empty } else { GbnfExpr::Literal(rest.to_vec()) }
            }
            _ => GbnfExpr::Fail,
        },
        GbnfExpr::CharClass { ranges, negated } => {
            let in_range = ranges.iter().any(|&(lo, hi)| b >= lo && b <= hi);
            let matched = if *negated { !in_range } else { in_range };
            if matched { GbnfExpr::Empty } else { GbnfExpr::Fail }
        }
        GbnfExpr::RuleRef(id) => derive_byte(&rules[*id], rules, b, depth + 1),
        GbnfExpr::Star(inner) => {
            let d_inner = derive_byte(inner, rules, b, depth + 1);
            simplify_seq(vec![d_inner, GbnfExpr::Star(inner.clone())])
        }
        GbnfExpr::Alt(choices) => {
            let mut next_choices = Vec::new();
            for c in choices {
                let dc = derive_byte(c, rules, b, depth + 1);
                if dc != GbnfExpr::Fail { next_choices.push(dc); }
            }
            simplify_alt(next_choices)
        }
        GbnfExpr::Seq(items) => {
            if items.is_empty() { return GbnfExpr::Fail; }
            let first = &items[0];
            let rest = &items[1..];
            let d_first = derive_byte(first, rules, b, depth + 1);
            let mut seq_branch = Vec::with_capacity(1 + rest.len());
            seq_branch.push(d_first);
            seq_branch.extend_from_slice(rest);
            let branch_a = simplify_seq(seq_branch);
            if is_nullable(first, rules, depth + 1) {
                let rest_expr = simplify_seq(rest.to_vec());
                let branch_b = derive_byte(&rest_expr, rules, b, depth + 1);
                simplify_alt(vec![branch_a, branch_b])
            } else {
                branch_a
            }
        }
    }
}

fn simplify_seq(items: Vec<GbnfExpr>) -> GbnfExpr {
    let mut flat = Vec::with_capacity(items.len());
    for item in items {
        match item {
            GbnfExpr::Fail => return GbnfExpr::Fail,
            GbnfExpr::Empty => {}
            GbnfExpr::Seq(sub) => flat.extend(sub),
            other => flat.push(other),
        }
    }
    match flat.len() {
        0 => GbnfExpr::Empty,
        1 => flat.pop().unwrap(),
        _ => GbnfExpr::Seq(flat),
    }
}

fn simplify_alt(choices: Vec<GbnfExpr>) -> GbnfExpr {
    let mut flat = Vec::with_capacity(choices.len());
    for choice in choices {
        match choice {
            GbnfExpr::Fail => {}
            GbnfExpr::Alt(sub) => {
                for s in sub {
                    if !flat.contains(&s) { flat.push(s); }
                }
            }
            other => {
                if !flat.contains(&other) { flat.push(other); }
            }
        }
    }
    match flat.len() {
        0 => GbnfExpr::Fail,
        1 => flat.pop().unwrap(),
        _ => GbnfExpr::Alt(flat),
    }
}

fn parse_rule_rhs(src: &str, name_to_id: &HashMap<String, usize>) -> Result<GbnfExpr, String> {
    let bytes = src.as_bytes();
    let mut pos = 0usize;
    parse_alt(bytes, &mut pos, name_to_id)
}

fn parse_alt(bytes: &[u8], pos: &mut usize, name_to_id: &HashMap<String, usize>) -> Result<GbnfExpr, String> {
    let mut branches = vec![parse_seq(bytes, pos, name_to_id)?];
    loop {
        skip_spaces(bytes, pos);
        if *pos < bytes.len() && bytes[*pos] == b'|' {
            *pos += 1;
            branches.push(parse_seq(bytes, pos, name_to_id)?);
        } else { break; }
    }
    Ok(simplify_alt(branches))
}

fn parse_seq(bytes: &[u8], pos: &mut usize, name_to_id: &HashMap<String, usize>) -> Result<GbnfExpr, String> {
    let mut items = Vec::new();
    loop {
        skip_spaces(bytes, pos);
        if *pos >= bytes.len() || bytes[*pos] == b'|' || bytes[*pos] == b')' { break; }
        let atom = parse_atom(bytes, pos, name_to_id)?;
        skip_spaces(bytes, pos);
        let quantified = if *pos < bytes.len() {
            match bytes[*pos] {
                b'*' => { *pos += 1; GbnfExpr::Star(Box::new(atom)) }
                b'+' => { *pos += 1; GbnfExpr::Seq(vec![atom.clone(), GbnfExpr::Star(Box::new(atom))]) }
                b'?' => { *pos += 1; simplify_alt(vec![atom, GbnfExpr::Empty]) }
                _ => atom,
            }
        } else { atom };
        items.push(quantified);
    }
    Ok(simplify_seq(items))
}

fn parse_atom(bytes: &[u8], pos: &mut usize, name_to_id: &HashMap<String, usize>) -> Result<GbnfExpr, String> {
    skip_spaces(bytes, pos);
    if *pos >= bytes.len() { return Ok(GbnfExpr::Empty); }
    match bytes[*pos] {
        b'(' => {
            *pos += 1;
            let inner = parse_alt(bytes, pos, name_to_id)?;
            skip_spaces(bytes, pos);
            if *pos < bytes.len() && bytes[*pos] == b')' { *pos += 1; }
            Ok(inner)
        }
        b'"' => {
            *pos += 1;
            let mut lit = Vec::new();
            while *pos < bytes.len() && bytes[*pos] != b'"' {
                if bytes[*pos] == b'\\' && *pos + 1 < bytes.len() {
                    *pos += 1;
                    lit.push(match bytes[*pos] { b'n'=>b'\n', b'r'=>b'\r', b't'=>b'\t', b'"'=>b'"', b'\\'=>b'\\', other=>other });
                } else { lit.push(bytes[*pos]); }
                *pos += 1;
            }
            if *pos < bytes.len() && bytes[*pos] == b'"' { *pos += 1; }
            Ok(GbnfExpr::Literal(lit))
        }
        b'[' => {
            *pos += 1;
            let negated = if *pos < bytes.len() && bytes[*pos] == b'^' { *pos += 1; true } else { false };
            let mut ranges = Vec::new();
            while *pos < bytes.len() && bytes[*pos] != b']' {
                let start_b = parse_class_byte(bytes, pos);
                if *pos + 1 < bytes.len() && bytes[*pos] == b'-' && bytes[*pos + 1] != b']' {
                    *pos += 1;
                    let end_b = parse_class_byte(bytes, pos);
                    ranges.push((start_b.min(end_b), start_b.max(end_b)));
                } else { ranges.push((start_b, start_b)); }
            }
            if *pos < bytes.len() && bytes[*pos] == b']' { *pos += 1; }
            Ok(GbnfExpr::CharClass { ranges, negated })
        }
        _ => {
            let start = *pos;
            while *pos < bytes.len() && (bytes[*pos].is_ascii_alphanumeric() || bytes[*pos] == b'-' || bytes[*pos] == b'_') { *pos += 1; }
            let ident = String::from_utf8_lossy(&bytes[start..*pos]).to_string();
            let id = name_to_id.get(&ident).copied().ok_or_else(|| format!("Undefined GBNF rule: '{ident}'"))?;
            Ok(GbnfExpr::RuleRef(id))
        }
    }
}

fn parse_class_byte(bytes: &[u8], pos: &mut usize) -> u8 {
    if bytes[*pos] == b'\\' && *pos + 1 < bytes.len() {
        *pos += 1;
        let b = match bytes[*pos] { b'n'=>b'\n', b'r'=>b'\r', b't'=>b'\t', other=>other };
        *pos += 1;
        b
    } else {
        let b = bytes[*pos];
        *pos += 1;
        b
    }
}

fn skip_spaces(bytes: &[u8], pos: &mut usize) {
    while *pos < bytes.len() && (bytes[*pos] == b' ' || bytes[*pos] == b'\t') { *pos += 1; }
}
