# AGENTS.md - Lex Machina (LM) Context & Directives

> **SYSTEM INSTRUCTION:** This file contains the master context, architectural constraints, and development guidelines for the "Lex Machina" (LM) project. Read this before generating code, planning tasks, or writing documentation.

---

## Quick Reference for Agents

### Commands
- **Desktop Dev:** `pnpm tauri dev` | **Build:** `pnpm tauri build`
- **Frontend only:** `pnpm dev` | `pnpm build` | `pnpm lint`
- **Rust (in src-tauri/):** `cargo build` | `cargo clippy` | `cargo test`
- **Single Rust test:** `cargo test <test_name>` (no TS test framework configured)

### Code Style
- **TypeScript:** Strict mode, `@/*` path aliases, UI rendering ONLY
- **Components:** Arrow functions, default exports, `"use client"` directive required
- **Imports:** External packages → `@/` aliases → relative paths
- **Styling:** Tailwind CSS with `cn()` from `@/lib/utils`
- **Rust:** `clippy` required, handle all `Result`/`Option`, snake_case
- **Naming:** Components PascalCase, TS files kebab-case, Rust snake_case
- **Architecture:** All business logic, data processing, and state in Rust. TypeScript renders UI only.
- **Desktop UX:** No web patterns. Dense layouts, resizable panes, keyboard shortcuts. Reference VS Code/Excel.

---

## 1. Project Identity & Mission

- **Project Name:** Lex Machina (formerly AfA).
- **Type:** Graduation Thesis Project (B.Sc. Data Science).
- **Authors:** Montaser Amoor & Rita Basbous.
- **Core Mission:** Democratize data analytics for SMEs, non-profits, and non-technical individuals.
- **Key Value Proposition:** A **No-Code**, **Local-First**, **Desktop** application that provides **Automated Machine Learning (AutoML)** with built-in **Explainable AI (XAI)**.
- **Philosophy:** Privacy-first, Accessibility, and Transparency.

---

## 2. Technical Stack (Strict Constraints & Documentation)

> **CRITICAL RULE:** Agents must always verify and utilize the **latest available documentation** linked below. Do not rely on deprecated patterns (e.g., Pages Router for Next.js, Tauri v1 patterns).

### **Frontend (Presentation Layer)**

- **Framework:** [Next.js](https://nextjs.org/docs) (configured for static export) with [React](https://react.dev/reference/react).
- **Language:** [TypeScript](https://www.typescriptlang.org/docs/) (Strict typing required).
- **Styling:** [Tailwind CSS](https://tailwindcss.com/docs).
- **Components:** **All components must be CUSTOM**. Do not rely on heavy pre-built component libraries unless they are lightweight headless primitives styled with Tailwind.
- **Visualization:** [Recharts](https://recharts.org/en-US/api) & [D3.js](https://d3js.org/getting-started).

### **Desktop/System Layer (Application Layer)**

- **Framework:** [Tauri 2.0+](https://v2.tauri.app/).
- **Core Backend:** [Rust](https://doc.rust-lang.org/std/).
- **Database:** [SQLite](https://www.sqlite.org/docs.html) (local persistence).

### **Data & Logic Layer (The "Rust Supremacy" Rule)**

- **Primary Logic:** **Rust**.
- **Data Engine:** [Polars (Rust)](https://docs.pola.rs/) | [Polars Crate Docs](https://docs.rs/polars/latest/polars/).
- **Logic Constraint:** **TypeScript is for UI rendering ONLY.** All business logic, data processing, validation, sorting, state calculations, and orchestration must happen in Rust.

### **Machine Learning Engine**

- **Language:** [Python](https://docs.python.org/3/) (sidecar/embedded via PyO3).
- **Integration:** [PyO3](https://pyo3.rs/v0.27.1/) (Rust bindings for Python).
- **Libraries:**
  - [Scikit-learn](https://scikit-learn.org/stable/user_guide.html)
  - [LightGBM](https://lightgbm.readthedocs.io/en/latest/)
  - [XGBoost](https://xgboost.readthedocs.io/en/stable/)
  - [Optuna](https://optuna.readthedocs.io/en/stable/) (Hyperparameter tuning)
  - [SHAP](https://shap.readthedocs.io/en/latest/) (Explainability)
  - [LIME](https://lime-ml.readthedocs.io/en/latest/) (Explainability)

---

## 3. UI/UX Design Directives

### **"Business-Grade" Desktop Aesthetic**

- **NOT a Website:** The application must **not** look like a website or a web-app.
- **Reference Models:** **VS Code, Microsoft Excel, Davinci Resolve**.
- **Design Characteristics:**
  - **High Density:** Maximize screen real estate. Avoid massive whitespace typical of landing pages.
  - **Structure:** Use resizable panes, status bars, context menus (right-click), and toolbars.
  - **Interactivity:** Support keyboard shortcuts for power users.
  - **Feedback:** Instant visual feedback (loaders, progress bars) for local operations.

---

## 4. System Architecture & Workflows

### The "Rust-Heavy" Architecture

1.  **View:** Next.js renders the state provided by Rust. It sends user actions (events) to Rust.
2.  **Controller/Model:** Rust receives events, executes **ALL** business logic, modifies state, and pushes updates back to the UI.
3.  **Data Ops:** Polars (Rust) handles all data manipulation.
4.  **ML Ops:** Rust orchestrates Python processes via PyO3 or IPC.

### Workflow: Customization vs. Defaults

The system must support two distinct user modes seamlessly:

1.  **Guided/Default Mode:** "I have a CSV, just give me insights." The system uses safe defaults for cleaning and AutoML budgets.
2.  **Power/Custom Mode:** "I want to control everything." The user can override:
    - Imputation strategies (Mean vs. Median vs. KNN).
    - AutoML time budgets and metric optimization goals.
    - Specific model families to include/exclude.
    - Preprocessing pipeline steps.

---

## 5. Key Directives for Agents

### A. Coding Standards

- **Rust (The Brain):**
  - Use `clippy`.
  - Handle all `Result`/`Option`.
  - **Performance is paramount.**
- **TypeScript (The Face):**
  - Keep it "dumb." It receives JSON from Rust and renders HTML.
  - No complex calculations in the webview thread.
- **Documentation:**
  - Always refer to the latest official docs linked in Section 2.

### B. The "Local-First" Constraint

- Assume **offline** operation.
- Optimize for **4GB-8GB RAM**.
- Use streaming/lazy loading in Polars for large files.

### C. Explainable AI (XAI)

- **SHAP/LIME:** Mandatory.
- **Context:** Translate math into business insights in the UI.

---

## 6. Development Roadmap

**Current Phase:** Implementation

### Phase 1: Foundation

- [ ] Initialize Tauri v2 + Next.js + Rust project structure.
- [ ] Configure Next.js for SSG (Static Site Generation) / `output: export`.
- [ ] Implement "Rust-heavy" communication bridge (Commands/Events).
- [ ] Design the "VS Code-like" layout shell (Sidebars, Panes, Status Bar).

### Phase 2: Analytics Engine

- [ ] Rust-based CSV/Excel ingestion (Polars).
- [ ] Rust-based Data Profiling (Histograms/Stats).
- [ ] Custom UI components for Data Grids and Charts.

### Phase 3: ML & Customization

- [ ] Implement Python sidecar.
- [ ] Build "Settings/Configuration" panes for granular ML control.
- [ ] Implement Training Loop with real-time progress feedback to UI.

---

## 7. Known Risks & Mitigations

- **Risk:** Next.js SSR features don't work in Tauri.
  - _Mitigation:_ Strictly use Client Components (`"use client"`) and `useEffect` / `react-query` or Tauri's async commands for data fetching. No Server Actions.
- **Risk:** UI feeling "web-like."
  - _Mitigation:_ Disable browser default context menus, enforce system fonts, prevent text selection where inappropriate, use dense layout.
- **Risk:** Python dependencies on user machines.
  - _Mitigation:_ Bundle a standalone Python environment or Docker container (if permissible) within the Tauri sidecar.
