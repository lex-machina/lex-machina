# lex-learning

Automated machine learning training library with embedded Python runtime.

Part of **Lex Machina**

## Overview

`lex-learning` is a Rust crate that provides automated ML training capabilities by embedding a Python 3.12 runtime with pre-installed ML libraries. It handles:

- **Preprocessing**: Automatic encoding and scaling of features
- **Algorithm Selection**: Heuristic-based selection of best algorithms for your data
- **Hyperparameter Optimization**: Optuna-based tuning with cross-validation
- **Model Training**: Support for sklearn, XGBoost, LightGBM, and neural networks
- **Explainability**: SHAP-based feature importance and explanation plots
- **Inference**: Single and batch predictions from trained models

## Quick Start (Rust)

```rust
use lex_learning::{Pipeline, PipelineConfig, ProblemType};
use polars::prelude::*;

// Initialize Python runtime (call once at startup)
lex_learning::initialize()?;

// Configure the pipeline
let config = PipelineConfig::builder()
    .problem_type(ProblemType::Classification)
    .target_column("Survived")
    .build()?;

// Build and run the pipeline
let pipeline = Pipeline::builder()
    .config(config)
    .on_progress(|u| println!("{:.0}% - {}", u.progress * 100.0, u.message))
    .build()?;

let result = pipeline.train(&dataframe)?;

// Create model for inference
let model = pipeline.create_trained_model(&result)?;
let prediction = model.predict(&serde_json::json!({"Age": 25, "Sex": "male"}))?;

// Save for later use
model.save("model.pkl")?;
```

## Quick Start (Python)

```python
from lex_learning import Pipeline, PipelineConfig, ProblemType, TrainedModel

config = PipelineConfig.builder() \
    .problem_type(ProblemType.CLASSIFICATION) \
    .target_column("Survived") \
    .build()

result = Pipeline.builder() \
    .config(config) \
    .on_progress(lambda u: print(f"{u.progress:.0%} - {u.message}")) \
    .build() \
    .train(dataframe)

# Save and load models
trained_model = pipeline.create_trained_model(result)
trained_model.save("model.pkl")

model = TrainedModel.load("model.pkl")
prediction = model.predict({"Age": 25, "Sex": "male"})
```

## Building

```bash
# Build the Rust crate
cargo build

# Run tests
cargo test

# Run Python tests
cd python/lex_learning && uv run pytest -v
```

## Documentation

See [AGENTS.md](./AGENTS.md) for comprehensive API documentation, architecture details, and implementation guidelines.

## License

[LICENSE](./LICENSE)
