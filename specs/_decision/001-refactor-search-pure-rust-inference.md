# Decisions: refactor-search-pure-rust-inference

## ADR: Adopt pure-Rust inference (Option B) to remove the ONNX Runtime dependency

**ID:** pure-rust-inference
**Plan:** refactor-search-pure-rust-inference
**Status:** Accepted

### Context

`speq search` relied on `fastembed`/`ort`, which binds to the ONNX Runtime C++ library. On Intel macOS (`x86_64-apple-darwin`), no prebuilt `ort` binary exists, so the library is loaded via `dlopen` at runtime. When absent, `ort` panics inside `TextEmbedding::try_new` rather than returning an error, causing an unrecoverable process abort (exit 101). A narrow fix wrapping the call in `catch_unwind` (branch `fix-intel-mac-onnxruntime-panic`) makes the failure survivable but leaves Intel-Mac users unable to search without `brew install onnxruntime`, keeps the C++ dependency, and keeps the platform-split `Cargo.toml`. The user asked to remove `ort` entirely.

### Decision

Replace `fastembed`/`ort` with a pure-Rust inference path. Remove the ONNX Runtime C/C++ library from the dependency tree entirely. Deliver one uniform inference code path for all platforms with no `cfg`-gated dependency split.

### Options Considered

- **Option A (narrow panic fix):** Keep `fastembed`/`ort`; wrap `TextEmbedding::try_new` in `catch_unwind`. A safety net, not a cure — leaves Intel-Mac degraded and the C++ dependency intact.
- **Option B (pure-Rust inference) — chosen:** Replace the ONNX-based stack with a pure-Rust alternative. Eliminates the root cause, unifies platforms, shrinks binary.
- **Option C (lexical BM25/TF-IDF):** Drop the neural model entirely. Rejected — regresses search from semantic to lexical matching, changing the product.

### Consequences

No ONNX Runtime library is required at build time or runtime. `cargo install speq-skill` requires no native toolchain or system library. The Intel-Mac `dlopen` panic is eliminated structurally. The `Cargo.toml` platform-split `[target.'cfg(...)'.dependencies]` blocks are removed.

## ADR: Use candle native BERT (Option B2) as the pure-Rust inference backend

**ID:** candle-native-bert
**Plan:** refactor-search-pure-rust-inference
**Status:** Superseded by tract-onnx-inference

### Context

Having decided on a pure-Rust inference path (`pure-rust-inference`), two sub-variants were evaluated for the embedding model `Snowflake/snowflake-arctic-embed-xs`, architecturally an `all-MiniLM-L6-v2` BERT encoder: a pure-Rust ONNX runtime (`tract`, keeping the existing `.onnx` file) and a native Rust BERT encoder (`candle`, storing the model as `safetensors`). Both were viable ways to run this standard 6-layer encoder without ONNX Runtime.

### Decision

Implement inference with `candle-core` + `candle-nn` + `candle-transformers` (`models::bert::BertModel`) plus the `tokenizers` crate, storing the model as `model.safetensors` + `tokenizer.json` + `config.json`, rejecting `tract` pure-Rust ONNX (Option B1) for its uneven ONNX operator coverage and higher integration risk.

## ADR: Move model acquisition out of the binary into the installer

**ID:** installer-model-provisioning
**Plan:** refactor-search-pure-rust-inference
**Status:** Accepted

### Context

The previous `fastembed`/`ort` path downloaded the model on first run via `hf-hub`. With the switch to candle (`candle-native-bert`), two alternative model-delivery approaches were considered: keep an in-binary downloader or embed the weights with `include_bytes!`. Both have significant downsides for a CLI tool distributed via `cargo install` and a shell installer.

### Decision

The `speq` binary contains no model-download code. It only reads model files from `$SPEQ_CACHE/speq/models/`. `install.sh` (and the future Homebrew formula) provision `model.onnx` and `tokenizer.json` from HuggingFace at install time (per `tract-onnx-inference`).

### Options Considered

- **In-binary downloader (`hf-hub`):** Re-introduces a network dependency and TLS stack into the binary; first-run search requires internet access.
- **`include_bytes!` embedded weights:** Bloats the binary by ~23 MB and ships the model on every `cargo install`, regardless of whether search is used.
- **Installer provisioning — chosen:** Keeps the binary small, makes first-run search offline once installed, and keeps `cargo install` free of a native build dependency or large download.

### Consequences

The model is Apache 2.0 (verified 2026-05-22). The installer downloads it from HuggingFace, so no release archive redistributes it. `THIRD_PARTY_LICENSES` attributes it in a Downloaded Assets section (`about.hbs`). `cargo deny` does not cover model files.
