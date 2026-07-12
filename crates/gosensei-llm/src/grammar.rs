use crate::parse::KNOWN_ERROR_CLASSES;

/// Name of the root rule in the coaching GBNF grammar.
pub const GRAMMAR_ROOT: &str = "root";

/// Maximum coaching text length enforced by the grammar, in characters.
/// Matches the cap applied by `parse::sanitize_coaching_text`.
pub const MAX_COACHING_CHARS: usize = 500;

/// Build a GBNF grammar that constrains decoding to the tagged coaching format:
///
/// ```text
/// <classification>{"error_class": "Direction"}</classification>
/// <coaching>1-3 sentences of coaching text.</coaching>
/// ```
///
/// The error class is restricted to `parse::KNOWN_ERROR_CLASSES` and the
/// coaching text is bounded in length and cannot contain `<` or `>` (so the
/// closing tag is unambiguous). `parse::parse_llm_output` remains the second
/// line of defense with unchanged semantics.
pub fn coaching_grammar() -> String {
    let classes = KNOWN_ERROR_CLASSES
        .iter()
        .map(|class| format!("\"{class}\""))
        .collect::<Vec<_>>()
        .join(" | ");

    format!(
        r#"{GRAMMAR_ROOT} ::= "<classification>{{\"error_class\": \"" class "\"}}</classification>\n<coaching>" text "</coaching>"
class ::= {classes}
text ::= [^<>]{{1,{MAX_COACHING_CHARS}}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_embeds_every_known_error_class() {
        let grammar = coaching_grammar();
        for class in KNOWN_ERROR_CLASSES {
            assert!(
                grammar.contains(&format!("\"{class}\"")),
                "grammar missing class {class}"
            );
        }
    }

    #[test]
    fn grammar_defines_root_rule() {
        let grammar = coaching_grammar();
        assert!(grammar.starts_with(&format!("{GRAMMAR_ROOT} ::= ")));
        // llama.cpp requires the root rule name to appear in the grammar string
        assert!(grammar.contains(GRAMMAR_ROOT));
    }

    #[test]
    fn grammar_enforces_tagged_shape() {
        let grammar = coaching_grammar();
        assert!(grammar.contains(r#""<classification>{\"error_class\": \""#));
        assert!(grammar.contains(r#""\"}</classification>\n<coaching>""#));
        assert!(grammar.contains(r#""</coaching>""#));
    }

    #[test]
    fn grammar_bounds_coaching_text_length() {
        let grammar = coaching_grammar();
        assert!(grammar.contains(&format!("[^<>]{{1,{MAX_COACHING_CHARS}}}")));
    }

    #[test]
    fn grammar_class_alternatives_match_known_count() {
        let grammar = coaching_grammar();
        let class_rule = grammar
            .lines()
            .find(|l| l.starts_with("class ::= "))
            .expect("class rule present");
        let alternatives = class_rule.trim_start_matches("class ::= ").split(" | ");
        assert_eq!(alternatives.count(), KNOWN_ERROR_CLASSES.len());
    }

    #[test]
    fn grammar_has_no_null_bytes() {
        // llama-cpp-2 rejects grammar strings containing NUL
        assert!(!coaching_grammar().contains('\0'));
    }
}
