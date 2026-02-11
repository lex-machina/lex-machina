# Lex Machina

A local-first, privacy-focused machine learning platform that democratizes data analytics and predictive modeling for SMEs and non-technical users.

![lex-machina](https://i.imgur.com/z3oSRMt.png)
---

## Executive Summary

Lex Machina is a desktop application that provides a complete no-code workflow for data preparation, analytics, visualization, and machine learning. Built on a modern Rust-Python architecture with embedded runtimes, it eliminates the barriers of cost, complexity, and technical expertise that prevent organizations from leveraging predictive analytics.

**Key Differentiators:**
- 🔒 **Local-First & Private**: All processing happens on your machine—no cloud dependencies
- 💰 **Zero Subscription Costs**: One-time installation, no recurring fees
- 🎯 **No-Code Workflow**: Complete ML pipeline accessible to non-programmers
- 📊 **Explainability-First**: SHAP-based insights integrated as standard output
- ⚡ **High Performance**: Rust core with Apache Arrow data bridge for optimal speed

---

## Research Motivation

### Data Availability vs. Analytics Access

While data availability continues to increase across industries, access to advanced analytics remains limited by prohibitive costs and technical complexity. Most AutoML systems are either cloud-dependent subscription services or require significant programming expertise.

### The Gap We Address

**Economic Barrier**: Subscription-heavy systems exclude small and medium enterprises (SMEs) from competitive advantages provided by predictive analytics.

**Workflow Fragmentation**: Data preparation, machine learning, and explainability are typically split across multiple disconnected tools, creating friction and inefficiency.

**Technical Barrier**: Most platforms assume programming expertise in Python, R, or SQL, limiting adoption to data science teams.

**Interpretability Gap**: Model explainability is rarely built-in by default, leaving users unable to understand or trust predictions.

---

## Key Features

### 🗂️ Data Viewer
Import and explore CSV datasets of any size with a high-performance grid interface.

![Data Viewer](https://i.imgur.com/1xyV11e.png)

![Data Viewer](https://i.imgur.com/gvsdQRX.png)

**Capabilities:**
- Virtual scrolling for large datasets (millions of rows)
- Resizable columns with automatic type detection
- Export processed data to CSV
- Data exploration

---

### ⚙️ Data Preprocessing
AI-guided data cleaning and transformation without writing code.

![Data Preprocessing](https://i.imgur.com/FHosv4M.png)

**Features:**
- Missing value imputation (KNN, statistical methods)
- Outlier detection and handling
- Type correction and data validation
- Smart preprocessing recommendations
- **100% Data Quality Score** tracking

![Preprocessing Results](https://i.imgur.com/GSTc3G0.png)

---

### 📊 Statistical Analysis
Comprehensive exploratory data analysis and profiling.

![Statistical Analysis](https://i.imgur.com/uLuwnc4.png)

**Analysis Tools:**
- Pearson and Spearman correlation matrices
- Distribution analysis with customizable bins
- Missing value and outlier reports
- Feature relationship visualization

![Correlation Analysis](https://i.imgur.com/EE6A5Aj.png)

---

### 📈 Visualizations
Auto-generated dashboards tailored to your data types.

![Visualizations](https://i.imgur.com/mYEFKjM.png)

**Visualization Features:**
- Smart chart selection by data type (histograms, line charts, distributions)
- Per-column chart style controls
- Interactive exploration of distributions and trends
- Integration with preprocessing outputs

![Data Distributions](https://i.imgur.com/5f4Ok1n.png)

---

### 🤖 Machine Learning
No-code AutoML with explainability built-in.

![ML Training Interface](https://i.imgur.com/0gJZoy5.png)

**ML Capabilities:**
- Auto-select algorithm based on problem type
- Hyperparameter optimization with Optuna
- Cross-validation with configurable folds
- **Feature Importance Analysis** via SHAP values
- Support for classification, regression, and more

**Key Metrics:**
- Feature importance scores for model transparency
- Training metrics (accuracy, precision, recall, F1, AUC-ROC)
- Model comparison across algorithms

---

### 🎯 Prediction Interface
Deploy trained models for single or batch predictions.

![Prediction Interface](https://i.imgur.com/nygMBrz.png)

**Prediction Options:**
- **Single Prediction**: Form-based input for real-time inference
- **Batch Prediction**: Upload CSV for bulk predictions
- Keep models in memory or load saved artifacts
- Export predictions with confidence scores

---

## System Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────┐
│                    UI Layer (Next.js)                   │
│              Presentation Only - JSON Rendering         │
└──────────────────────┬──────────────────────────────────┘
                       │ IPC (Tauri)
┌──────────────────────▼───────────────────────────────────┐
│                   Rust Core (Tauri 2.9)                  │
│         Business Logic, State Management, Security       │
├──────────────────────────────────────────────────────────┤
│  ┌──────────────────┐         ┌────────────────────┐     │
│  │  Lex Processing  │         │   Lex Learning     │     │
│  │  (Polars Engine) │◄────────┤  (AutoML Engine)   │     │
│  └──────────────────┘  Arrow  └────────────────────┘     │
│                         Bridge          │                │
└─────────────────────────────────────────┼────────────────┘
                                          │
                              ┌───────────▼──────────────┐
                              │  Embedded Python 3.12    │
                              │  (Bundled Runtime)       │
                              │  • scikit-learn          │
                              │  • XGBoost, LightGBM     │
                              │  • TensorFlow            │
                              │  • SHAP, Optuna          │
                              └──────────────────────────┘
```

### Architecture Principles

**Separation of Concerns**
- UI renders JSON only—no business logic in presentation layer
- Rust owns all application logic and state
- Python runtime isolated for ML execution only

**Local-First & Dependency-Free**
- Bundled Python runtime (no user installation required)
- All ML libraries embedded
- Zero external dependencies

**Performance Optimization**
- Apache Arrow for zero-copy data transfer between Rust and Python
- Polars for high-speed data processing
- Async operations throughout

---

## Technical Stack

### Application Layer
| Component | Technology | Version |
|-----------|-----------|---------|
| Core Runtime | Rust | 2024 Edition |
| Desktop Framework | Tauri | 2.9+ |
| Embedded ML Runtime | Python | 3.12 (Bundled) |

### Presentation Layer
| Component | Technology | Version |
|-----------|-----------|---------|
| Framework | Next.js | 15 |
| UI Library | React | 19 |
| Language | TypeScript | Latest |
| Styling | Tailwind CSS | 4 |

### Data Processing Engine (Lex Processing)
- **Polars**: High-performance DataFrame operations
- **Strict Data Contracts**: Clean, validated data for AutoML
- **No-Code Configuration**: All transformations via UI controls

### AutoML Engine (Lex Learning)
| Library | Purpose |
|---------|---------|
| scikit-learn | Classical ML algorithms |
| XGBoost | Gradient boosting |
| LightGBM | Fast gradient boosting |
| TensorFlow | Deep learning |
| Optuna | Hyperparameter optimization |
| SHAP | Model explainability |

### Data Bridge
**Apache Arrow**: Zero-copy memory transfer between Rust (Polars) and Python (pandas), enabling fast, memory-efficient data interchange for training and inference.

---

## Workflows

### 1. Data Preparation Workflow (No-Code)

```
Import Dataset → Profile & Validate → Clean & Transform → Export
```

**Steps:**
1. Load dataset from CSV
2. Automatic profiling: types, missing values, distributions
3. No-code cleaning: handle missing values, outliers, type corrections
4. Export clean DataFrame for ML

**Result**: Strict, clean data contract ready for AutoML.

---

### 2. Model Training Workflow (No-Code)

```
Configure Problem → Auto-Select Algorithms → Tune & Train → Explain
```

**Steps:**
1. No-code problem setup (select target variable and task type)
2. Auto-select candidate algorithms based on task
3. Hyperparameter tuning with Optuna
4. Cross-validation for robust evaluation
5. **Explainability generated automatically** via SHAP

**Result**: Trained model with performance metrics and feature importance.

---

### 3. Inference Workflow

```
Load Model → Input Data → Predict → Export Results
```

**Steps:**
1. Keep trained model in memory or load saved model artifact
2. Single prediction via form or batch prediction via CSV upload
3. Run predictions with confidence scores
4. Export predictions or model artifacts

---

### 4. Analysis & Visualization Workflow

```
Import Data → EDA & Profiling → Visualize → Insights
```

**Steps:**
1. Exploratory analysis: summary stats, missing values, distributions
2. Correlation analysis (Pearson & Spearman)
3. Auto-generated visualizations tailored to data types
4. Export insights for reporting

**Result**: Clear, guided visuals for non-technical stakeholders.

---

## Project Structure

Lex Machina is composed of three standalone sub-projects:

### 1. **Lex Machina** (Main Application)
Desktop application orchestrating the complete no-code workflow.

**Responsibilities:**
- User interface and interaction
- Workflow orchestration
- State management
- File I/O and configuration

---

### 2. **Lex Processing** (Data Engine)
No-code data cleaning, profiling, and transformation engine.

**Capabilities:**
- Data validation and type inference
- Missing value imputation
- Outlier detection and handling
- Data quality scoring
- Export to multiple formats

**Technology**: Polars-based pipeline for maximum performance.

---

### 3. **Lex Learning** (AutoML Backend)
No-code AutoML training and explainability engine.

**Capabilities:**
- Algorithm selection and comparison
- Hyperparameter optimization
- Cross-validation
- Model training and evaluation
- SHAP-based explainability
- Model persistence and versioning

**Technology**: Embedded Python runtime with sklearn, XGBoost, LightGBM, TensorFlow.

---

## Contributing

We welcome contributions from the community! Lex Machina is designed with modularity in mind, making it easy to extend.

### Areas for Contribution
- 🛠️ **New Algorithms**: Add support for additional ML algorithms
- 📊 **Visualizations**: Create new chart types and analysis tools
- 🧪 **Testing**: Improve test coverage and add benchmarks
- 📖 **Documentation**: Help make our docs more accessible

### Development Setup

```bash
# Clone the repository
git clone https://github.com/lex-machina/lex-machina
cd lex-machina

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js dependencies
npm install

# Run in development mode
npm run tauri dev
```
---

## License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

Lex Machina is built on the shoulders of giants:
- **Tauri** - for the secure, lightweight desktop framework
- **Polars** - for blazing-fast DataFrame operations
- **Apache Arrow** - for efficient data interchange
- **scikit-learn, XGBoost, LightGBM** - for robust ML algorithms
- **SHAP** - for model explainability
