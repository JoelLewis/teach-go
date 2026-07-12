//! Real-model integration test for the coaching pipeline on sensei-llm.
#![cfg(feature = "llm")]

use std::path::PathBuf;

use gosensei_app::llm_support::coaching_generate_options;
use sensei_llm::{GEMMA4_E2B_Q4, ModelManager, ModelStore, format_chat};

/// Downloads Gemma 4 E2B Q4_K_M (~2.9 GiB) on first run, loads it, and
/// verifies grammar-constrained generation end to end.
///
/// Run explicitly with:
/// `cargo test -p gosensei-app --test real_model -- --ignored`
/// Set `GOSENSEI_LLM_MODEL_DIR` to reuse an existing model directory.
#[test]
#[ignore = "downloads and loads the ~2.9 GiB Gemma 4 E2B model"]
fn real_model_constrained_generation() {
    let models_dir = std::env::var_os("GOSENSEI_LLM_MODEL_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("gosensei-llm-real-model"));

    let store = ModelStore::new(models_dir);
    let model_path = store.download(&GEMMA4_E2B_Q4, |_, _| {}).expect("download");
    let manager = ModelManager::load(&model_path).expect("model load");

    let prompt = format_chat(
        gosensei_llm::prompt::SYSTEM_PROMPT,
        "Player rank: 12k. Move 42: played D4 (score loss: 3.5pt, severity: Mistake). \
         Best/simplest move was Q16.",
    );
    let options = coaching_generate_options(200, gosensei_llm::grammar::coaching_grammar());
    let mut streamed = String::new();
    let output = manager
        .generate_streaming(&prompt, &options, |piece| streamed.push_str(piece))
        .expect("constrained generation");

    assert!(
        output.starts_with("<classification>{\"error_class\": \""),
        "grammar should force the tagged prefix, got: {output}"
    );
    assert_eq!(
        streamed.trim(),
        output,
        "raw streamed pieces should reassemble into the final output"
    );
    let parsed = gosensei_llm::parse::parse_llm_output(&output);
    assert!(
        parsed.error_class.is_some(),
        "classification should be a known class, got: {output}"
    );
    assert!(!parsed.coaching_text.is_empty());
}
