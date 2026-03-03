use crate::types::LlmCoachingOutput;

/// Known error classes that the LLM might return.
const KNOWN_ERROR_CLASSES: &[&str] = &[
    "Direction",
    "Shape",
    "Reading",
    "LifeAndDeath",
    "Endgame",
    "Opening",
    "Ko",
];

/// Valid GTP column letters (A-T, skipping I).
const GTP_COLUMNS: &[u8] = b"ABCDEFGHJKLMNOPQRST";

/// Valid Go terms that should appear in quality coaching output.
/// Used to gauge whether LLM output is actually Go-relevant.
const GO_TERMS: &[&str] = &[
    // Stones and groups
    "stone", "stones", "group", "groups", "chain",
    // Board areas
    "corner", "side", "center", "edge", "territory", "influence",
    // Tactics
    "atari", "capture", "connect", "cut", "extend", "attach", "hane",
    "push", "peep", "probe", "block", "jump", "keima", "kosumi",
    "nobi", "tenuki", "sente", "gote",
    // Strategy
    "shape", "eye", "eyes", "life", "death", "live", "dead", "alive",
    "ko", "ladder", "net", "snapback", "tesuji", "joseki", "fuseki",
    "endgame", "yose", "miai", "aji", "thickness", "thinness",
    "overplay", "direction", "opening", "approach", "invasion",
    "reduction", "moyo", "framework",
    // General move terms
    "move", "play", "defend", "attack", "respond", "response",
    "sacrifice", "exchange", "sequence", "variation",
    // Points and scoring
    "point", "points", "komi",
];

/// Parse structured output from the LLM response.
///
/// Expected format:
/// ```text
/// <classification>{"error_class": "Direction"}</classification>
/// <coaching>Your 1-3 sentence message.</coaching>
/// ```
///
/// Falls back gracefully: if tags are missing, uses the full text as coaching.
pub fn parse_llm_output(raw: &str) -> LlmCoachingOutput {
    let error_class = extract_between(raw, "<classification>", "</classification>")
        .and_then(|json| {
            serde_json::from_str::<serde_json::Value>(json).ok()
        })
        .and_then(|val| val.get("error_class")?.as_str().map(String::from))
        .filter(|class| validate_error_class(class).is_some());

    let coaching_text = extract_between(raw, "<coaching>", "</coaching>")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| {
            // Fallback: strip any tags and use the raw text
            let stripped = raw
                .replace("<classification>", "")
                .replace("</classification>", "")
                .replace("<coaching>", "")
                .replace("</coaching>", "");
            stripped.trim().to_string()
        });

    let coaching_text = sanitize_coordinates_in_text(&coaching_text);
    let coaching_text = sanitize_coaching_text(&coaching_text, 500);

    LlmCoachingOutput {
        error_class,
        coaching_text,
    }
}

/// Validate that an error class string matches a known category.
pub fn validate_error_class(class: &str) -> Option<&str> {
    KNOWN_ERROR_CLASSES
        .iter()
        .find(|&&known| known == class)
        .copied()
}

/// Sanitize coaching text: cap length at the last sentence boundary within max_len.
pub fn sanitize_coaching_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    // Find the last sentence-ending punctuation within max_len
    let truncated = &text[..max_len];
    let last_sentence_end = truncated
        .rfind(". ")
        .or_else(|| truncated.rfind("! "))
        .or_else(|| truncated.rfind("? "))
        .or_else(|| truncated.rfind('.'))
        .or_else(|| truncated.rfind('!'))
        .or_else(|| truncated.rfind('?'));

    match last_sentence_end {
        Some(pos) => text[..=pos].to_string(),
        None => format!("{}...", &text[..max_len.saturating_sub(3)]),
    }
}

/// Validate that a coordinate string is valid GTP notation for a Go board.
///
/// Valid format: column letter (A-T, no I) + row number (1-19).
/// Examples: "D4", "Q16", "C3"
pub fn validate_coordinate(coord: &str) -> bool {
    let coord = coord.trim();
    if coord.len() < 2 || coord.len() > 3 {
        return false;
    }
    let bytes = coord.as_bytes();
    let col = bytes[0].to_ascii_uppercase();
    if !GTP_COLUMNS.contains(&col) {
        return false;
    }
    let row_str = &coord[1..];
    match row_str.parse::<u8>() {
        Ok(row) => (1..=19).contains(&row),
        Err(_) => false,
    }
}

/// Check whether coaching text contains Go-relevant terminology.
///
/// Returns true if at least one Go term is found, indicating the output
/// is likely about Go rather than hallucinated/off-topic content.
pub fn text_contains_go_terms(text: &str) -> bool {
    let lower = text.to_lowercase();
    GO_TERMS.iter().any(|term| {
        // Match whole words only (bounded by non-alphanumeric chars or string boundaries)
        lower
            .find(term)
            .map(|pos| {
                let before_ok =
                    pos == 0 || !lower.as_bytes()[pos - 1].is_ascii_alphanumeric();
                let after_pos = pos + term.len();
                let after_ok = after_pos >= lower.len()
                    || !lower.as_bytes()[after_pos].is_ascii_alphanumeric();
                before_ok && after_ok
            })
            .unwrap_or(false)
    })
}

/// Strip any hallucinated coordinates from coaching text.
///
/// Replaces coordinate-like patterns that are invalid with "[...]".
/// Keeps valid coordinates intact.
pub fn sanitize_coordinates_in_text(text: &str) -> String {
    // Match patterns that look like coordinates (letter + 1-2 digits)
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        // Check if this could be a coordinate: uppercase letter followed by 1-2 digits
        if chars[i].is_ascii_uppercase()
            && i + 1 < chars.len()
            && chars[i + 1].is_ascii_digit()
        {
            let start = i;
            let mut end = i + 2;
            if end < chars.len() && chars[end].is_ascii_digit() {
                end += 1;
            }
            // Only treat as coordinate if bounded (not inside a word)
            let before_ok = start == 0 || !chars[start - 1].is_ascii_alphanumeric();
            let after_ok = end >= chars.len() || !chars[end].is_ascii_alphanumeric();
            if before_ok && after_ok {
                let candidate: String = chars[start..end].iter().collect();
                if validate_coordinate(&candidate) {
                    result.push_str(&candidate);
                } else {
                    result.push_str("[...]");
                }
                i = end;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn extract_between<'a>(text: &'a str, start_tag: &str, end_tag: &str) -> Option<&'a str> {
    let start = text.find(start_tag)? + start_tag.len();
    let end = text[start..].find(end_tag)? + start;
    Some(&text[start..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_output() {
        let raw = r#"<classification>{"error_class": "Direction"}</classification>
<coaching>Playing on the third line in the opening is slow. Consider approaching the corner first to establish influence.</coaching>"#;

        let output = parse_llm_output(raw);
        assert_eq!(output.error_class.as_deref(), Some("Direction"));
        assert!(output.coaching_text.contains("third line"));
    }

    #[test]
    fn parse_missing_classification_falls_back() {
        let raw = "<coaching>This move gives up too much territory.</coaching>";
        let output = parse_llm_output(raw);
        assert_eq!(output.error_class, None);
        assert!(output.coaching_text.contains("territory"));
    }

    #[test]
    fn parse_no_tags_uses_raw_text() {
        let raw = "You should consider playing closer to the corner.";
        let output = parse_llm_output(raw);
        assert_eq!(output.error_class, None);
        assert_eq!(output.coaching_text, raw);
    }

    #[test]
    fn parse_invalid_json_graceful() {
        let raw = r#"<classification>not valid json</classification>
<coaching>Good try, but the shape is inefficient.</coaching>"#;

        let output = parse_llm_output(raw);
        assert_eq!(output.error_class, None);
        assert!(output.coaching_text.contains("shape is inefficient"));
    }

    #[test]
    fn parse_unknown_error_class_rejected() {
        let raw = r#"<classification>{"error_class": "Telepathy"}</classification>
<coaching>Interesting move.</coaching>"#;

        let output = parse_llm_output(raw);
        assert_eq!(output.error_class, None);
        assert_eq!(output.coaching_text, "Interesting move.");
    }

    #[test]
    fn sanitize_caps_length_at_sentence() {
        let text = "First sentence. Second sentence. Third sentence that is very long and should be truncated.";
        let result = sanitize_coaching_text(text, 40);
        assert_eq!(result, "First sentence. Second sentence.");
    }

    #[test]
    fn sanitize_short_text_unchanged() {
        let text = "Short text.";
        assert_eq!(sanitize_coaching_text(text, 500), "Short text.");
    }

    #[test]
    fn validate_error_class_known() {
        assert_eq!(validate_error_class("Direction"), Some("Direction"));
        assert_eq!(validate_error_class("Shape"), Some("Shape"));
        assert_eq!(validate_error_class("LifeAndDeath"), Some("LifeAndDeath"));
    }

    #[test]
    fn validate_error_class_unknown() {
        assert_eq!(validate_error_class("Telekinesis"), None);
        assert_eq!(validate_error_class(""), None);
    }

    #[test]
    fn validate_coordinate_valid() {
        assert!(validate_coordinate("D4"));
        assert!(validate_coordinate("Q16"));
        assert!(validate_coordinate("A1"));
        assert!(validate_coordinate("T19"));
        assert!(validate_coordinate("C3"));
    }

    #[test]
    fn validate_coordinate_invalid() {
        // I is skipped in GTP
        assert!(!validate_coordinate("I5"));
        // Out of range
        assert!(!validate_coordinate("A0"));
        assert!(!validate_coordinate("A20"));
        // Too short / wrong format
        assert!(!validate_coordinate("Z"));
        assert!(!validate_coordinate("99"));
        assert!(!validate_coordinate(""));
        assert!(!validate_coordinate("DD4"));
    }

    #[test]
    fn text_contains_go_terms_positive() {
        assert!(text_contains_go_terms("Playing on the third line gives up territory."));
        assert!(text_contains_go_terms("This move creates a dead group in the corner."));
        assert!(text_contains_go_terms("Consider a keima to extend your influence."));
    }

    #[test]
    fn text_contains_go_terms_negative() {
        assert!(!text_contains_go_terms("The weather is nice today."));
        assert!(!text_contains_go_terms("Hello world!"));
    }

    #[test]
    fn sanitize_coordinates_keeps_valid() {
        let text = "Playing at D4 instead of Q16 would be better.";
        assert_eq!(sanitize_coordinates_in_text(text), text);
    }

    #[test]
    fn sanitize_coordinates_strips_invalid() {
        let text = "Consider playing at Z99 for better shape.";
        let result = sanitize_coordinates_in_text(text);
        assert_eq!(result, "Consider playing at [...] for better shape.");
    }

    #[test]
    fn sanitize_coordinates_preserves_words_with_digits() {
        // Words like "1st" or abbreviations shouldn't be treated as coordinates
        let text = "The 3rd line is important for territory.";
        assert_eq!(sanitize_coordinates_in_text(text), text);
    }

    #[test]
    fn parse_with_hallucinated_coordinate() {
        let raw = r#"<classification>{"error_class": "Shape"}</classification>
<coaching>Playing at Z99 creates bad shape. Consider D4 instead.</coaching>"#;
        let output = parse_llm_output(raw);
        assert_eq!(output.error_class.as_deref(), Some("Shape"));
        assert!(output.coaching_text.contains("D4"));
        assert!(!output.coaching_text.contains("Z99"));
        assert!(output.coaching_text.contains("[...]"));
    }
}
