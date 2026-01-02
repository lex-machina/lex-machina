"""
lex-learning: Automated Machine Learning Pipeline

CLI interface for training models and making predictions.

Usage:
    lex-learning train <dataset.csv> [OPTIONS]
    lex-learning predict <model.pkl> <data.csv> [OPTIONS]
    lex-learning predict-one <model.pkl> '<json>'
    lex-learning info <model.pkl>
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import pandas as pd

from src import (
    ClassificationMetrics,
    Pipeline,
    PipelineConfig,
    ProblemType,
    ProgressUpdate,
    RegressionMetrics,
    TrainedModel,
)
from src.explainability import save_explainability_plots


def cmd_train(args: argparse.Namespace) -> int:
    """Train a model on a dataset."""
    dataset_path = Path(args.input)

    if not dataset_path.exists():
        print(f"Error: Dataset file not found: {dataset_path}")
        return 1

    # Load data
    print(f"Loading data from {dataset_path}...")
    data = pd.read_csv(dataset_path)
    print(f"Loaded {len(data)} rows, {len(data.columns)} columns")

    # Determine problem type
    problem_type = None
    if args.problem_type:
        problem_type = ProblemType(args.problem_type)
    else:
        # Auto-detect based on target column
        target_col = args.target or data.columns[-1]
        if target_col in data.columns:
            unique_values = data[target_col].nunique()
            if unique_values <= 20 or data[target_col].dtype == "object":
                problem_type = ProblemType.CLASSIFICATION
            else:
                problem_type = ProblemType.REGRESSION
            print(f"Auto-detected problem type: {problem_type.value}")
        else:
            print(f"Error: Target column '{target_col}' not found")
            return 1

    # Progress callback
    def on_progress(update: ProgressUpdate) -> None:
        pct = f"{update.progress * 100:5.1f}%"
        if args.verbose:
            print(f"[{pct}] {update.stage.value}: {update.message}")
        else:
            # Simple progress bar
            bar_len = 30
            filled = int(bar_len * update.progress)
            bar = "=" * filled + "-" * (bar_len - filled)
            print(f"\r[{bar}] {pct} {update.message[:40]:<40}", end="", flush=True)

    # Configure pipeline
    config = (
        PipelineConfig.builder()
        .problem_type(problem_type)
        .target_column(args.target)
        .top_k_algorithms(args.top_k)
        .optimize_hyperparams(not args.no_optimize)
        .enable_neural_networks(not args.no_neural)
        .enable_explainability(not args.no_explain)
        .n_trials(args.trials)
        .random_seed(args.seed)
        .build()
    )

    if args.algorithm:
        config = (
            PipelineConfig.builder()
            .problem_type(problem_type)
            .target_column(args.target)
            .algorithm(args.algorithm)
            .optimize_hyperparams(not args.no_optimize)
            .enable_neural_networks(not args.no_neural)
            .enable_explainability(not args.no_explain)
            .n_trials(args.trials)
            .random_seed(args.seed)
            .build()
        )

    # Train
    print(f"\nTraining with top {args.top_k} algorithms...")
    print("-" * 60)

    pipeline = Pipeline.builder().config(config).on_progress(on_progress).build()

    try:
        result = pipeline.train(data)
    except KeyboardInterrupt:
        print("\n\nTraining cancelled by user")
        return 1

    print("\n" + "-" * 60)
    print(f"\nBest model: {result.best_model_name}")
    print(f"Test score: {result.metrics.test_score:.4f}")

    if isinstance(result.metrics, ClassificationMetrics):
        print(f"Accuracy:   {result.metrics.accuracy:.4f}")
        print(f"Precision:  {result.metrics.precision:.4f}")
        print(f"Recall:     {result.metrics.recall:.4f}")
        print(f"F1 Score:   {result.metrics.f1_score:.4f}")
    elif isinstance(result.metrics, RegressionMetrics):
        print(f"R2 Score:   {result.metrics.r2:.4f}")
        print(f"RMSE:       {result.metrics.rmse:.4f}")
        print(f"MAE:        {result.metrics.mae:.4f}")

    print(f"\nModel comparison ({len(result.model_comparison)} models trained):")
    for m in result.model_comparison[:5]:
        risk = f" [overfit: {m.overfitting_risk}]" if m.overfitting_risk != "low" else ""
        print(f"  - {m.name}: test={m.test_score:.4f}, cv={m.cv_score:.4f}{risk}")

    # Prepare output directory
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Save model
    model_path = output_dir / "model.pkl"
    trained_model = pipeline.create_trained_model(result)
    trained_model.save(model_path)
    print(f"\nOutputs saved to: {output_dir}/")
    print(f"  - {model_path.name}")

    # Save explainability plots to disk
    if not args.no_explain and result.explainability.method == "shap":
        saved_plots = save_explainability_plots(result.explainability, output_dir)
        for p in saved_plots:
            print(f"  - {p.name}")

    return 0


def cmd_predict(args: argparse.Namespace) -> int:
    """Make batch predictions."""
    model_path = Path(args.model)
    data_path = Path(args.input)

    if not model_path.exists():
        print(f"Error: Model file not found: {model_path}")
        return 1

    if not data_path.exists():
        print(f"Error: Data file not found: {data_path}")
        return 1

    # Load model
    print(f"Loading model from {model_path}...")
    model = TrainedModel.load(model_path)

    # Make predictions
    print(f"Making predictions on {data_path}...")
    predictions = model.predict_batch(data_path)

    # Save predictions
    output_path = Path(args.output)
    predictions.to_csv(output_path, index=False)
    print(f"Predictions saved to: {output_path}")

    return 0


def cmd_predict_one(args: argparse.Namespace) -> int:
    """Make a single prediction."""
    model_path = Path(args.model)

    if not model_path.exists():
        print(f"Error: Model file not found: {model_path}")
        return 1

    # Parse JSON data
    try:
        instance = json.loads(args.json_data)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON: {e}")
        return 1

    # Load model
    model = TrainedModel.load(model_path)

    # Make prediction
    result = model.predict(instance)

    print(json.dumps(result, indent=2))
    return 0


def cmd_info(args: argparse.Namespace) -> int:
    """Show model information."""
    model_path = Path(args.model)

    if not model_path.exists():
        print(f"Error: Model file not found: {model_path}")
        return 1

    # Load model
    model = TrainedModel.load(model_path)
    info = model.get_info()

    print(f"Model: {model_path}")
    print(f"  Version:      {info['version']}")
    print(f"  Problem Type: {info['problem_type']}")
    print(f"  Algorithm:    {info['best_model_name']}")
    print(f"  Target:       {info['target_column']}")
    print(f"  Trained:      {info['trained_at']}")
    print(f"  Training Time: {info['training_time_seconds']:.1f}s")

    print("\nFeatures:")
    for f in info["feature_names"]:
        print(f"  - {f}")

    if info.get("class_labels"):
        print("\nClass Labels:")
        for label in info["class_labels"]:
            print(f"  - {label}")

    print("\nMetrics:")
    metrics = info["metrics"]
    if metrics.get("accuracy") is not None:
        print(f"  Accuracy:  {metrics['accuracy']:.4f}")
        print(f"  Precision: {metrics['precision']:.4f}")
        print(f"  Recall:    {metrics['recall']:.4f}")
        print(f"  F1 Score:  {metrics['f1_score']:.4f}")
    if metrics.get("r2") is not None:
        print(f"  R2:   {metrics['r2']:.4f}")
        print(f"  RMSE: {metrics['rmse']:.4f}")
        print(f"  MAE:  {metrics['mae']:.4f}")
    print(f"  Test Score: {metrics['test_score']:.4f}")
    print(f"  CV Score:   {metrics['cv_score']:.4f}")

    if info.get("feature_importance"):
        print("\nFeature Importance (top 10):")
        for name, importance in info["feature_importance"][:10]:
            bar = "=" * int(importance * 50)
            print(f"  {name:20s} {importance:.3f} {bar}")

    return 0


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="lex-learning: Automated Machine Learning Pipeline",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    subparsers = parser.add_subparsers(dest="command", help="Command to run")

    # Train command
    train_parser = subparsers.add_parser("train", help="Train a model")
    train_parser.add_argument("input", help="Input CSV file")
    train_parser.add_argument("-t", "--target", help="Target column name")
    train_parser.add_argument(
        "-p",
        "--problem-type",
        choices=["classification", "regression"],
        help="Problem type (auto-detected if not specified)",
    )
    train_parser.add_argument("-a", "--algorithm", help="Specific algorithm to use")
    train_parser.add_argument(
        "-o", "--output", default="output", help="Output directory for model and plots"
    )
    train_parser.add_argument("-k", "--top-k", type=int, default=3, help="Top K algorithms to try")
    train_parser.add_argument("--trials", type=int, default=30, help="Optuna trials per model")
    train_parser.add_argument("--seed", type=int, default=42, help="Random seed")
    train_parser.add_argument(
        "--no-optimize", action="store_true", help="Disable hyperparameter optimization"
    )
    train_parser.add_argument("--no-neural", action="store_true", help="Disable neural networks")
    train_parser.add_argument("--no-explain", action="store_true", help="Disable explainability")
    train_parser.add_argument("-v", "--verbose", action="store_true", help="Verbose output")

    # Predict command
    predict_parser = subparsers.add_parser("predict", help="Make batch predictions")
    predict_parser.add_argument("model", help="Model file (.pkl)")
    predict_parser.add_argument("input", help="Input CSV file")
    predict_parser.add_argument("-o", "--output", default="predictions.csv", help="Output CSV path")

    # Predict-one command
    predict_one_parser = subparsers.add_parser("predict-one", help="Make a single prediction")
    predict_one_parser.add_argument("model", help="Model file (.pkl)")
    predict_one_parser.add_argument(
        "json_data", help='JSON data (e.g., \'{"Age": 25, "Sex": "male"}\')'
    )

    # Info command
    info_parser = subparsers.add_parser("info", help="Show model information")
    info_parser.add_argument("model", help="Model file (.pkl)")

    args = parser.parse_args()

    if args.command is None:
        parser.print_help()
        return 1

    if args.command == "train":
        return cmd_train(args)
    elif args.command == "predict":
        return cmd_predict(args)
    elif args.command == "predict-one":
        return cmd_predict_one(args)
    elif args.command == "info":
        return cmd_info(args)

    return 0


if __name__ == "__main__":
    sys.exit(main())
