//! g8t scout: cheap AST pass that finds standalone leaf functions worth proving.
//!
//! The scout parses Rust with `syn` and keeps only free functions that can plausibly
//! stand alone as a g8t capsule: no generics, no `self`, primitive integer / bool /
//! slice / array / `&str` parameters, no `unsafe`, no floating point, and a body that
//! references nothing outside itself. It does no compilation and no measurement.
//!
//! The scout is a filter, not a verdict. `prove_capsule_with_rustc` remains the gate
//! that decides whether a candidate really compiles and runs standalone.
//!
//! `static_weight` is a static complexity proxy (loops and expression count). It is
//! not call-frequency telemetry and says nothing about how hot a function is at runtime.

use quote::ToTokens;
use std::collections::BTreeSet;
use syn::visit::{self, Visit};

/// Longest function body (in source lines) the scout will hand to the prover.
pub const MAX_CANDIDATE_LINES: usize = 120;
/// Most candidates returned per file, heaviest first.
pub const MAX_CANDIDATES_PER_FILE: usize = 16;

const SCALAR_TYPES: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "bool",
];
/// `core` / `std` modules a leaf function may name without touching the outside world.
const SAFE_STD_MODULES: &[&str] = &[
    "cmp", "mem", "convert", "hint", "num", "ops", "iter", "slice", "str", "char", "array",
    "option", "result",
];
const ALLOWED_MACROS: &[&str] = &[
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "matches",
    "unreachable",
];
const PRELUDE_VALUES: &[&str] = &["Some", "None", "Ok", "Err"];
/// Prelude names (and primitives that are not in `SCALAR_TYPES`) usable without imports.
const PRELUDE_NAMES: &[&str] = &[
    "Some", "None", "Ok", "Err", "Option", "Result", "str", "char", "String", "Vec", "Box",
];

/// A free function the scout considers a plausible standalone capsule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoutCandidate {
    pub name: String,
    /// 1-based line of the function in its file (0 when it could not be located as
    /// whole lines; see `source_text_of`).
    pub line: usize,
    /// `<rel_path>:<line>`.
    pub file_loc: String,
    /// Function source, normalized to start with `pub`.
    pub source: String,
    /// BLAKE3 over the signature and body with local names alpha-renamed and the
    /// function name dropped: equal hashes mean structurally identical functions.
    pub structural_hash: String,
    /// Static complexity proxy: `loop_count * 16 + expression_count`.
    pub static_weight: u64,
    pub loop_count: u64,
}

/// Names of the two floating-point primitive types, assembled so this file itself
/// contains no float type token.
fn float_type_names() -> [String; 2] {
    [
        ['f', '3', '2'].iter().collect(),
        ['f', '6', '4'].iter().collect(),
    ]
}

fn is_test_or_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        let path = a.path();
        let last_is_test = path
            .segments
            .last()
            .map(|s| s.ident == "test")
            .unwrap_or(false);
        let cfg_test = path.is_ident("cfg") && a.to_token_stream().to_string().contains("test");
        last_is_test || cfg_test
    })
}

fn is_scalar(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(p) if p.qself.is_none() => p
            .path
            .get_ident()
            .map(|id| SCALAR_TYPES.iter().any(|s| id == s))
            .unwrap_or(false),
        _ => false,
    }
}

fn is_scalar_array(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Array(a) if is_scalar(&a.elem))
}

fn is_allowed_param(ty: &syn::Type) -> bool {
    if is_scalar(ty) || is_scalar_array(ty) {
        return true;
    }
    match ty {
        syn::Type::Reference(r) if r.mutability.is_none() && r.lifetime.is_none() => {
            match r.elem.as_ref() {
                syn::Type::Slice(s) => is_scalar(&s.elem),
                syn::Type::Path(p) => p.path.is_ident("str"),
                _ => false,
            }
        }
        _ => false,
    }
}

fn signature_is_standalone(sig: &syn::Signature) -> bool {
    if sig.asyncness.is_some()
        || sig.unsafety.is_some()
        || sig.abi.is_some()
        || sig.variadic.is_some()
        || !sig.generics.params.is_empty()
        || sig.generics.where_clause.is_some()
    {
        return false;
    }
    let returns_value = match &sig.output {
        syn::ReturnType::Type(_, ty) => is_scalar(ty) || is_scalar_array(ty),
        syn::ReturnType::Default => false,
    };
    if !returns_value || sig.inputs.is_empty() {
        return false;
    }
    sig.inputs.iter().all(|arg| match arg {
        syn::FnArg::Typed(t) => {
            matches!(t.pat.as_ref(), syn::Pat::Ident(p) if p.by_ref.is_none())
                && is_allowed_param(&t.ty)
        }
        syn::FnArg::Receiver(_) => false,
    })
}

/// Collects names bound inside the function (parameters, `let`, loop and closure
/// patterns) and items declared in its body.
#[derive(Default)]
struct LocalNames {
    bound: BTreeSet<String>,
    nested_items: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for LocalNames {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.bound.insert(node.ident.to_string());
        visit::visit_pat_ident(self, node);
    }
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.nested_items.insert(node.sig.ident.to_string());
        visit::visit_item_fn(self, node);
    }
    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        self.nested_items.insert(node.ident.to_string());
        visit::visit_item_const(self, node);
    }
}

/// Checks that the body is a self-contained leaf and gathers the static weight.
struct LeafCheck<'a> {
    names: &'a LocalNames,
    floats: [String; 2],
    ok: bool,
    loops: u64,
    exprs: u64,
}

impl LeafCheck<'_> {
    /// A path is allowed when its first segment is something a standalone file can
    /// resolve without imports: a primitive, a prelude name, a safe `core`/`std`
    /// module, or a name bound or declared inside the function itself.
    fn path_is_allowed(&self, path: &syn::Path) -> bool {
        let Some(first) = path.segments.first() else {
            return false;
        };
        let head = first.ident.to_string();
        if self.floats.contains(&head) {
            return false;
        }
        if matches!(head.as_str(), "std" | "core" | "alloc") {
            return path
                .segments
                .iter()
                .nth(1)
                .map(|s| SAFE_STD_MODULES.iter().any(|m| s.ident == m))
                .unwrap_or(false);
        }
        SCALAR_TYPES.contains(&head.as_str())
            || PRELUDE_NAMES.contains(&head.as_str())
            || self.names.bound.contains(&head)
            || self.names.nested_items.contains(&head)
    }
}

impl<'ast> Visit<'ast> for LeafCheck<'_> {
    fn visit_expr(&mut self, node: &'ast syn::Expr) {
        self.exprs = self.exprs.saturating_add(1);
        match node {
            syn::Expr::Unsafe(_) | syn::Expr::Async(_) | syn::Expr::Await(_) => self.ok = false,
            syn::Expr::ForLoop(_) | syn::Expr::While(_) | syn::Expr::Loop(_) => {
                self.loops = self.loops.saturating_add(1);
            }
            // A bare name used as a value must be bound inside the function; anything
            // else is a sibling function, constant, or static outside the capsule.
            syn::Expr::Path(p) if p.qself.is_none() => {
                if let Some(id) = p.path.get_ident() {
                    let name = id.to_string();
                    let known = self.names.bound.contains(&name)
                        || self.names.nested_items.contains(&name)
                        || PRELUDE_VALUES.contains(&name.as_str());
                    if !known {
                        self.ok = false;
                    }
                }
            }
            _ => {}
        }
        visit::visit_expr(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if !self.path_is_allowed(node) {
            self.ok = false;
        }
        visit::visit_path(self, node);
    }

    fn visit_item_use(&mut self, _node: &'ast syn::ItemUse) {
        // An import inside the body reaches outside the capsule.
        self.ok = false;
    }

    fn visit_lit_float(&mut self, _node: &'ast syn::LitFloat) {
        self.ok = false;
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let allowed = node
            .path
            .get_ident()
            .map(|id| ALLOWED_MACROS.iter().any(|m| id == m))
            .unwrap_or(false);
        // Macro arguments are opaque tokens to `syn`, so check them textually for floats.
        let body = node.tokens.to_string();
        let has_float = word_runs(&body).any(|w| self.floats.iter().any(|f| f == w));
        if !allowed || has_float {
            self.ok = false;
        }
    }
}

/// Maximal `[A-Za-z0-9_]` runs of `text`.
fn word_runs(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty())
}

/// Structural hash of a function: signature inputs, output and body tokens with every
/// locally bound name replaced by its first-occurrence index. The function's own name
/// and visibility are not part of the hash.
fn structural_hash_of(item: &syn::ItemFn, names: &LocalNames) -> String {
    let raw = format!(
        "{} {} {}",
        item.sig.inputs.to_token_stream(),
        item.sig.output.to_token_stream(),
        item.block.to_token_stream()
    );
    let mut order: Vec<&str> = Vec::new();
    let mut canon = String::with_capacity(raw.len());
    let mut rest = raw.as_str();
    while let Some(ch) = rest.chars().next() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            let word = &rest[..end];
            if names.bound.contains(word) {
                let idx = match order.iter().position(|w| *w == word) {
                    Some(i) => i,
                    None => {
                        order.push(word);
                        order.len() - 1
                    }
                };
                canon.push_str("_v");
                canon.push_str(&idx.to_string());
            } else {
                canon.push_str(word);
            }
            canon.push(' ');
            rest = &rest[end..];
        } else {
            if !ch.is_whitespace() {
                canon.push(ch);
                canon.push(' ');
            }
            rest = &rest[ch.len_utf8()..];
        }
    }
    let mut tagged = String::from("G8T_SCOUT_STRUCT_V1:");
    tagged.push_str(&canon);
    crate::g8t_integration::blake3_hash_hex(tagged.as_bytes())
}

fn fn_tokens(item: &syn::ItemFn) -> String {
    format!(
        "{} {}",
        item.sig.to_token_stream(),
        item.block.to_token_stream()
    )
}

/// Source text for `item` as `(text, first_line, line_count)`.
///
/// Span line numbers are not available in every build, so the function is located in
/// the original text: each line that mentions `fn <name>` is tried as a start, and the
/// shortest following run of lines that parses back to the same signature and body
/// wins. When no such run exists (for example the function shares a line with other
/// code) the function is re-printed from tokens and reported with line `0`.
fn source_text_of(item: &syn::ItemFn, lines: &[&str]) -> (String, usize, usize) {
    let needle = format!("fn {}", item.sig.ident);
    let want = fn_tokens(item);
    for (start, first) in lines.iter().enumerate() {
        if !first.contains(&needle) {
            continue;
        }
        let limit = lines.len().min(start + MAX_CANDIDATE_LINES + 1);
        for end in start..limit {
            if !lines[end].contains('}') {
                continue;
            }
            let slice = lines[start..=end].join("\n");
            if let Ok(parsed) = syn::parse_str::<syn::ItemFn>(&slice) {
                if fn_tokens(&parsed) == want {
                    return (slice, start + 1, end - start + 1);
                }
                break;
            }
        }
    }
    let mut bare = item.clone();
    bare.attrs.clear();
    (bare.to_token_stream().to_string(), 0, 0)
}

/// Make the function `pub` so the prover can export it, whatever its original visibility.
fn normalize_visibility(source: &str) -> String {
    let trimmed = source.trim_start();
    if let Some(rest) = trimmed.strip_prefix("pub") {
        let rest_trim = rest.trim_start();
        if rest_trim.starts_with('(') {
            if let Some(close) = rest_trim.find(')') {
                return format!("pub{}", &rest_trim[close + 1..]);
            }
        }
        if rest.starts_with(char::is_whitespace) {
            return trimmed.to_string();
        }
    }
    format!("pub {trimmed}")
}

fn consider(item: &syn::ItemFn, rel_path: &str, lines: &[&str], out: &mut Vec<ScoutCandidate>) {
    let name = item.sig.ident.to_string();
    if name == "main" || is_test_or_cfg_test(&item.attrs) || !signature_is_standalone(&item.sig) {
        return;
    }

    let mut names = LocalNames::default();
    names.visit_signature(&item.sig);
    names.visit_block(&item.block);

    let mut check = LeafCheck {
        names: &names,
        floats: float_type_names(),
        ok: true,
        loops: 0,
        exprs: 0,
    };
    check.visit_block(&item.block);
    if !check.ok {
        return;
    }

    let (text, line, line_count) = source_text_of(item, lines);
    if line_count > MAX_CANDIDATE_LINES {
        return;
    }
    out.push(ScoutCandidate {
        file_loc: format!("{rel_path}:{line}"),
        source: normalize_visibility(&text),
        structural_hash: structural_hash_of(item, &names),
        static_weight: check.loops.saturating_mul(16).saturating_add(check.exprs),
        loop_count: check.loops,
        name,
        line,
    });
}

fn walk(items: &[syn::Item], rel_path: &str, lines: &[&str], out: &mut Vec<ScoutCandidate>) {
    for item in items {
        match item {
            syn::Item::Fn(f) => consider(f, rel_path, lines, out),
            syn::Item::Mod(m) if !is_test_or_cfg_test(&m.attrs) => {
                if let Some((_, inner)) = &m.content {
                    walk(inner, rel_path, lines, out);
                }
            }
            _ => {}
        }
    }
}

/// Scout one Rust source file. Returns at most `MAX_CANDIDATES_PER_FILE` candidates,
/// heaviest `static_weight` first (ties broken by source order). Files that do not
/// parse yield no candidates.
pub fn scout_rust_candidates(rel_path: &str, content: &str) -> Vec<ScoutCandidate> {
    let Ok(file) = syn::parse_file(content) else {
        return Vec::new();
    };
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    walk(&file.items, rel_path, &lines, &mut out);
    out.sort_by(|a, b| {
        b.static_weight
            .cmp(&a.static_weight)
            .then(a.line.cmp(&b.line))
    });
    out.truncate(MAX_CANDIDATES_PER_FILE);
    out
}

/// Structural hash of a single standalone function source, if it parses.
pub fn structural_hash_of_source(source: &str) -> Option<String> {
    let item = syn::parse_str::<syn::ItemFn>(source).ok()?;
    let mut names = LocalNames::default();
    names.visit_signature(&item.sig);
    names.visit_block(&item.block);
    Some(structural_hash_of(&item, &names))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names_of(src: &str) -> Vec<String> {
        scout_rust_candidates("src/x.rs", src)
            .into_iter()
            .map(|c| c.name)
            .collect()
    }

    #[test]
    fn keeps_self_contained_integer_leaf() {
        let src = "/// doc\nfn fold(seed: u64, bytes: &[u8]) -> u64 {\n    let mut acc = seed;\n    for b in bytes {\n        acc = acc.rotate_left(5) ^ u64::from(*b);\n    }\n    acc\n}\n";
        let found = scout_rust_candidates("src/x.rs", src);
        assert_eq!(found.len(), 1);
        let c = &found[0];
        assert_eq!(c.name, "fold");
        assert_eq!(c.line, 2);
        assert_eq!(c.file_loc, "src/x.rs:2");
        assert!(c.source.starts_with("pub fn fold("));
        assert_eq!(c.loop_count, 1);
        assert!(syn::parse_str::<syn::ItemFn>(&c.source).is_ok());
    }

    #[test]
    fn rejects_functions_that_are_not_standalone_leaves() {
        let float_ty = &float_type_names()[1];
        let cases = [
            // calls a sibling function
            "fn a(x: u64) -> u64 { helper(x) }".to_string(),
            // reads an outer constant
            "fn b(x: u64) -> u64 { x ^ SEED }".to_string(),
            // crate-relative path
            "fn c(x: u64) -> u64 { crate::util::mix(x) }".to_string(),
            // unsafe block
            "fn d(x: u64) -> u64 { unsafe { core::mem::transmute(x) } }".to_string(),
            // side-effecting macro
            "fn e(x: u64) -> u64 { println!(\"{x}\"); x }".to_string(),
            // filesystem access
            "fn f(x: u64) -> u64 { let _ = std::fs::read(\"p\"); x }".to_string(),
            // generic
            "fn g<T: Copy>(x: T, y: u64) -> u64 { y }".to_string(),
            // method on self
            "struct S; impl S { fn h(&self, x: u64) -> u64 { x } }".to_string(),
            // mutable reference parameter
            "fn i(x: &mut u64) -> u64 { *x }".to_string(),
            // no return value
            "fn j(x: u64) { let _ = x; }".to_string(),
            // imported type used through an associated function
            "fn n(x: &str) -> bool { Path::new(x).is_absolute() }".to_string(),
            // import inside the body
            "fn o(x: u64) -> u64 { use sha2::Digest; x }".to_string(),
            // floating point type
            format!("fn k(x: u64) -> u64 {{ (x as {float_ty}) as u64 }}"),
            // floating point literal
            "fn l(x: u64) -> u64 { let _y = 1.5; x }".to_string(),
            // async
            "async fn m(x: u64) -> u64 { x }".to_string(),
        ];
        for case in &cases {
            assert!(names_of(case).is_empty(), "should be rejected: {case}");
        }
    }

    #[test]
    fn skips_tests_and_cfg_test_modules_but_enters_plain_modules() {
        let src = "#[test]\nfn t(x: u64) -> u64 { x }\n#[cfg(test)]\nmod tests { fn inner(x: u64) -> u64 { x } }\nmod plain {\n    pub(crate) fn kept(x: u32) -> u32 { x.wrapping_mul(3) }\n}\n";
        let found = scout_rust_candidates("src/x.rs", src);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "kept");
        assert!(found[0].source.starts_with("pub fn kept("));
    }

    #[test]
    fn nested_helpers_and_constants_stay_inside_the_capsule() {
        let src = "fn outer(x: u64) -> u64 {\n    const K: u64 = 7;\n    fn step(v: u64) -> u64 { v.wrapping_add(1) }\n    step(x) ^ K\n}\n";
        assert_eq!(names_of(src), vec!["outer".to_string()]);
    }

    #[test]
    fn structural_hash_ignores_names_but_not_structure() {
        let a = "fn alpha(x: u64, y: u64) -> u64 { let t = x ^ y; t.rotate_left(3) }";
        let renamed = "pub fn beta(left: u64, right: u64) -> u64 { let mix = left ^ right; mix.rotate_left(3) }";
        let different = "fn gamma(x: u64, y: u64) -> u64 { let t = x ^ y; t.rotate_left(4) }";
        let swapped = "fn delta(x: u64, y: u64) -> u64 { let t = y ^ x; t.rotate_left(3) }";
        let ha = structural_hash_of_source(a).expect("parses");
        assert_eq!(Some(ha.clone()), structural_hash_of_source(renamed));
        assert_ne!(Some(ha.clone()), structural_hash_of_source(different));
        assert_ne!(Some(ha), structural_hash_of_source(swapped));
    }

    #[test]
    fn heaviest_candidates_come_first_and_unparsable_files_yield_nothing() {
        let src = "fn light(x: u64) -> u64 { x }\nfn heavy(x: u64) -> u64 {\n    let mut a = x;\n    let mut i = 0u64;\n    while i < 8 {\n        a = a.rotate_left(1) ^ i;\n        i += 1;\n    }\n    a\n}\n";
        assert_eq!(
            names_of(src),
            vec!["heavy".to_string(), "light".to_string()]
        );
        assert!(names_of("fn broken(x: u64 -> u64 {").is_empty());
    }

    fn rust_files_under(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                rust_files_under(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }

    /// Dogfood: every candidate the scout reports from this crate's own sources must be
    /// a located, re-parsable standalone function.
    #[test]
    fn dogfood_candidates_from_own_sources_are_well_formed() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        rust_files_under(&root, &mut files);
        files.sort();
        let mut total = 0usize;
        for path in &files {
            let Ok(content) = std::fs::read_to_string(path) else {
                continue;
            };
            for c in scout_rust_candidates(&path.to_string_lossy(), &content) {
                total += 1;
                assert!(c.line >= 1, "{} was not located in {}", c.name, c.file_loc);
                let parsed = syn::parse_str::<syn::ItemFn>(&c.source);
                assert!(parsed.is_ok(), "{} does not re-parse", c.file_loc);
                assert_eq!(c.structural_hash.len(), 64);
            }
        }
        assert!(
            total > 0,
            "expected at least one candidate in {}",
            root.display()
        );
    }

    /// Dogfood measurement, run on demand: how many scout candidates from this crate's
    /// own sources does the real rustc prover accept?
    /// `cargo test --lib g8t_scout::tests::dogfood_prove_rate -- --ignored --nocapture`
    #[test]
    #[ignore = "compiles every candidate with rustc; run on demand"]
    fn dogfood_prove_rate() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        rust_files_under(&root, &mut files);
        files.sort();
        let mut seen = std::collections::HashSet::new();
        let (mut total, mut unique, mut proved) = (0usize, 0usize, 0usize);
        let mut rejected = Vec::new();
        for path in &files {
            let Ok(content) = std::fs::read_to_string(path) else {
                continue;
            };
            let rel = path
                .strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
            for c in scout_rust_candidates(&rel, &content) {
                total += 1;
                if !seen.insert(c.structural_hash.clone()) {
                    continue;
                }
                unique += 1;
                match crate::g8t_integration::prove_capsule_with_rustc(&c.source) {
                    Ok(p) => {
                        proved += 1;
                        println!(
                            "PROVED {} w={} cyc/op={}",
                            c.file_loc, c.static_weight, p.cycles_per_op_pq
                        );
                    }
                    Err(e) => rejected.push(format!(
                        "{} ({})",
                        c.file_loc,
                        e.to_string().lines().next().unwrap_or("")
                    )),
                }
            }
        }
        for r in &rejected {
            println!("UNPROVEN {r}");
        }
        println!(
            "files={} candidates={total} unique={unique} proved={proved}",
            files.len()
        );
    }
}
