# Feature: Embedding Model Provisioning

Ensures the semantic-search embedding model is placed into the speq model cache during installation, so the first `speq search` invocation works offline with no network access.

## Background

* The model is distributed as two files: model weights (`model.onnx`) and a tokenizer definition (`tokenizer.json`); the ONNX graph embeds the model configuration, so no separate config file is provisioned
* Provisioning is idempotent: if the model files already exist in the cache, the installer SHALL NOT re-download them

## Scenarios

### Scenario: Installer provisions model on a clean machine

* *GIVEN* the speq model cache directory contains no model files
* *AND* the install script is run for a published release version
* *WHEN* the install script reaches the model-provisioning step
* *THEN* the script SHALL download the model weights and tokenizer definition
* *AND* the script SHALL place both files (`model.onnx` and `tokenizer.json`) under the speq model cache directory
* *AND* a subsequent `speq search query` SHALL succeed without network access

### Scenario: Installer skips provisioning when the model is already cached

* *GIVEN* the speq model cache directory already contains all required model files (`model.onnx` and `tokenizer.json`)
* *WHEN* the install script reaches the model-provisioning step
* *THEN* the script SHALL detect the existing model files
* *AND* the script SHALL NOT re-download the model files
* *AND* the script SHALL report that the model is already provisioned

### Scenario: Model download fails during installation

* *GIVEN* the speq model cache directory contains no model files
* *AND* the model release assets cannot be downloaded
* *WHEN* the install script reaches the model-provisioning step
* *THEN* the script SHALL report a clear error identifying the failed download
* *AND* the script SHALL instruct the user how to provision the model manually
* *AND* the script MUST NOT leave a partially written model file in the cache directory

### Scenario: Model provisioning honors a custom cache directory

* *GIVEN* `$SPEQ_CACHE_DIR` is set to a custom path
* *WHEN* the install script provisions the embedding model
* *THEN* the script SHALL place the model files (`model.onnx` and `tokenizer.json`) under `$SPEQ_CACHE_DIR/models/`
* *AND* a subsequent `speq search query` run with the same `$SPEQ_CACHE_DIR` SHALL load the model from that path

### Scenario: Model provisioning uses the macOS cache directory

* *GIVEN* `$SPEQ_CACHE_DIR` is unset
* *AND* the install script runs on macOS
* *WHEN* the install script provisions the embedding model
* *THEN* the script SHALL place the model files (`model.onnx` and `tokenizer.json`) under `$HOME/Library/Caches/speq/models/`
* *AND* a subsequent `speq search query` SHALL load the model from that same path

### Scenario: Model provisioning uses the Linux XDG cache directory

* *GIVEN* `$SPEQ_CACHE_DIR` is unset
* *AND* the install script runs on Linux
* *WHEN* the install script provisions the embedding model
* *THEN* the script SHALL place the model files (`model.onnx` and `tokenizer.json`) under `${XDG_CACHE_HOME:-$HOME/.cache}/speq/models/`
* *AND* a subsequent `speq search query` SHALL load the model from that same path
