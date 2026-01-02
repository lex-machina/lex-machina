# lex-learning (Python Library)

Python machine learning library for automated training with explainability.

## Installation

```bash
# Using uv (recommended)
uv pip install -e .

# Using pip
pip install -e .
```

## Usage

```python
from lex_learning import Pipeline, PipelineConfig, ProblemType, TrainedModel
import pandas as pd

# Load data
df = pd.read_csv("data.csv")

# Configure pipeline
config = PipelineConfig.builder() \
    .problem_type(ProblemType.CLASSIFICATION) \
    .target_column("target") \
    .top_k_algorithms(3) \
    .optimize_hyperparams(True) \
    .enable_explainability(True) \
    .build()

# Train
pipeline = Pipeline.builder() \
    .config(config) \
    .on_progress(lambda u: print(f"{u.progress:.0%} - {u.message}")) \
    .build()

result = pipeline.train(df)

# Save model
trained_model = pipeline.create_trained_model(result)
trained_model.save("model.pkl")

# Load and predict
model = TrainedModel.load("model.pkl")
prediction = model.predict({"feature1": 1.0, "feature2": "value"})
print(prediction)  # {"prediction": "class_a", "probability": 0.85}
```

## Development

```bash
# Run tests
uv run pytest -v

# Run specific test
uv run pytest -v tests/test_pipeline.py::TestPipelineBasic
```

## Documentation

See [AGENTS.md](../../AGENTS.md) for complete documentation.
