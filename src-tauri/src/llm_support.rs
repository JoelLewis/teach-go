//! App-side LLM policy: where the model lives on disk, migration of
//! pre-sensei-kit installs, and GoSensei's generation settings.
//!
//! The runtime itself (download, load, generation) is the shared
//! `sensei-llm` crate; everything in this module is GoSensei-specific
//! configuration around it.

use std::path::{Path, PathBuf};

use sensei_llm::{GEMMA4_E2B_Q4, GenerateOptions, ModelSpec, ModelStore, SamplerConfig};
use tracing::{info, warn};

/// The sampler values GoSensei has always used. `sensei-llm`'s defaults are
/// the ChessMentor values (temperature 0.3, top-k 50, seed 42), so these must
/// be passed explicitly on every generation call.
pub const GO_SAMPLER: SamplerConfig = SamplerConfig {
    temperature: 0.6,
    top_k: None,
    min_p: Some(0.05),
    top_p: 0.9,
    repeat_penalty: 1.15,
    repeat_penalty_last_n: 64,
    seed: 1234,
};

/// Generation options for grammar-constrained coaching output.
///
/// - `n_ctx: 512` keeps the pre-sensei-kit context size (the shared default
///   is 1024), preserving the prompt-length guard and memory profile.
/// - `filter_channels: false` keeps streaming byte-identical to the old
///   crate: the frontend accumulates raw pieces, and the coaching grammar
///   already forbids the `<|channel>` markers the filter strips.
pub fn coaching_generate_options(max_tokens: u32, grammar: String) -> GenerateOptions {
    GenerateOptions {
        max_tokens,
        n_ctx: 512,
        sampler: GO_SAMPLER,
        grammar: Some(grammar),
        grammar_root: "root".to_string(),
        filter_channels: false,
    }
}

/// Store for the coaching model, rooted at `{app_data}/models`.
///
/// Migrates any pre-sensei-kit install into sensei-llm's
/// `{models}/{repo_id}/{filename}` layout first, so existing users do not
/// re-download the ~2.9 GiB GGUF.
pub fn model_store(app_data_dir: &Path) -> ModelStore {
    let models_dir = app_data_dir.join("models");
    migrate_legacy_model_layout(app_data_dir, &models_dir, &GEMMA4_E2B_Q4);
    ModelStore::new(models_dir)
}

/// One-time move of a legacy flat model file into the
/// `{models_dir}/{repo_id}/{filename}` layout used by [`ModelStore`].
///
/// The old `gosensei-llm` crate stored the GGUF flat at `{dir}/{filename}`
/// and used `{dir}` itself as the hf-hub cache, so the flat path is usually
/// a symlink into a `models--…` snapshot tree. Two directories were in use:
/// `{app_data}/llm` (startup download manager) and `{app_data}/models`
/// (the `init_llm_model` command).
fn migrate_legacy_model_layout(app_data_dir: &Path, models_dir: &Path, spec: &ModelSpec) {
    let target = models_dir.join(spec.repo_id).join(spec.filename);
    if target.exists() {
        return;
    }

    for legacy_dir in [app_data_dir.join("llm"), models_dir.to_path_buf()] {
        if move_legacy_file(&legacy_dir.join(spec.filename), &target, spec) {
            remove_legacy_hf_cache(&legacy_dir, spec);
            return;
        }
    }
}

/// Move one legacy flat model file (or the file behind its symlink) to
/// `target`. Returns `true` when a complete model file was migrated.
fn move_legacy_file(flat: &Path, target: &Path, spec: &ModelSpec) -> bool {
    let Ok(meta) = std::fs::symlink_metadata(flat) else {
        return false;
    };

    let source: PathBuf = if meta.file_type().is_symlink() {
        match flat.canonicalize() {
            Ok(resolved) => resolved,
            Err(e) => {
                warn!(
                    "Removing dangling legacy model symlink {}: {e}",
                    flat.display()
                );
                let _ = std::fs::remove_file(flat);
                return false;
            }
        }
    } else {
        flat.to_path_buf()
    };

    // A file well below the published size is a truncated download the old
    // code could never repair; clear it so the store re-downloads cleanly.
    let complete = std::fs::metadata(&source).is_ok_and(|m| m.len() >= spec.size_bytes * 9 / 10);
    if !complete {
        warn!("Removing truncated legacy model file {}", source.display());
        let _ = std::fs::remove_file(&source);
        let _ = std::fs::remove_file(flat);
        return false;
    }

    let Some(parent) = target.parent() else {
        return false;
    };
    if let Err(e) = std::fs::create_dir_all(parent) {
        warn!("Model migration: create {}: {e}", parent.display());
        return false;
    }
    match std::fs::rename(&source, target) {
        Ok(()) => {
            if meta.file_type().is_symlink() {
                let _ = std::fs::remove_file(flat);
            }
            info!(
                "Migrated model {} -> {}",
                source.display(),
                target.display()
            );
            true
        }
        Err(e) => {
            warn!(
                "Model migration: move {} -> {}: {e}",
                source.display(),
                target.display()
            );
            false
        }
    }
}

/// Best-effort removal of the legacy hf-hub cache tree. After the blob has
/// been moved out it holds only tiny metadata files and dangling snapshot
/// symlinks.
fn remove_legacy_hf_cache(legacy_dir: &Path, spec: &ModelSpec) {
    let cache = legacy_dir.join(format!("models--{}", spec.repo_id.replace('/', "--")));
    if cache.exists() {
        let _ = std::fs::remove_dir_all(&cache);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SPEC: ModelSpec = ModelSpec {
        repo_id: "test/repo",
        filename: "model.gguf",
        size_bytes: 1000,
        sha256: None,
    };

    fn target_path(app_data: &Path) -> PathBuf {
        app_data
            .join("models")
            .join(TEST_SPEC.repo_id)
            .join(TEST_SPEC.filename)
    }

    fn migrate(app_data: &Path) {
        migrate_legacy_model_layout(app_data, &app_data.join("models"), &TEST_SPEC);
    }

    #[test]
    fn migrates_flat_file_from_legacy_llm_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let legacy = tmp.path().join("llm");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join(TEST_SPEC.filename), vec![0u8; 1000]).unwrap();

        migrate(tmp.path());

        let target = target_path(tmp.path());
        assert!(target.exists(), "model should be moved to the new layout");
        assert_eq!(std::fs::metadata(&target).unwrap().len(), 1000);
        assert!(
            !legacy.join(TEST_SPEC.filename).exists(),
            "flat file should be gone"
        );
    }

    #[test]
    fn migrates_flat_file_from_legacy_models_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let legacy = tmp.path().join("models");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join(TEST_SPEC.filename), vec![0u8; 1000]).unwrap();

        migrate(tmp.path());

        assert!(target_path(tmp.path()).exists());
        assert!(!legacy.join(TEST_SPEC.filename).exists());
    }

    #[test]
    fn migrates_symlinked_file_and_cleans_hf_cache() {
        let tmp = tempfile::tempdir().unwrap();
        let legacy = tmp.path().join("llm");
        // Mimic the old hf-hub cache layout: blob + snapshot + flat symlink.
        let cache = legacy.join("models--test--repo");
        let snapshot = cache.join("snapshots").join("abc123");
        std::fs::create_dir_all(&snapshot).unwrap();
        let blob = snapshot.join(TEST_SPEC.filename);
        std::fs::write(&blob, vec![0u8; 1000]).unwrap();
        let flat = legacy.join(TEST_SPEC.filename);
        std::os::unix::fs::symlink(&blob, &flat).unwrap();

        migrate(tmp.path());

        let target = target_path(tmp.path());
        assert!(target.exists());
        assert_eq!(std::fs::metadata(&target).unwrap().len(), 1000);
        assert!(!flat.exists(), "flat symlink should be removed");
        assert!(!cache.exists(), "legacy hf cache tree should be removed");
    }

    #[test]
    fn removes_truncated_legacy_file_without_migrating() {
        let tmp = tempfile::tempdir().unwrap();
        let legacy = tmp.path().join("llm");
        std::fs::create_dir_all(&legacy).unwrap();
        // Below 90% of size_bytes.
        std::fs::write(legacy.join(TEST_SPEC.filename), vec![0u8; 100]).unwrap();

        migrate(tmp.path());

        assert!(!target_path(tmp.path()).exists());
        assert!(
            !legacy.join(TEST_SPEC.filename).exists(),
            "truncated file should be cleared so the store re-downloads"
        );
    }

    #[test]
    fn removes_dangling_legacy_symlink() {
        let tmp = tempfile::tempdir().unwrap();
        let legacy = tmp.path().join("llm");
        std::fs::create_dir_all(&legacy).unwrap();
        let flat = legacy.join(TEST_SPEC.filename);
        std::os::unix::fs::symlink(legacy.join("missing-blob"), &flat).unwrap();

        migrate(tmp.path());

        assert!(!target_path(tmp.path()).exists());
        assert!(
            std::fs::symlink_metadata(&flat).is_err(),
            "dangling symlink should be removed"
        );
    }

    #[test]
    fn noop_when_new_layout_already_populated() {
        let tmp = tempfile::tempdir().unwrap();
        let target = target_path(tmp.path());
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, vec![1u8; 1000]).unwrap();

        let legacy = tmp.path().join("llm");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join(TEST_SPEC.filename), vec![0u8; 1000]).unwrap();

        migrate(tmp.path());

        assert!(
            legacy.join(TEST_SPEC.filename).exists(),
            "legacy file must be left alone once the new layout exists"
        );
        assert_eq!(std::fs::read(&target).unwrap(), vec![1u8; 1000]);
    }

    #[test]
    fn noop_without_any_legacy_install() {
        let tmp = tempfile::tempdir().unwrap();
        migrate(tmp.path());
        assert!(!target_path(tmp.path()).exists());
    }

    #[test]
    fn model_store_resolves_to_new_layout() {
        let tmp = tempfile::tempdir().unwrap();
        let store = model_store(tmp.path());
        assert_eq!(
            store.model_path(&GEMMA4_E2B_Q4),
            tmp.path()
                .join("models")
                .join(GEMMA4_E2B_Q4.repo_id)
                .join(GEMMA4_E2B_Q4.filename)
        );
    }

    #[test]
    fn go_sampler_matches_historical_values() {
        assert_eq!(GO_SAMPLER.temperature, 0.6);
        assert_eq!(GO_SAMPLER.top_k, None);
        assert_eq!(GO_SAMPLER.min_p, Some(0.05));
        assert_eq!(GO_SAMPLER.top_p, 0.9);
        assert_eq!(GO_SAMPLER.repeat_penalty, 1.15);
        assert_eq!(GO_SAMPLER.repeat_penalty_last_n, 64);
        assert_eq!(GO_SAMPLER.seed, 1234);
    }

    #[test]
    fn coaching_options_preserve_pre_kit_behavior() {
        let opts = coaching_generate_options(150, "root ::= \"x\"".to_string());
        assert_eq!(opts.max_tokens, 150);
        assert_eq!(opts.n_ctx, 512);
        assert_eq!(opts.sampler, GO_SAMPLER);
        assert_eq!(opts.grammar_root, "root");
        assert!(!opts.filter_channels);
    }
}
