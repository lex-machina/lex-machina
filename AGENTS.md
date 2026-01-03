# AGENTS.md - Lex Machina (LM) Context & Directives

> **SYSTEM INSTRUCTION:** This file contains the master context and architectural constraints for the "Lex Machina" project. Read this before generating code or planning tasks.

---

## CRITICAL: Four Principles

### 1. THIS IS A DESKTOP APPLICATION

**NOT a web app. NOT a website. This is a native DESKTOP APPLICATION.**

- Dense information displays (like **VS Code**, **Excel**, **DaVinci Resolve**)
- Native OS integrations (file dialogs, window controls, context menus)
- Resizable panes and panels, status bars, toolbars
- No responsive layouts, no hero sections, no "above the fold" thinking
- Muted, consistent color scheme (no colorful badges)

**NOT:** Marketing pages, SaaS dashboards, mobile-first designs

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
- Receives JSON from Rust via Tauri commands (`invoke()`)
- Renders HTML/CSS based on that JSON
- Sends user events to Rust, subscribes to events via `listen()`
- **NOTHING ELSE**

### 3. FETCH DOCUMENTATION BEFORE IMPLEMENTING

Use MCP Context7 to fetch latest documentation before implementing features:

| Technology | Context7 Library ID |
|------------|---------------------|
| Rust | `/websites/doc_rust-lang_stable` |
| Tauri 2.9 | `/websites/rs_tauri_2_9_5` |
| Next.js 15 | `/websites/nextjs` |
| React 19 | `/websites/react_dev` |
| Polars | `/websites/pola_rs` |
| Tailwind CSS 4 | `/websites/tailwindcss` |

```
get-library-docs context7CompatibleLibraryID="/websites/rs_tauri_2_9_5" topic="commands"
```

### 4. ASK BEFORE ACTING

**Do NOT make assumptions on decisions that matter. The user is here to help, not just to request.**

Communicate and ask when:
- Requirements are ambiguous or incomplete
- Trade-offs exist between different approaches
- Scope is unclear (what's in/out)
- User preference matters (design, UX, naming)
- You're unsure about the right approach

Do NOT ask about: obvious file locations, following existing patterns, trivial details.

**Remember:** The user wants to collaborate, not just delegate. When in doubt, ask rather than assume.

---

## 1. Quick Reference

### Commands

```bash
# Development
pnpm tauri dev                    # Start dev server + Tauri window
cargo build                       # Build workspace
cargo test                        # Test all crates

# Code Quality (MUST pass before committing)
cargo clippy                      # 0 warnings required
cargo fmt --check                 # Formatting check
pnpm lint                         # TypeScript linting
```

### Code Style

| Aspect | Convention |
|--------|------------|
| Rust | Edition 2024, `thiserror` errors, `tracing` logging, `parking_lot::RwLock` |
| TypeScript | Strict mode, `@/*` paths, `"use client"` on all components |
| Components | Arrow functions, default exports, `cn()` for classes |
| Icons | Lucide React only (no inline SVGs) |
| Naming | Rust: `snake_case`, Components: `PascalCase`, TS files: `kebab-case` |

---

## 2. Project Identity

- **Project:** Lex Machina (Graduation Thesis - B.Sc. Data Science)
- **Authors:** Montaser Amoor & Rita Basbous
- **Mission:** Democratize data analytics for SMEs and non-technical users

**Value Proposition:** No-Code, Local-First, Desktop AutoML with Explainable AI (XAI)

**Philosophy:** Privacy-First (local processing), Accessibility (no coding required), Transparency (explainable decisions), Desktop-Native (professional software UX)

---

## 3. Technical Stack

### Frontend (RENDERING ONLY)
- Next.js 15+ (static export, `"use client"` on all components)
- React 19, TypeScript (strict), Tailwind CSS 4, Lucide React

**Constraints:** No SSR, No Server Actions, No `fetch()` to external APIs

### Backend (ALL LOGIC)
- Tauri 2.9+ (desktop framework)
- Rust 2024 edition (all business logic)
- Polars 0.51 (DataFrame operations)
- lex-processing crate (preprocessing pipeline)

**Tauri Plugins:** `tauri-plugin-dialog`, `tauri-plugin-log`

---

## 4. Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│  FRONTEND (Next.js + React) - RENDERING ONLY                    │
│  invoke() ──────────────────────────────────────> listen()      │
└─────────────────────────────────────────────────────────────────┘
                              │ IPC │
┌─────────────────────────────────────────────────────────────────┐
│  RUST BACKEND (src-tauri + lex-processing)                      │
│  AppState (RwLock): dataframe, processed_dataframe, ui_state,   │
│                     preprocessing_history, ai_config, theme     │
│  ALL: business logic, data processing, validation, I/O          │
└─────────────────────────────────────────────────────────────────┘
```

### Current State

**Completed:** Foundation, Data Grid, File Loading, Preprocessing Integration, Settings

**Capabilities:** CSV loading with virtual scroll, configurable preprocessing, AI-assisted decisions (OpenRouter/Gemini), processed data viewing, export to CSV, theme settings

**Next:** Analytics, Visualizations, ML, and Python sidecar

---

## 5. Desktop UX Rules

| Do | Don't |
|----|-------|
| Dense layouts, minimal whitespace | Hero sections, large padding |
| Native dialogs, context menus | Custom modals for file access |
| Status bar feedback | Toast-only feedback |
| Virtual scrolling | Infinite scroll |
| Muted color scheme | Colorful badges (green/red/yellow) |
| Resizable panels | Fixed card layouts |
| Icons only when necessary | Excessive icons everywhere |

**Reference:** VS Code, Excel, DaVinci Resolve

**UI Guidelines:**
- Use muted colors (`text-muted-foreground`, `bg-muted`) - avoid bright/colorful indicators
- Icons should be functional, not decorative - don't add icons just for aesthetics
- Prefer text labels over icon-only buttons when space allows

**Risks & Mitigations:**
- Next.js SSR in Tauri → `"use client"` on ALL components, static export only
- Large dataset memory → Polars lazy evaluation, virtual scrolling
- AI provider offline → Rule-based fallback, clear offline indicators

---

## 6. Common Mistakes to Avoid

- Adding logic in TypeScript (calculations, filtering, validation) - **use Rust**
- Creating new UI components without checking `components/ui/` first
- Creating page-specific components when a reusable one already exists in `components/ui/` or `components/layout/`
- Using colorful styling (green/red/yellow badges) - **use muted colors**
- Forgetting `"use client"` directive on components
- Using `unwrap()` in Rust production code - **use proper error handling**
- Modifying `crates/lex-processing/` without asking permission
- Adding excessive icons for decoration
- Using inline SVGs instead of Lucide React

---

## 7. Development Patterns

When adding new functionality, reference existing implementations:

### Tauri Commands
1. Create/edit module in `src-tauri/src/commands/`
2. Add `#[tauri::command]` function
3. Export from `commands/mod.rs`
4. Register in `lib.rs` invoke_handler

**Reference:** `commands/settings.rs` (simple), `commands/preprocessing.rs` (complex)

### Events (Rust → Frontend)
1. Add constant + payload in `src-tauri/src/events.rs`
2. Add method to `AppEventEmitter` trait + implement for `AppHandle`

**Reference:** `events.rs` preprocessing events section

### Frontend Components

> **CRITICAL: Component Reuse is MANDATORY.** Before creating ANY component, you MUST verify it doesn't already exist. This is a top priority when working on the frontend.

**Search the `components/` directory thoroughly:**
- `components/ui/` - Base/reusable primitives (Button, Input, Select, Tabs, etc.)
- `components/layout/` - App-wide layout components (AppShell, ContextSidebar, NavSidebar, Toolbar, StatusBar)
- `components/<page>/` - Page-specific components (home/, data-grid/, preprocessing/, settings/)

```
components/
├── ui/              # Base primitives - used across multiple pages
├── layout/          # App-wide layout structure
├── home/            # Home page-specific
├── data-grid/       # Data page-specific
├── preprocessing/   # Processing page-specific
└── settings/        # Settings page-specific
```

#### Decision Tree for New Components

1. **Does the component already exist?**
   - YES → **Use the existing component.** Do NOT create a duplicate.
   - NO → Continue to step 2.

2. **Is this a base/reusable component?** (buttons, inputs, cards, panels, modals, etc.)
   - YES → Create in `components/ui/`
   - NO → Continue to step 3.

3. **Is this a page-specific component?** (only used in one page)
   - YES → Create in `components/<page-name>/` (e.g., `components/home/`)
   - NO → If it's layout-related, create in `components/layout/`

4. **Does a similar component exist in another page's directory?**
   - YES → **Move it to `components/ui/`**, update imports in the original page, then use it.
   - NO → Create new component following the rules above.

#### Examples

| Scenario | Correct Action |
|----------|---------------|
| Need a sidebar for any page | Use `ContextSidebar` from `components/layout/` - do NOT create a custom `<aside>` with fixed width |
| Need a button with loading state | Use `Button` from `components/ui/button` - do NOT create a new button component |
| Need a file info card for home page, similar one exists in data page | Move the component to `components/ui/`, refactor data page imports, then use it |
| Need a specialized config panel only for preprocessing | Create in `components/preprocessing/` |

#### Why This Matters

- **DRY (Don't Repeat Yourself)** - One source of truth for each component
- **Consistency** - Same component behaves the same everywhere
- **Maintainability** - Bug fixes and improvements apply globally
- **Code Quality** - Smaller codebase, easier to understand

#### Usage Pattern

1. Use `"use client"` directive on all components
2. Use `invoke()` for Tauri commands, `useRustEvent()` for events
3. Use `cn()` from `@/lib/utils` for conditional class names

**Reference:** `components/layout/context-sidebar.tsx` (resizable sidebar), `components/ui/button.tsx` (base primitive)

### Hooks
**Reference:** `lib/hooks/use-settings.ts` (simple), `lib/hooks/use-preprocessing.ts` (complex)

---

## 8. lex-processing Library

Automated data preprocessing: profiling, quality analysis, AI/rule-based decisions, imputation, outlier handling.

**Full API documentation:** `crates/lex-processing/AGENTS.md`

**IMPORTANT:** You may request edits to the lex-processing crate, but you **MUST ASK FOR PERMISSION** before making any changes to files in `crates/lex-processing/`.

---

## 9. Reference Locations

| What | Where |
|------|-------|
| All commands | `src-tauri/src/lib.rs` invoke_handler |
| All events | `src-tauri/src/events.rs` (constants at top) |
| Error codes | `src-tauri/src/events.rs` error_codes module |
| TypeScript types | `types/index.ts` |
| AppState structure | `src-tauri/src/state.rs` |
