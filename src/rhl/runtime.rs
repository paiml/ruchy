//! RHL-4b: the ruchy runtime a vocabulary term's `lowers_to` names (spec
//! RHL-001 §4, Amendment A4). A binding `<module>::<name>` names the function
//! `fun <name>` in `<root>/vocab/runtime/<module>.ruchy`; lowering copies that
//! one function into the generated source. `[U]` is the unbound marker: no
//! local source of truth exists, and a job that needs one is refused by name.

use std::path::Path;

/// The unbound marker of a vocabulary field.
pub const UNBOUND: &str = "[U]";

/// Split a binding into `(module, function)`: both lowercase ASCII words, so
/// a binding can name no path outside `vocab/runtime/`.
#[must_use]
pub fn parse_binding(binding: &str) -> Option<(&str, &str)> {
    let (module, name) = binding.split_once("::")?;
    let word = |w: &str| {
        !w.is_empty()
            && w.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    };
    (word(module) && word(name)).then_some((module, name))
}

/// The source of the function `binding` names, from `<root>/vocab/runtime/`:
/// from its `fun <name>(` line through the first line that is exactly `}`.
///
/// # Errors
///
/// A message naming the binding when it is `[U]`, malformed, its runtime file
/// cannot be read, or the file defines no such function.
pub fn binding_function(root: &Path, binding: &str) -> Result<String, String> {
    if binding == UNBOUND {
        return Err("the binding is `[U]`: no local source of truth".to_string());
    }
    let (module, name) =
        parse_binding(binding).ok_or_else(|| format!("`{binding}` is not `<module>::<name>`"))?;
    let path = root.join("vocab/runtime").join(format!("{module}.ruchy"));
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("`{binding}`: cannot read {}: {e}", path.display()))?;
    extract(&text, name)
        .ok_or_else(|| format!("`{binding}`: {} has no `fun {name}`", path.display()))
}

fn extract(text: &str, name: &str) -> Option<String> {
    let head = format!("fun {name}(");
    let mut lines = text.lines().skip_while(|l| !l.starts_with(&head));
    let first = lines.next()?;
    let mut out = format!("{first}\n");
    for line in lines {
        out.push_str(line);
        out.push('\n');
        if line == "}" {
            return Some(out);
        }
    }
    None
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod runtime_tests;
