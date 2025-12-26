# AGENTS.md - Lex Machina (LM) Context & Directives

> **SYSTEM INSTRUCTION:** This file contains the master context, architectural constraints, and development guidelines for the "Lex Machina" (LM) project. **Read this entire file before generating code, planning tasks, or writing documentation.**

---

## CRITICAL: Read This First

Before doing ANY work on this project, you MUST understand these four fundamental principles:

### 1. THIS IS A DESKTOP APPLICATION

**NOT a web app. NOT a website. This is a native DESKTOP APPLICATION.**

What this means:
- Web design patterns **DO NOT** translate to desktop UX
- No responsive layouts for mobile - this runs at fixed desktop resolution
- Native OS integrations (file dialogs, window controls, system menus)
- Dense information displays (like **VS Code**, **Excel**, **DaVinci Resolve**)
- No scroll-hijacking, no hero sections, no "above the fold" thinking
- Users expect desktop keyboard shortcuts (`Ctrl+S`, `Ctrl+O`, `Ctrl+W`, etc.)
- Context menus (right-click), toolbars, status bars are standard
- Resizable panes and panels, not fixed card layouts

**Reference Applications:**
- VS Code (panels, keyboard-first, dense information)
- Microsoft Excel (data grids, formula bar, status bar)
- DaVinci Resolve (professional desktop software)

**NOT:**
- Marketing landing pages
- SaaS web dashboards
- Mobile-first responsive designs

### 2. RUST SUPREMACY

**ALL logic lives in Rust. TypeScript is for UI RENDERING ONLY.**

| NEVER in TypeScript | ALWAYS in Rust |
|---------------------|----------------|
| Business logic | All business logic |
| Data transformations | All data processing (Polars) |
| State calculations | All state management |
| Sorting/filtering | All sorting/filtering |
| Validation | All validation |
| API calls | All external communication |

**The TypeScript frontend:**
- Receives JSON from Rust via Tauri commands
- Renders HTML/CSS based on that JSON
- Sends user events (clicks, keystrokes) to Rust via `invoke()`
- Subscribes to events from Rust via `listen()`
- **NOTHING ELSE**

### 3. ALWAYS FETCH LATEST DOCUMENTATION

Before implementing ANY feature, use MCP Context7 to fetch the latest documentation:

| Technology | Context7 Library ID | Purpose |
|------------|---------------------|---------|
| **Rust** | `/websites/doc_rust-lang_stable` | Rust language & std library |
| **Rust Book** | `/rust-lang/book` | Rust concepts & patterns |
| **Tauri 2.9** | `/websites/rs_tauri_2_9_5` | Desktop framework |
| **Next.js** | `/websites/nextjs` | Frontend framework |
| **React** | `/websites/react_dev` | UI library |
| **Polars (Rust)** | `/websites/pola_rs` | DataFrame operations |
| **Tailwind CSS** | `/websites/tailwindcss` | Styling |
| **lex-processing** | See `crates/lex-processing/AGENTS.md` | Preprocessing library |

**How to fetch docs:**
```
get-library-docs context7CompatibleLibraryID="/websites/rs_tauri_2_9_5" topic="commands"
```

### 4. ASK BEFORE ACTING

**Do NOT make assumptions on decisions that matter. Ask the user for clarification.**

Ask the user when:
- **Requirements are ambiguous** (if the task/feature isn't fully specified)
- **Trade-offs exist** (multiple valid approaches with different pros/cons)
- **Scope is unclear** (what's in/out of scope for this task)
- **User preference matters** (visual design, UX flows, naming conventions)
- **Something contradicts existing patterns** (you think a different approach is better)

Do NOT ask about:
- Obvious file locations (components go in `components/`, hooks go in `lib/hooks/`)
- Following existing patterns (if the codebase already does X, continue doing X)
- Trivial implementation details (variable names, import order)

**Examples of good questions:**
- "The preprocessing panel needs to show progress. Should it be a modal overlay or an inline section in the existing panel?"
- "I see two ways to handle cancellation: A stores the token in AppState, B creates it per-request. A allows cancel from anywhere, B is simpler. Which do you prefer?"
- "The AGENTS.md doesn't specify how to handle AI provider errors in the UI. Should we show a toast, inline error, or retry dialog?"

**When in doubt, ask.**

---

## 1. Quick Reference

### Workspace Commands

```bash
# From workspace root (lex-machina/)

# BUILD & DEVELOPMENT
# ═══════════════════════════════════════════════════════════════════════════════
cargo build                              # Build entire workspace
cargo build -p lex_machina               # Build Tauri backend only
cargo build -p lex-processing            # Build preprocessing library only
pnpm tauri dev                           # Start dev server + Tauri window
pnpm build                               # Build Next.js frontend
pnpm tauri build                         # Build production release

# TESTING
# ═══════════════════════════════════════════════════════════════════════════════
cargo test                               # Test all Rust crates
cargo test -p lex-processing             # Test preprocessing library (337 tests)
cargo test -p lex-processing --lib       # Test preprocessing unit tests only (318 tests)
cargo test -p lex_machina                # Test Tauri backend
pnpm lint                                # Lint TypeScript/Next.js

# CODE QUALITY
# ═══════════════════════════════════════════════════════════════════════════════
cargo clippy                             # Lint all Rust code (MUST pass with 0 warnings)
cargo clippy -p lex-processing           # Lint preprocessing library
cargo clippy -p lex_machina              # Lint Tauri backend
cargo fmt                                # Format Rust code
cargo fmt --check                        # Check Rust formatting

# DOCUMENTATION
# ═══════════════════════════════════════════════════════════════════════════════
cargo doc --workspace --open             # Generate and view all docs
cargo doc -p lex-processing --open       # View preprocessing library docs
```

### Code Style Summary

| Aspect | Convention |
|--------|------------|
| **Rust Edition** | 2024 |
| **Error Handling** | `thiserror` for custom errors, `anyhow` for application errors |
| **Logging** | `tracing` crate (`info!`, `debug!`, `warn!`, `error!`) |
| **State** | `parking_lot::RwLock` (faster than std) |
| **TypeScript** | Strict mode, `@/*` path aliases |
| **Components** | Arrow functions, default exports, `"use client"` directive |
| **Styling** | Tailwind CSS with `cn()` from `@/lib/utils` |
| **Naming** | Rust: `snake_case`, Components: `PascalCase`, TS files: `kebab-case` |

### Key File Locations

```
lex-machina/
├── AGENTS.md                     # THIS FILE - Primary onboarding document
├── Cargo.toml                    # Workspace root (resolver = "3")
│
├── src-tauri/                    # ═══ TAURI APPLICATION ═══
│   ├── Cargo.toml                # Tauri crate config
│   ├── tauri.conf.json           # Tauri configuration
│   └── src/
│       ├── lib.rs                # App entry, plugin setup, command registration
│       ├── main.rs               # Binary entry point
│       ├── state.rs              # AppState (RwLock-wrapped state)
│       ├── events.rs             # Event system (Rust -> Frontend)
│       └── commands/             # IPC command handlers
│           ├── mod.rs            # Re-exports all commands
│           ├── dialog.rs         # Native file dialogs
│           ├── file_io.rs        # CSV loading with Polars
│           ├── dataframe.rs      # Virtual scrolling, file operations
│           └── ui.rs             # UI state persistence
│
├── crates/
│   └── lex-processing/           # ═══ PREPROCESSING LIBRARY ═══
│       ├── AGENTS.md             # READ THIS for preprocessing API
│       ├── Cargo.toml            # Library config
│       └── src/                  # See crates/lex-processing/AGENTS.md
│
├── app/                          # ═══ NEXT.JS PAGES (rendering only) ═══
│   ├── layout.tsx                # Root layout
│   ├── page.tsx                  # Home page
│   ├── data/page.tsx             # Data view page
│   ├── analysis/page.tsx         # Analysis page
│   └── ml/page.tsx               # ML page
│
├── components/                   # ═══ REACT COMPONENTS (rendering only) ═══
│   ├── layout/                   # App shell components
│   │   ├── app-shell.tsx         # Main layout container
│   │   ├── nav-sidebar.tsx       # Left navigation
│   │   ├── context-sidebar.tsx   # Right context panel
│   │   ├── toolbar.tsx           # Top toolbar
│   │   └── status-bar.tsx        # Bottom status bar
│   ├── data-grid/                # Data grid components
│   │   ├── data-grid.tsx         # Main grid component
│   │   ├── grid-header.tsx       # Column headers
│   │   ├── grid-body.tsx         # Virtual scrolling body
│   │   ├── grid-cell.tsx         # Individual cell
│   │   ├── use-grid-data.ts      # Data fetching hook
│   │   └── use-grid-scroll.ts    # Scroll handling hook
│   └── ui/                       # Primitive UI components
│       ├── button.tsx
│       ├── resize-handle.tsx
│       ├── scrollbar.tsx
│       └── toast.tsx
│
├── lib/                          # ═══ UTILITIES & HOOKS ═══
│   ├── utils.ts                  # Utility functions (cn, etc.)
│   └── hooks/
│       ├── use-rust-event.ts     # Subscribe to Rust events
│       ├── use-file-state.ts     # File state management
│       └── use-app-status.ts     # App status hook
│
└── types/
    └── index.ts                  # TypeScript types (mirror Rust structs)
```

---

## 2. Project Identity & Mission

- **Project Name:** Lex Machina (formerly AfA)
- **Type:** Graduation Thesis Project (B.Sc. Data Science)
- **Authors:** Montaser Amoor & Rita Basbous
- **Core Mission:** Democratize data analytics for SMEs, non-profits, and non-technical individuals

### Key Value Proposition

A **No-Code**, **Local-First**, **Desktop** application that provides **Automated Machine Learning (AutoML)** with built-in **Explainable AI (XAI)**.

### Philosophy

| Principle | Description |
|-----------|-------------|
| **Privacy-First** | All processing happens locally. No data leaves the user's machine. |
| **Accessibility** | Non-technical users can perform advanced analytics without coding. |
| **Transparency** | Every ML decision is explainable. No black boxes. |
| **Local-First** | Works offline. No cloud dependency for core features. |
| **Desktop-Native** | Feels like professional desktop software, not a web app. |

---

## 3. Technical Stack (Strict Constraints)

> **CRITICAL:** Always verify and use the **latest documentation** via MCP Context7. Never rely on outdated patterns.

### Workspace Structure

```toml
# Cargo.toml (workspace root)
[workspace]
resolver = "3"
members = ["src-tauri", "crates/lex-processing"]

[workspace.dependencies]
polars = { version = "0.51", features = ["lazy", "csv", "dtype-full", ...] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"
parking_lot = "0.12"
tokio = { version = "1.48", features = ["full"] }
```

### Technology Stack

#### Frontend Layer (RENDERING ONLY)

| Technology | Version | Purpose | Context7 ID |
|------------|---------|---------|-------------|
| Next.js | 15+ | Static site generation | `/websites/nextjs` |
| React | 19 | Component rendering | `/websites/react_dev` |
| TypeScript | Strict | Type safety | - |
| Tailwind CSS | 4 | Utility-first styling | `/websites/tailwindcss` |

**Constraints:**
- `"use client"` directive on ALL components (no SSR in Tauri)
- No Server Actions (doesn't work in Tauri)
- No `fetch()` to external APIs (Rust handles all external communication)
- Static export only (`output: 'export'` in next.config.ts)

#### Desktop/System Layer

| Technology | Version | Purpose | Context7 ID |
|------------|---------|---------|-------------|
| Tauri | 2.9+ | Desktop framework | `/websites/rs_tauri_2_9_5` |
| Rust | 2024 edition | Backend logic | `/websites/doc_rust-lang_stable` |

**Key Tauri Plugins:**
- `tauri-plugin-dialog` - Native file dialogs
- `tauri-plugin-log` - Logging (debug builds only)

#### Data & Logic Layer (Rust Supremacy)

| Technology | Version | Purpose | Context7 ID |
|------------|---------|---------|-------------|
| Polars | 0.51 | DataFrame operations | `/websites/pola_rs` |
| parking_lot | 0.12 | Fast RwLock | - |
| serde | 1.0 | Serialization | - |
| thiserror | 2.0 | Error types | - |
| tracing | 0.1 | Logging/tracing | - |

#### Preprocessing Library

| Crate | Purpose | Documentation |
|-------|---------|---------------|
| lex-processing | Data preprocessing, imputation, profiling | `crates/lex-processing/AGENTS.md` |

#### Machine Learning Engine (Future)

| Technology | Purpose |
|------------|---------|
| Python (sidecar) | ML model training |
| PyO3 | Rust-Python bindings |
| Scikit-learn | ML algorithms |
| SHAP/LIME | Explainability |

---

## 4. Architecture Overview

### The Rust-Heavy Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           USER INTERFACE                                    │
│                     (Next.js + React + Tailwind)                           │
│                                                                             │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│   │  Data Grid  │  │   Sidebar   │  │   Toolbar   │  │ Status Bar  │       │
│   └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                                             │
│   ONLY: Render HTML/CSS, handle user interactions, send to Rust            │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    │       TAURI IPC BRIDGE        │
                    │  Commands (TS->Rust)          │
                    │  Events (Rust->TS)            │
                    └───────────────┬───────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────────┐
│                           RUST BACKEND                                      │
│                      (src-tauri + lex-processing)                          │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                         AppState (RwLock)                           │  │
│   │  ┌──────────────────────┐  ┌──────────────────────────────────────┐ │  │
│   │  │ dataframe: Option<   │  │ ui_state: UIState                    │ │  │
│   │  │   LoadedDataFrame    │  │   - sidebar_width                    │ │  │
│   │  │   - df: DataFrame    │  │   - column_widths                    │ │  │
│   │  │   - file_info        │  │   - grid_scroll                      │ │  │
│   │  └──────────────────────┘  └──────────────────────────────────────┘ │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│   │   Commands   │  │    Events    │  │    Polars    │  │lex-processing│   │
│   │  (IPC in)    │  │  (IPC out)   │  │  (DataFrames)│  │(preprocessing)│  │
│   └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘   │
│                                                                             │
│   ALL: Business logic, data processing, state, validation, I/O             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Action (click, type, scroll)
         │
         ▼
┌─────────────────────────┐
│  TypeScript Component   │
│  (captures user event)  │
└───────────┬─────────────┘
            │ invoke("command_name", { args })
            ▼
┌─────────────────────────┐
│     Rust Command        │
│  (processes request)    │
│  - Validates input      │
│  - Updates AppState     │
│  - Performs logic       │
└───────────┬─────────────┘
            │ Returns Result<T> or emits event
            ▼
┌─────────────────────────┐
│  TypeScript Component   │
│  (receives response)    │
│  - Updates local state  │
│  - Re-renders UI        │
└─────────────────────────┘
```

### Current Implementation Status

#### Implemented

**Rust Backend (`src-tauri/`):**
- AppState with `parking_lot::RwLock`
- CSV loading with Polars (`load_file` command)
- Virtual scrolling (`get_rows` command)
- Event system (`AppEventEmitter` trait)
- UI state persistence (sidebar, columns, scroll)
- Native file dialogs (`tauri-plugin-dialog`)

**Frontend (`app/`, `components/`):**
- App shell layout (nav sidebar, context sidebar, toolbar, status bar)
- Data grid with virtual scrolling
- File loading flow with events
- `useRustEvent` hook for event subscriptions
- TypeScript types mirroring Rust structs

**Preprocessing Library (`crates/lex-processing/`):**
- Full preprocessing pipeline
- Dataset profiling and type inference
- Missing value imputation (KNN, statistical)
- Outlier detection and handling
- AI-guided and rule-based decisions
- Progress reporting and cancellation
- **337 passing tests**

#### Not Yet Implemented

| Component | Status | Location |
|-----------|--------|----------|
| Preprocessing Tauri commands | Not created | `src-tauri/src/commands/preprocessing.rs` |
| Preprocessing events | Not defined | `src-tauri/src/events.rs` |
| AppState preprocessing fields | Not added | `src-tauri/src/state.rs` |
| Preprocessing TypeScript types | Not added | `types/index.ts` |
| `usePreprocessing` hook | Not created | `lib/hooks/use-preprocessing.ts` |
| Preprocessing UI components | Not created | `components/preprocessing/` |
| Python ML sidecar | Not started | Future phase |

### Command Reference

| Command | Module | Purpose | Parameters | Returns |
|---------|--------|---------|------------|---------|
| `open_file_dialog` | dialog | Show native file picker | - | `Option<String>` |
| `load_file` | file_io | Load CSV into memory | `path: String` | `Result<FileInfo>` |
| `get_file_info` | file_io | Get cached file metadata | - | `Option<FileInfo>` |
| `get_rows` | dataframe | Fetch rows for virtual scroll | `start, count` | `Option<RowsResponse>` |
| `close_file` | dataframe | Close file, free memory | - | - |
| `get_ui_state` | ui | Get UI layout state | - | `UIState` |
| `set_sidebar_width` | ui | Update sidebar width | `width: f32` | - |
| `set_column_width` | ui | Update single column width | `col, width` | - |
| `set_column_widths` | ui | Update all column widths | `widths: Vec<f32>` | - |
| `get_grid_scroll` | ui | Get grid scroll position | - | `GridScrollPosition` |
| `set_grid_scroll` | ui | Update grid scroll position | `row_index, scroll_left` | - |

### Event Reference

| Event | Payload | Emitted When |
|-------|---------|--------------|
| `file:loaded` | `{ file_info: FileInfo }` | CSV successfully loaded |
| `file:closed` | `()` | File closed, memory freed |
| `app:loading` | `{ is_loading, message }` | Loading state changes |
| `app:error` | `{ code, message }` | Error occurs |

### TypeScript Types (from `types/index.ts`)

```typescript
// Core data types
interface ColumnInfo { name, dtype, null_count, width }
interface FileInfo { path, name, size_bytes, row_count, column_count, columns }
interface RowsResponse { rows, start, total_rows }

// UI state types
interface GridScrollPosition { row_index, scroll_left }
interface UIState { sidebar_width, column_widths, grid_scroll }

// Event payloads
interface FileLoadedPayload { file_info: FileInfo }
interface LoadingPayload { is_loading, message }
interface ErrorPayload { code, message }
```

---

## 5. Desktop UI/UX Design Directives

### THIS IS NOT A WEBSITE

The application must **NOT** look or behave like a website or web app.

| Aspect | Desktop Pattern | Web Anti-Pattern |
|--------|-----------------|------------------|
| **Density** | Pack information densely | Large whitespace, hero sections |
| **Layout** | Resizable panes, panels, docked windows | Fixed containers, cards, grids |
| **Navigation** | Keyboard shortcuts, context menus | Mouse-only, hamburger menus |
| **Feedback** | Status bar, progress in toolbar | Toast notifications only |
| **Menus** | Native context menus (right-click) | Custom dropdown menus |
| **Windows** | Native window controls | Custom close/minimize buttons |
| **Selection** | Multi-select with Shift/Ctrl | Single selection only |
| **Scrolling** | Native scroll, virtual scrolling | Smooth scroll, parallax, infinite scroll |
| **File Access** | Native file dialogs | Drag-and-drop upload only |

### Design Reference Models

Study these applications for UX patterns:

1. **VS Code**
   - Activity bar (left icon rail)
   - Sidebar with resizable panels
   - Editor area with tabs
   - Status bar with contextual information
   - Command palette (`Ctrl+Shift+P`)
   - Keyboard-first navigation

2. **Microsoft Excel**
   - Dense data grid with headers
   - Formula bar / toolbar
   - Status bar with selection info
   - Sheet tabs
   - Context menus for cells

3. **DaVinci Resolve**
   - Professional dark theme
   - Tabbed workspace pages
   - Dense control panels
   - Persistent playback controls

### Anti-Patterns to AVOID

```
DO NOT:
- Use "above the fold" thinking (this isn't a landing page)
- Add mobile-first responsive breakpoints
- Include hero images or marketing sections
- Use scroll-hijacking or parallax effects
- Show cookie consent banners (this is local software)
- Add "Sign up" or "Get started" CTAs
- Use hamburger menus for navigation
- Implement infinite scroll (use virtual scroll or pagination)
- Rely on toast-only feedback (use status bar)
- Create custom window chrome (use native Tauri window)
- Add large padding/margins between elements
- Use cards with shadows for every piece of content

DO:
- Design for 1920x1080 minimum resolution
- Use dense, information-rich layouts
- Implement fixed viewport with resizable panels
- Use native file dialogs via Tauri
- Add native context menus (right-click)
- Support keyboard shortcuts for all actions
- Show persistent state in status bar
- Use progress bars in toolbar/status bar
- Implement virtual scrolling for large datasets
- Follow OS conventions for your platform
```

### Keyboard Shortcuts (Standard Desktop Conventions)

| Shortcut | Action |
|----------|--------|
| `Ctrl+O` | Open file |
| `Ctrl+S` | Save / Export |
| `Ctrl+W` | Close file |
| `Ctrl+F` | Find in data |
| `Escape` | Cancel / Close modal |
| `Arrow keys` | Navigate grid cells |
| `Enter` | Confirm / Execute |
| `Delete` | Delete selected |

---

## 6. Development Patterns

### Adding a New Tauri Command

**Location:** `src-tauri/src/commands/`

1. Create or edit the appropriate module (e.g., `preprocessing.rs`)
2. Add the command function with `#[tauri::command]`
3. Export from `commands/mod.rs`
4. Register in `lib.rs` via `invoke_handler`

**Pattern:**

```rust
// src-tauri/src/commands/example.rs
use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub fn my_command(
    arg1: String,
    state: State<'_, AppState>,
) -> Result<ReturnType, ErrorType> {
    // 1. Acquire lock on state
    let guard = state.some_field.read(); // or .write() for mutation
    
    // 2. Perform logic
    
    // 3. Return result (automatically serialized to JSON)
    Ok(result)
}
```

### Adding a New Event

**Location:** `src-tauri/src/events.rs`

1. Add event name constant
2. Add payload struct (with `Serialize`)
3. Add method to `AppEventEmitter` trait
4. Implement for `AppHandle`

**Pattern:**

```rust
// Event name constant
pub const EVENT_MY_EVENT: &str = "my:event";

// Payload struct
#[derive(Debug, Clone, Serialize)]
pub struct MyEventPayload {
    pub field: String,
}

// Add to trait
pub trait AppEventEmitter {
    fn emit_my_event(&self, payload: &MyEventPayload);
}

// Implement for AppHandle
impl AppEventEmitter for AppHandle {
    fn emit_my_event(&self, payload: &MyEventPayload) {
        self.emit(EVENT_MY_EVENT, payload).ok();
    }
}
```

### Adding a Frontend Component

**Location:** `components/`

**Pattern:**

```tsx
"use client";

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useRustEvent } from "@/lib/hooks/use-rust-event";
import { cn } from "@/lib/utils";
import type { SomeType } from "@/types";

interface MyComponentProps {
  className?: string;
}

export default function MyComponent({ className }: MyComponentProps) {
  const [data, setData] = useState<SomeType | null>(null);

  // Subscribe to Rust events
  useRustEvent<SomeType>("event:name", (payload) => {
    setData(payload);
  });

  // Call Rust command
  const handleAction = async () => {
    const result = await invoke<SomeType>("command_name", { arg: "value" });
    setData(result);
  };

  return (
    <div className={cn("base-styles", className)}>
      {/* Render based on data from Rust */}
    </div>
  );
}
```

### State Management Pattern

**Rust Side (source of truth):**

```rust
// State lives in AppState with RwLock
pub struct AppState {
    pub my_field: RwLock<MyType>,
}

// Commands read/write state
#[tauri::command]
pub fn get_my_data(state: State<'_, AppState>) -> MyType {
    state.my_field.read().clone()
}

#[tauri::command]
pub fn set_my_data(data: MyType, state: State<'_, AppState>) {
    *state.my_field.write() = data;
}
```

**TypeScript Side (display only):**

```tsx
// Frontend caches for display, but Rust is source of truth
const [displayData, setDisplayData] = useState<MyType | null>(null);

// Fetch from Rust on mount
useEffect(() => {
  invoke<MyType>("get_my_data").then(setDisplayData);
}, []);

// Update goes to Rust first, then frontend updates from event
const updateData = async (newData: MyType) => {
  await invoke("set_my_data", { data: newData });
  // OR: subscribe to event that Rust emits after update
};
```

---

## 7. lex-processing Integration

### Overview

The `lex-processing` crate provides automated data preprocessing capabilities:

| Feature | Description |
|---------|-------------|
| **Dataset Profiling** | Automatic type inference, role detection, statistical analysis |
| **Quality Analysis** | Detect missing values, outliers, type mismatches, duplicates |
| **Decision Making** | AI-guided (OpenRouter, Gemini) or rule-based preprocessing strategies |
| **Data Cleaning** | Type correction, imputation (KNN, statistical), outlier handling |
| **Progress Reporting** | Real-time progress updates with sub-stage granularity |
| **Cancellation** | Thread-safe cancellation from UI |

### Full Documentation

**For the complete lex-processing API and Tauri integration guide, refer to:**

> **`crates/lex-processing/AGENTS.md`**

This document contains:
- Complete API reference (all types, methods, error codes)
- **Section 5: Tauri Integration Guide** (9 steps with full code examples)
- TypeScript types for frontend
- React hooks and component examples
- How to extend (add AI providers, imputers)

**IMPORTANT:** When working on preprocessing features, you MUST read `crates/lex-processing/AGENTS.md` for the authoritative API documentation. The summary below is for orientation only.

### Quick API Reference

```rust
use lex_processing::{
    // Core pipeline
    Pipeline, PipelineConfig, PipelineResult,
    
    // Progress and cancellation
    ProgressReporter, ProgressUpdate, PreprocessingStage, CancellationToken,
    
    // AI providers (requires "ai" feature)
    ai::{AIProvider, OpenRouterProvider, GeminiProvider},
    
    // Configuration
    OutlierStrategy, NumericImputation, CategoricalImputation,
    
    // Results
    PreprocessingSummary, DatasetProfile, ColumnProfile,
};

// Basic usage
let result = Pipeline::builder()
    .config(PipelineConfig::default())
    .cancellation_token(token)
    .on_progress(|update| { /* handle progress */ })
    .build()?
    .process(dataframe)?;
```

### Integration Status

| Component | Status | Next Step |
|-----------|--------|-----------|
| Workspace dependency | Done | - |
| Tauri commands | Missing | See `crates/lex-processing/AGENTS.md` Section 5 |
| Preprocessing events | Missing | See `crates/lex-processing/AGENTS.md` Section 5 |
| AppState fields | Missing | See `crates/lex-processing/AGENTS.md` Section 5 |
| TypeScript types | Missing | See `crates/lex-processing/AGENTS.md` Section 5 |
| React hook | Missing | See `crates/lex-processing/AGENTS.md` Section 5 |
| UI components | Missing | See `crates/lex-processing/AGENTS.md` Section 5 |

---

## 8. Development Roadmap

### Phase 1: Foundation - COMPLETE

- [x] Initialize Tauri v2 + Next.js + Rust project structure
- [x] Configure Next.js for SSG (`output: 'export'`)
- [x] Implement Rust-heavy communication bridge (Commands/Events)
- [x] Design VS Code-like layout shell (sidebars, toolbar, status bar)
- [x] Set up Cargo workspace with shared dependencies

### Phase 2: Data Grid & File Loading - COMPLETE

- [x] Rust-based CSV ingestion with Polars
- [x] Virtual scrolling data grid for large datasets
- [x] File metadata display in sidebar
- [x] Column type inference and display
- [x] UI state persistence (column widths, scroll position)
- [x] Native file dialogs via `tauri-plugin-dialog`

### Phase 2.5: Preprocessing Integration - IN PROGRESS

- [ ] Wire `lex-processing` library to Tauri commands
- [ ] Add preprocessing progress events
- [ ] Create preprocessing configuration UI
- [ ] Implement progress display component
- [ ] Add AI provider configuration

### Phase 3: Analytics & Profiling - PLANNED

- [ ] Dataset profiling UI (statistics, distributions)
- [ ] Data quality report visualization
- [ ] Column-level analysis views
- [ ] Issue highlighting in data grid

### Phase 4: ML & Customization - FUTURE

- [ ] Implement Python sidecar (PyO3)
- [ ] Build ML configuration panes
- [ ] Implement training loop with real-time progress
- [ ] Add SHAP/LIME explainability visualizations
- [ ] Model comparison and selection UI

---

## 9. Phase 2.5 Implementation Plan: Preprocessing Integration

> **STATUS:** IN PROGRESS
> **Last Updated:** 2024-12-26

This section contains the detailed implementation plan for integrating the `lex-processing` library into the Tauri application.

### 9.1 Overview

**Goals:**
- Add a **Processing Page** (`/processing`) for configuring and running preprocessing
- Add a **Settings Page** (`/settings`) for AI provider config and theme settings
- Update **Data Page** with tabs for Original vs Processed data with history
- Update **Navigation** with new nav items

**Key Decisions:**
- History is session-only (in-memory, max 10 entries) - disk persistence planned for future
- API keys are session-only (stored in AppState, not persisted)
- Processing runs in background thread via `tauri::async_runtime::spawn` - UI remains responsive
- Columns: visual selector showing all columns with data types, none selected by default, Select All/Deselect All buttons
- Rows: index range selection with basic filtering
- Theme: System/Light/Dark, defaults to system preference, immediate application

### 9.2 Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND (React)                               │
├─────────────────────────────────────────────────────────────────────────────┤
│  /processing                  │  /settings           │  /data               │
│  ├─ Column selector           │  ├─ AI provider      │  ├─ Tabs (Original/  │
│  ├─ Row range selector        │  ├─ API key input    │  │   Processed)       │
│  ├─ Config options            │  └─ Theme toggle     │  └─ History selector │
│  ├─ Start/Cancel buttons      │                      │                      │
│  └─ Progress panel (sidebar)  │                      │                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                              TAURI IPC                                       │
│  Commands: start_preprocessing, cancel_preprocessing, set_ai_provider, etc. │
│  Events: preprocessing:progress, preprocessing:complete, preprocessing:error │
├─────────────────────────────────────────────────────────────────────────────┤
│                              RUST BACKEND                                    │
│  AppState: + ai_provider, preprocessing_token, preprocessing_history        │
│  Commands: preprocessing.rs, settings.rs                                    │
│  Events: + preprocessing events                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 9.3 Task Breakdown

#### Phase A: Rust Backend (Foundation)

| Task | File | Status | Description |
|------|------|--------|-------------|
| A1 | `src-tauri/src/state.rs` | [ ] | Add preprocessing & settings state fields |
| A2 | `src-tauri/src/events.rs` | [ ] | Add preprocessing events |
| A3 | `src-tauri/src/commands/preprocessing.rs` | [ ] | Create preprocessing commands |
| A4 | `src-tauri/src/commands/settings.rs` | [ ] | Create settings commands |
| A5 | `src-tauri/src/commands/mod.rs` | [ ] | Export new command modules |
| A6 | `src-tauri/src/lib.rs` | [ ] | Register new commands |

**A1: State Updates (`state.rs`)**

New fields to add:
```rust
pub struct AppState {
    // Existing
    pub dataframe: RwLock<Option<LoadedDataFrame>>,
    pub ui_state: RwLock<UIState>,
    
    // NEW: Preprocessing
    pub ai_provider_config: RwLock<Option<AIProviderConfig>>,
    pub preprocessing_token: RwLock<Option<CancellationToken>>,
    pub preprocessing_history: RwLock<Vec<PreprocessingHistoryEntry>>, // Max 10
    pub processed_dataframe: RwLock<Option<LoadedDataFrame>>,
    
    // NEW: Settings
    pub theme: RwLock<Theme>,
}
```

New types:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderConfig {
    pub provider: AIProviderType,
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIProviderType { None, OpenRouter, Gemini }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessingHistoryEntry {
    pub id: String,                        // UUID
    pub timestamp: i64,                    // Unix timestamp
    pub config: PreprocessingConfigSnapshot,
    pub summary: PreprocessingSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Theme { #[default] System, Light, Dark }
```

**A2: Events Updates (`events.rs`)**

New events:
```rust
pub const EVENT_PREPROCESSING_PROGRESS: &str = "preprocessing:progress";
pub const EVENT_PREPROCESSING_COMPLETE: &str = "preprocessing:complete";
pub const EVENT_PREPROCESSING_ERROR: &str = "preprocessing:error";
pub const EVENT_PREPROCESSING_CANCELLED: &str = "preprocessing:cancelled";
pub const EVENT_THEME_CHANGED: &str = "settings:theme-changed";
```

**A3: Preprocessing Commands (`commands/preprocessing.rs`)**

| Command | Purpose | Parameters | Returns |
|---------|---------|------------|---------|
| `start_preprocessing` | Start pipeline in background | `PreprocessingRequest` | `Result<(), Error>` |
| `cancel_preprocessing` | Cancel running pipeline | - | - |
| `get_preprocessing_history` | Get history entries | - | `Vec<HistoryEntry>` |
| `load_history_entry` | Load a previous result | `entry_id: String` | `Result<(), Error>` |
| `get_processed_rows` | Virtual scroll for processed data | `start, count` | `Option<RowsResponse>` |
| `get_processed_file_info` | Get processed data info | - | `Option<FileInfo>` |
| `clear_preprocessing_history` | Clear all history | - | - |

**A4: Settings Commands (`commands/settings.rs`)**

| Command | Purpose | Parameters | Returns |
|---------|---------|------------|---------|
| `get_ai_provider_config` | Get current AI config | - | `Option<AIProviderConfig>` |
| `set_ai_provider_config` | Set AI config | `Option<AIProviderConfig>` | - |
| `get_theme` | Get current theme | - | `Theme` |
| `set_theme` | Set theme | `Theme` | - |
| `validate_ai_api_key` | Test API key validity | `provider, api_key` | `Result<bool, String>` |

---

#### Phase B: TypeScript Types & Hooks

| Task | File | Status | Description |
|------|------|--------|-------------|
| B1 | `types/index.ts` | [ ] | Add all preprocessing & settings types |
| B2 | `lib/hooks/use-preprocessing.ts` | [ ] | Preprocessing state & operations hook |
| B3 | `lib/hooks/use-processed-data.ts` | [ ] | Processed DataFrame access hook |
| B4 | `lib/hooks/use-settings.ts` | [ ] | App settings hook |
| B5 | `lib/hooks/use-theme.ts` | [ ] | Theme application hook |

**B1: New TypeScript Types**

```typescript
// Preprocessing
export type OutlierStrategy = "cap" | "remove" | "median" | "keep";
export type NumericImputation = "mean" | "median" | "knn" | "zero" | "drop";
export type CategoricalImputation = "mode" | "constant" | "drop";

export interface PreprocessingRequest {
  selected_columns: string[];
  row_range: RowRange | null;
  config: PipelineConfigRequest;
}

export interface RowRange { start: number; end: number; }

export interface PipelineConfigRequest {
  missing_column_threshold: number;
  missing_row_threshold: number;
  outlier_strategy: OutlierStrategy;
  numeric_imputation: NumericImputation;
  categorical_imputation: CategoricalImputation;
  enable_type_correction: boolean;
  remove_duplicates: boolean;
  knn_neighbors: number;
  use_ai_decisions: boolean;
  target_column: string | null;
}

export interface PreprocessingHistoryEntry {
  id: string;
  timestamp: number;
  config: PipelineConfigRequest;
  summary: PreprocessingSummary;
}

export interface PreprocessingSummary {
  duration_ms: number;
  rows_before: number;
  rows_after: number;
  rows_removed: number;
  columns_before: number;
  columns_after: number;
  columns_removed: number;
  issues_found: number;
  issues_resolved: number;
  data_quality_score_before: number;
  data_quality_score_after: number;
  actions: PreprocessingAction[];
  column_summaries: ColumnSummary[];
  warnings: string[];
}

export type PreprocessingStage =
  | "initializing" | "profiling" | "quality_analysis" | "type_correction"
  | "decision_making" | "cleaning" | "imputation" | "outlier_handling"
  | "report_generation" | "complete" | "cancelled" | "failed";

// Settings
export type AIProviderType = "none" | "openrouter" | "gemini";
export interface AIProviderConfig { provider: AIProviderType; api_key: string; }
export type Theme = "system" | "light" | "dark";
```

---

#### Phase C: UI Components

| Task | File | Status | Description |
|------|------|--------|-------------|
| C1 | `components/ui/tabs.tsx` | [ ] | Reusable tab component |
| C2 | `components/ui/checkbox.tsx` | [ ] | Checkbox for column selection |
| C3 | `components/ui/select.tsx` | [ ] | Select dropdown for config options |
| C4 | `components/ui/slider.tsx` | [ ] | Slider for threshold values |
| C5 | `components/ui/input.tsx` | [ ] | Text input for API keys, etc. |
| C6 | `components/ui/toggle.tsx` | [ ] | Toggle switch for boolean settings |
| C7 | `components/ui/progress-bar.tsx` | [ ] | Progress bar for preprocessing status |

---

#### Phase D: Processing Page

| Task | File | Status | Description |
|------|------|--------|-------------|
| D1 | `components/preprocessing/column-selector.tsx` | [ ] | Visual column selection with data types |
| D2 | `components/preprocessing/row-range-selector.tsx` | [ ] | Row range selection |
| D3 | `components/preprocessing/config-panel.tsx` | [ ] | Preprocessing configuration options |
| D4 | `components/preprocessing/progress-panel.tsx` | [ ] | Right sidebar progress display |
| D5 | `components/preprocessing/results-panel.tsx` | [ ] | Results summary after processing |
| D6 | `components/preprocessing/history-list.tsx` | [ ] | Processing history list |
| D7 | `components/preprocessing/dataset-preview.tsx` | [ ] | Dataset summary before processing |
| D8 | `app/processing/page.tsx` | [ ] | Main processing page |

**D1: Column Selector Features**
- Shows all columns with checkboxes
- Shows data type badge for each column
- "Select All" / "Deselect All" buttons
- None selected by default

**D4: Progress Panel Features**
- Current stage display with sub-stage
- Overall progress bar + stage progress bar
- Progress message
- Items processed / total
- Cancel button
- Time elapsed

**D8: Processing Page Layout**
```
┌─────────────────────────────────────────────────────────────────────────┐
│ [Nav Sidebar] │           MAIN AREA              │  [Right Sidebar]    │
│               │                                   │                     │
│   Home        │  ┌─────────────────────────────┐  │  Progress Panel     │
│   Data        │  │  Dataset Preview            │  │  ├─ Current Stage  │
│   Processing ←│  │  ├─ Rows/columns count      │  │  ├─ Sub-progress   │
│   Analysis    │  └─────────────────────────────┘  │  ├─ Time elapsed   │
│   ML          │                                   │  └─ Cancel button  │
│               │  ┌─────────────────────────────┐  │                     │
│   ─────────   │  │  Column Selector            │  │  Results Summary    │
│   Settings    │  │  [Select All] [Deselect]    │  │  (after completion) │
│               │  │  ☐ col1 (int64)             │  │  ├─ Rows processed  │
│               │  │  ☐ col2 (string)            │  │  ├─ Issues found    │
│               │  └─────────────────────────────┘  │  └─ [View Results]  │
│               │                                   │                     │
│               │  ┌─────────────────────────────┐  │  History List       │
│               │  │  Row Range Selector         │  │  ├─ Entry 1         │
│               │  └─────────────────────────────┘  │  ├─ Entry 2         │
│               │                                   │  └─ [Clear]         │
│               │  ┌─────────────────────────────┐  │                     │
│               │  │  Config Panel               │  │                     │
│               │  │  ├─ Thresholds              │  │                     │
│               │  │  ├─ Imputation methods      │  │                     │
│               │  │  └─ AI toggle               │  │                     │
│               │  └─────────────────────────────┘  │                     │
│               │                                   │                     │
│               │  [▶ Start Processing]             │                     │
└─────────────────────────────────────────────────────────────────────────┘
```

---

#### Phase E: Settings Page

| Task | File | Status | Description |
|------|------|--------|-------------|
| E1 | `components/settings/ai-provider-section.tsx` | [ ] | AI provider configuration |
| E2 | `components/settings/theme-section.tsx` | [ ] | Theme settings |
| E3 | `app/settings/page.tsx` | [ ] | Settings page |

**E1: AI Provider Section Features**
- Provider selection (None, OpenRouter, Gemini)
- API key input (masked)
- Validate button
- Status indicator (valid/invalid/untested)

**E2: Theme Section Features**
- Three options: System, Light, Dark
- Radio buttons or segmented control
- Immediate preview

---

#### Phase F: Data Page Updates

| Task | File | Status | Description |
|------|------|--------|-------------|
| F1 | `app/data/page.tsx` | [ ] | Add tabs for Original/Processed |
| F2 | `components/data-grid/processed-data-grid.tsx` | [ ] | Data grid for processed data |

**F1: Data Page Changes**
- Add Tabs component at top
- Tab 1: "Original" (existing DataGrid)
- Tab 2: "Processed" (disabled until processing done)
- History dropdown in Processed tab header

---

#### Phase G: Navigation Updates

| Task | File | Status | Description |
|------|------|--------|-------------|
| G1 | `components/layout/nav-sidebar.tsx` | [ ] | Add Processing and Settings nav items |
| G2 | `app/layout.tsx` | [ ] | Apply theme hook |

**G1: Navigation Structure**
```
Home
Data
Processing    ← NEW (requiresFile: true)
Analysis
ML
─────────
Settings      ← NEW (at bottom)
```

---

### 9.4 File Summary

#### New Files (25 files)

**Rust (2 new files):**
- `src-tauri/src/commands/preprocessing.rs`
- `src-tauri/src/commands/settings.rs`

**TypeScript Hooks (4 files):**
- `lib/hooks/use-preprocessing.ts`
- `lib/hooks/use-processed-data.ts`
- `lib/hooks/use-settings.ts`
- `lib/hooks/use-theme.ts`

**UI Components (7 files):**
- `components/ui/tabs.tsx`
- `components/ui/checkbox.tsx`
- `components/ui/select.tsx`
- `components/ui/slider.tsx`
- `components/ui/input.tsx`
- `components/ui/toggle.tsx`
- `components/ui/progress-bar.tsx`

**Preprocessing Components (7 files):**
- `components/preprocessing/column-selector.tsx`
- `components/preprocessing/row-range-selector.tsx`
- `components/preprocessing/config-panel.tsx`
- `components/preprocessing/progress-panel.tsx`
- `components/preprocessing/results-panel.tsx`
- `components/preprocessing/history-list.tsx`
- `components/preprocessing/dataset-preview.tsx`

**Settings Components (2 files):**
- `components/settings/ai-provider-section.tsx`
- `components/settings/theme-section.tsx`

**Pages (2 files):**
- `app/processing/page.tsx`
- `app/settings/page.tsx`

**Data Grid (1 file):**
- `components/data-grid/processed-data-grid.tsx`

#### Modified Files (8 files)
- `src-tauri/src/state.rs`
- `src-tauri/src/events.rs`
- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/lib.rs`
- `types/index.ts`
- `components/layout/nav-sidebar.tsx`
- `app/data/page.tsx`
- `app/layout.tsx`

---

### 9.5 Implementation Order

**Week 1: Rust Backend**
1. A1: Update state.rs
2. A2: Update events.rs
3. A3: Create preprocessing.rs
4. A4: Create settings.rs
5. A5: Update commands/mod.rs
6. A6: Update lib.rs
7. Run `cargo clippy` and `cargo test`

**Week 2: TypeScript Foundation**
1. B1: Update types/index.ts
2. B2-B5: Create all hooks
3. Run `pnpm lint`

**Week 3: UI Components**
1. C1-C7: Create UI primitives
2. D1-D7: Create preprocessing components

**Week 4: Pages & Integration**
1. D8: Create /processing page
2. E1-E3: Create settings components and /settings page
3. F1-F2: Update /data page with tabs
4. G1-G2: Update navigation and theme
5. End-to-end testing

---

### 9.6 Validation Checklist

Before marking Phase 2.5 as complete:

- [ ] `cargo clippy -p lex_machina` passes with 0 warnings
- [ ] `cargo test -p lex_machina` passes
- [ ] `pnpm lint` passes
- [ ] Processing page shows "Load a file first" when no file loaded
- [ ] Can select/deselect columns with Select All/Deselect All
- [ ] Can configure all preprocessing options
- [ ] Processing runs in background (UI remains responsive)
- [ ] Progress updates display in real-time
- [ ] Can cancel processing mid-way
- [ ] Processed data appears in Data page "Processed" tab
- [ ] History shows last 10 processing runs
- [ ] Can load previous history entries
- [ ] Settings page shows AI provider options
- [ ] Settings page shows theme options (System/Light/Dark)
- [ ] Theme changes apply immediately
- [ ] Navigation shows Processing and Settings items

---

## 10. Key Directives for Agents

### A. Documentation Fetching (MANDATORY)

**Before implementing ANY feature**, fetch the latest documentation using MCP Context7:

```
# Tauri commands/events
get-library-docs context7CompatibleLibraryID="/websites/rs_tauri_2_9_5" topic="commands"

# Polars DataFrame operations
get-library-docs context7CompatibleLibraryID="/websites/pola_rs" topic="dataframe"

# React hooks
get-library-docs context7CompatibleLibraryID="/websites/react_dev" topic="hooks"

# Rust error handling
get-library-docs context7CompatibleLibraryID="/websites/doc_rust-lang_stable" topic="error handling"
```

### B. Rust Supremacy Rules

1. **All calculations in Rust** - Even simple ones like "rows remaining" or "percentage complete"
2. **All data transformations in Rust** - Polars handles everything
3. **All validation in Rust** - Frontend never validates input
4. **All sorting/filtering in Rust** - Frontend just displays
5. **State lives in Rust** - Frontend caches for display only
6. **All file I/O in Rust** - Use Tauri plugins for native dialogs

### C. Code Quality Standards

**Rust:**
- `cargo clippy` MUST pass with **0 warnings**
- Handle all `Result`/`Option` explicitly (no `unwrap()` except in tests)
- Use `thiserror` for custom error types
- Use `tracing` for logging (`info!`, `debug!`, `warn!`, `error!`)
- Add doc comments (`///`) to all public items
- Use `parking_lot::RwLock` for state (faster than std)

**TypeScript:**
- Strict mode enabled (no `any` types)
- All components are arrow functions with default exports
- All components have `"use client"` directive
- Use `@/` path aliases for imports
- Use `cn()` from `@/lib/utils` for conditional classes
- Types must mirror Rust structs exactly

### D. The "Local-First" Constraint

- Assume **offline** operation for all core features
- Optimize for **4GB-8GB RAM** machines
- Use lazy loading / virtual scrolling for large datasets
- Store user preferences locally (not in cloud)
- All data processing happens on the user's machine
- Network features (AI providers) must be optional

### E. Desktop UX Requirements

- Follow desktop application conventions (VS Code, Excel)
- Support keyboard shortcuts for all major actions
- Use native OS dialogs and context menus
- Provide status bar feedback for operations
- Implement resizable panels and panes
- Dense information display, minimal whitespace

---

## 11. Known Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Next.js SSR features don't work in Tauri | Build failures, runtime errors | Use `"use client"` on ALL components. No Server Actions. Static export only. |
| UI feeling "web-like" | Poor UX, unprofessional appearance | Follow desktop reference models (VS Code, Excel). Use dense layouts. Native menus. |
| Python dependencies on user machines | Installation complexity | Bundle standalone Python environment or use PyO3 embedded interpreter. |
| Large dataset memory usage | OOM errors, slow performance | Use Polars lazy evaluation. Virtual scrolling. Streaming for huge files. |
| AI provider network dependency | Feature unavailable offline | Rule-based fallback when AI unavailable. Clear offline indicators. |
| Cross-platform inconsistencies | Different behavior on Win/Mac/Linux | Test on all platforms. Use Tauri's cross-platform APIs. |

---

## 12. Appendix: Quick Reference Tables

### Workspace Dependencies

```toml
[workspace.dependencies]
polars = { version = "0.51", features = ["lazy", "csv", "dtype-full", "parquet", "describe", "strings"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
anyhow = "1.0"
tokio = { version = "1.48", features = ["full"] }
parking_lot = "0.12"
chrono = "0.4"
tracing = "0.1"
log = "0.4"
```

### Error Codes

| Code | Description |
|------|-------------|
| `FILE_NOT_FOUND` | File doesn't exist at path |
| `FILE_READ_ERROR` | Failed to read file (I/O, permissions) |
| `FILE_PARSE_ERROR` | Failed to parse CSV |
| `FILE_METADATA_ERROR` | Failed to get file metadata |
| `UNKNOWN_ERROR` | Generic/unexpected error |

### Event Names

```typescript
const RUST_EVENTS = {
  FILE_LOADED: "file:loaded",
  FILE_CLOSED: "file:closed",
  LOADING: "app:loading",
  ERROR: "app:error",
  // Future: preprocessing events
  // PREPROCESSING_PROGRESS: "preprocessing:progress",
  // PREPROCESSING_COMPLETE: "preprocessing:complete",
} as const;
```

### Context7 Library IDs

| Library | Context7 ID | Use For |
|---------|-------------|---------|
| Rust (full docs) | `/websites/doc_rust-lang_stable` | Language reference, std library |
| Rust Book | `/rust-lang/book` | Concepts, patterns, tutorials |
| Tauri 2.9 | `/websites/rs_tauri_2_9_5` | Commands, events, plugins, window |
| Next.js | `/websites/nextjs` | Pages, routing, configuration |
| React | `/websites/react_dev` | Hooks, components, patterns |
| Polars | `/websites/pola_rs` | DataFrame operations, expressions |
| Tailwind CSS | `/websites/tailwindcss` | Utility classes, configuration |

---

## Contact

- **Project:** Lex Machina (Graduation Thesis)
- **Authors:** Montaser Amoor & Rita Basbous
