# Sidebar Redesign Plan

## Overview

Merge the left NavSidebar and right ContextSidebar into a **unified right sidebar** with two modes:

- **Expanded**: Horizontal nav row at top + dynamic page content below
- **Collapsed**: Vertical icon-only nav strip (like current NavSidebar, but on the right)

This design is inspired by VS Code's right-side sidebar layout.

---

## Design Specifications

### Expanded Sidebar Layout

```
┌───────────────────────────────────────────────────────────────────────────────┐
│ [Lex Machina]                                                      [≡]        │ ← Toolbar
├─────────────────────────────────────────────────────────┬─────────────────────┤
│                                                         │ 🏠 📊 ⚙️ 📈 🧠 ⚙️    │ ← Horizontal nav icons
│                                                         ├─────────────────────┤
│                                                         │                     │
│                    MAIN CONTENT AREA                    │   Page-specific     │
│                    (full width)                         │   sidebar content   │
│                                                         │   (passed as        │
│                                                         │   children)         │
│                                                         │                     │
├─────────────────────────────────────────────────────────┴─────────────────────┤
│                                STATUS BAR                                     │
└───────────────────────────────────────────────────────────────────────────────┘
```

### Collapsed Sidebar Layout

```
┌──────────────────────────────────────────────────────────────────────────┐
│ [Lex Machina]                                                      [≡]   │ ← Toolbar
├─────────────────────────────────────────────────────────────────────┬────┤
│                                                                     │ 🏠 │
│                                                                     │ 📊 │
│                         MAIN CONTENT AREA                           │ ⚙️ │ ← Vertical nav icons
│                         (nearly full width)                         │ 📈 │    (~56px width)
│                                                                     │ 🧠 │
│                                                                     │ ⚙️ │
├─────────────────────────────────────────────────────────────────────┴────┤
│                                STATUS BAR                                │
└──────────────────────────────────────────────────────────────────────────┘
```

### Opt-Out Pages (Settings)

Pages that opt-out of sidebar content get the collapsed vertical nav only (no expand capability):

```
┌──────────────────────────────────────────────────────────────────────────┐
│ [Lex Machina]                                                            │ ← Toolbar (no toggle button)
├─────────────────────────────────────────────────────────────────────┬────┤
│                                                                     │ 🏠 │
│                                                                     │ 📊 │
│                         PAGE CONTENT                                │ ⚙️ │
│                         (Settings grid, etc.)                       │ 📈 │
│                                                                     │ 🧠 │
│                                                                     │ ⚙️ │
├─────────────────────────────────────────────────────────────────────┴────┤
│                                STATUS BAR                                │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Component Architecture

### New/Modified Components

| Component    | Location                            | Purpose                                                    |
| ------------ | ----------------------------------- | ---------------------------------------------------------- |
| `Sidebar`    | `components/layout/sidebar.tsx`     | **NEW** - Unified sidebar with nav + content               |
| `SidebarNav` | `components/layout/sidebar-nav.tsx` | **NEW** - Nav icons (horizontal or vertical based on mode) |
| `AppShell`   | `components/layout/app-shell.tsx`   | **MODIFY** - Remove left sidebar, update layout            |
| `Toolbar`    | `components/layout/toolbar.tsx`     | **MODIFY** - Add toggle button, remove page-specific slots |

### Removed Components

| Component        | Reason                               |
| ---------------- | ------------------------------------ |
| `NavSidebar`     | Merged into `Sidebar` / `SidebarNav` |
| `ContextSidebar` | Replaced by `Sidebar`                |

### Component API Design

**Simple rule:** Pages pass sidebar content via the `sidebar` prop. The `Sidebar` component is internal to `AppShell`.

```tsx
// AppShell API
interface AppShellProps {
    children: ReactNode; // Main content area
    sidebar?: ReactNode | false; // Sidebar content, or false to opt-out
}
```

**Usage Examples:**

```tsx
// Page WITH sidebar content
// app/data/page.tsx
export default function DataPage() {
    return (
        <AppShell
            sidebar={
                <>
                    <DataToolbarActions /> {/* Action buttons at top */}
                    <FileInfoPanel />
                    <ColumnList />
                </>
            }
        >
            <DataGrid />
        </AppShell>
    );
}

// Page WITHOUT sidebar content (opt-out)
// app/settings/page.tsx
export default function SettingsPage() {
    return (
        <AppShell sidebar={false}>
            <SettingsGrid />
        </AppShell>
    );
}
```

**Behavior:**

| `sidebar` prop value       | Result                                           |
| -------------------------- | ------------------------------------------------ |
| `<Content />`              | Full sidebar (expandable), toggle button visible |
| `false`                    | Vertical nav only, toggle button hidden          |
| `undefined` (not provided) | Same as `false`                                  |

**Page-Specific Action Buttons:**

- Move from toolbar INTO sidebar content
- Each page places its action buttons at the top of its sidebar content
- This keeps the toolbar clean (just branding + toggle)

---

## State Management

### Rust Backend State

Add to `UIState` in `src-tauri/src/state.rs`:

```rust
pub struct UIState {
    pub sidebar_width: f32,           // Existing
    pub sidebar_collapsed: bool,      // NEW - collapsed state
    pub column_widths: Vec<f32>,      // Existing
    pub grid_scroll: GridScrollPosition, // Existing
}
```

Add to persisted settings:

- `sidebar_collapsed: bool` - Persisted across app restarts

### New Rust Commands

| Command                 | Purpose                                     |
| ----------------------- | ------------------------------------------- |
| `toggle_sidebar`        | Toggle collapsed state, persist to settings |
| `set_sidebar_collapsed` | Explicitly set collapsed state              |

### Frontend State (Rendering Cache Only)

**IMPORTANT (Rust Supremacy):** The frontend does NOT own state. It only:

1. Fetches state from Rust on mount
2. Calls Rust commands to request state changes
3. Caches Rust state for rendering

Create `SidebarContext` as a **rendering cache** (NOT source of truth):

```tsx
// lib/contexts/sidebar-context.tsx
interface SidebarContextValue {
    // Cached state from Rust (read-only for rendering)
    width: number;
    collapsed: boolean;
    isOptOut: boolean;
    isInitialized: boolean;

    // These call Rust commands - no TS logic
    requestToggle: () => Promise<void>; // invoke("toggle_sidebar")
    requestSetWidth: (w: number) => Promise<void>; // invoke("set_sidebar_width")
}
```

**State Flow (Rust Supremacy):**

```
User clicks toggle button
       │
       ▼
invoke("toggle_sidebar")  ← TypeScript calls Rust
       │
       ▼
Rust toggles AppState.ui_state.sidebar_collapsed
       │
       ▼
Rust persists via tauri-plugin-store (settings.json)
       │
       ▼
Rust returns new collapsed state (bool)
       │
       ▼
TypeScript updates cached state for re-render
```

**Benefits:**

- Rust is single source of truth (per AGENTS.md)
- TypeScript only renders based on Rust state
- No duplicate fetches from Rust on every page mount
- Clean separation: Rust = logic, TypeScript = rendering

---

## Navigation Configuration

Icons in order (left-to-right when horizontal, top-to-bottom when vertical):

| ID         | Label      | Icon        | Route         |
| ---------- | ---------- | ----------- | ------------- |
| home       | Home       | `Home`      | `/home`       |
| data       | Data       | `Table2`    | `/data`       |
| processing | Processing | `Cog`       | `/processing` |
| analysis   | Analysis   | `BarChart3` | `/analysis`   |
| ml         | ML         | `Brain`     | `/ml`         |
| settings   | Settings   | `Settings`  | `/settings`   |

**Active indicator:** Background highlight (`bg-primary text-primary-foreground`)

---

## Toolbar Changes

### Current Toolbar

```
┌────────────────────────────────────────────────────────┐
│ [Lex Machina]              [page-specific buttons]     │
└────────────────────────────────────────────────────────┘
```

### New Toolbar

```
┌────────────────────────────────────────────────────────┐
│ [Lex Machina]                              [≡ toggle]  │
└────────────────────────────────────────────────────────┘
```

- **Left:** "Lex Machina" branding (stays)
- **Right:** Sidebar toggle button (new)
- **Page-specific buttons:** Move INTO sidebar content (each page handles its own)

Toggle button:

- Icon: `PanelRight` or `Sidebar` from Lucide
- Hidden on opt-out pages
- Toggles between expanded/collapsed state

---

## Resize Handle Behavior

| State     | Resize Handle                         |
| --------- | ------------------------------------- |
| Expanded  | Visible on left edge, draggable       |
| Collapsed | **Hidden** (no content to resize)     |
| Opt-out   | **Hidden** (fixed vertical nav width) |

**Improvements over current:**

1. **Double-click to reset** - Double-click resize handle resets to default width (280px)
2. **Visual feedback** - Already has hover/active states, keep as-is
3. **Hidden when not applicable** - No resize handle for collapsed/opt-out states

---

## Settings Page: New UI Section

Add a new "UI" or "Appearance" section to the Settings page with:

### Navigation Bar Position Setting

| Option   | Description                                                                                 |
| -------- | ------------------------------------------------------------------------------------------- |
| `left`   | Traditional left sidebar (vertical icons, always visible)                                   |
| `right`  | Right sidebar (vertical icons, always visible)                                              |
| `merged` | Merged with right sidebar (horizontal when expanded, vertical when collapsed) - **DEFAULT** |

### Rust State

Add to settings persistence:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NavBarPosition {
    Left,
    Right,
    #[default]
    Merged,
}
```

Add to `AppState` or settings store:

- `nav_bar_position: NavBarPosition`

### UI Component

```tsx
// In Settings page, under new "UI" section
<Select
    label="Navigation Bar Position"
    value={navBarPosition}
    onChange={handleNavBarPositionChange}
    options={[
        { value: "left", label: "Left (always visible)" },
        { value: "right", label: "Right (always visible)" },
        { value: "merged", label: "Merged with sidebar" },
    ]}
/>
```

---

## Implementation Phases

### Phase 1: Rust State Foundation

**Files:** `src-tauri/src/state.rs`, `src-tauri/src/commands/ui.rs`, `src-tauri/src/commands/settings.rs`

**Persistence:** Use existing `tauri-plugin-store` mechanism (same as theme, sidebar_width, ai_provider). All settings stored in `settings.json` via the store plugin.

1. Add `sidebar_collapsed: bool` to `UIState` struct (default: `false`)
2. Add `NavBarPosition` enum to `state.rs`:
    ```rust
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum NavBarPosition {
        Left,
        Right,
        #[default]
        Merged,
    }
    ```
3. Add `nav_bar_position: RwLock<NavBarPosition>` to `AppState`
4. Add new store keys to `settings.rs::store_keys` module:
    - `SIDEBAR_COLLAPSED: &str = "sidebar_collapsed"`
    - `NAV_BAR_POSITION: &str = "nav_bar_position"`
5. Add persistence helpers in `settings.rs` (like existing `persist_sidebar_width`):
    - `persist_sidebar_collapsed(app, collapsed) -> Result<(), String>`
    - `persist_nav_bar_position(app, position) -> Result<(), String>`
6. Update `init_settings_from_store()` to restore:
    - `sidebar_collapsed` → `state.ui_state.write().sidebar_collapsed`
    - `nav_bar_position` → `state.nav_bar_position.write()`
7. Add Rust commands in `commands/ui.rs`:
    - `toggle_sidebar(app, state) -> bool` - Toggle collapsed, persist, return new state
    - `set_sidebar_collapsed(collapsed, app, state)` - Explicitly set collapsed state
8. Add Rust commands in `commands/settings.rs`:
    - `get_nav_bar_position(state) -> NavBarPosition`
    - `set_nav_bar_position(position, app, state) -> Result<(), String>`
9. Register new commands in `lib.rs` invoke_handler
10. Update TypeScript types in `types/index.ts`:
    - Add `sidebar_collapsed: boolean` to `UIState`
    - Add `NavBarPosition` type: `"left" | "right" | "merged"`

### Phase 2: Sidebar Context (Rendering Cache)

**Files:** `lib/contexts/sidebar-context.tsx` (new), `app/layout.tsx` (modify)

1. Create `SidebarContext` as rendering cache (NOT source of truth)
2. Fetch initial state from Rust via `invoke("get_ui_state")` on mount
3. Provide methods that call Rust commands (no TS logic)
4. Cache Rust responses for rendering
5. Export `SidebarProvider` and `useSidebar` hook
6. **Wrap app in `SidebarProvider` in `app/layout.tsx`** (single instance for all pages)

**Key constraint:** Context only caches and invokes. All logic in Rust.

### Phase 3: Core Sidebar Components

**Files:** `components/layout/sidebar.tsx` (new), `components/layout/sidebar-nav.tsx` (new)

1. Create `SidebarNav` component (handles both horizontal and vertical layouts)
2. Create `Sidebar` component (internal to AppShell, orchestrates nav + content + resize)
3. Implement expanded/collapsed modes
4. Implement resize handle with double-click reset
5. **Hide resize handle when collapsed** (no content to resize)
6. Handle active route highlighting

### Phase 4: AppShell & Toolbar Updates

**Files:** `components/layout/app-shell.tsx`, `components/layout/toolbar.tsx`

1. Update `AppShell` to use new `Sidebar` component
2. Remove left sidebar slot from `AppShell`
3. Add toggle button to `Toolbar`
4. Implement opt-out logic (hide toggle button when `sidebar={false}`)

### Phase 5: Page Migrations

**Files:** All pages in `app/*/page.tsx`

Migrate each page to new sidebar pattern:

| Page       | Has Sidebar Content? | Migration Notes                                            |
| ---------- | -------------------- | ---------------------------------------------------------- |
| Home       | Yes                  | Move `HomeSidebarContent` into sidebar prop                |
| Data       | Yes                  | Move data sidebar content into sidebar prop                |
| Processing | Yes                  | **Special:** Sidebar + 3-col main content (see note below) |
| Analysis   | Yes                  | Placeholder content for now                                |
| ML         | Yes                  | Placeholder content for now                                |
| Settings   | **No (opt-out)**     | Use `sidebar={false}`, vertical nav only                   |

**Processing Page Note:**
The Processing page currently has a custom 3-column layout. With the new sidebar:

- Sidebar contains: Column selector, row range selector (currently in left column)
- Main content: Config panel + Results panel (2-column layout, not 3)
- This simplifies Processing to match other pages while keeping its functionality

### Phase 6: Settings UI Section

**Files:** `app/settings/page.tsx`, `components/settings/nav-position-selector.tsx` (new)

1. Add "UI" section to Settings page
2. Create `NavPositionSelector` component
3. Wire up to Rust backend for persistence

### Phase 7: NavBarPosition Support (FUTURE ENHANCEMENT)

**Status:** Optional, implement after core functionality is stable.

**Files:** Multiple - depends on selected position

Implement the three navigation position modes:

- `left`: Render vertical nav on left side (similar to current)
- `right`: Render vertical nav on right side (always visible, no content)
- `merged`: Default behavior (what we're building in phases 1-6)

This phase adds significant complexity with conditional rendering throughout `AppShell`.
**Recommendation:** Ship with `merged` only first, add other modes based on user feedback.

### Phase 8: Cleanup & Polish

1. Remove old `NavSidebar` component
2. Remove old `ContextSidebar` component
3. Update barrel exports in `components/layout/index.tsx`
4. Run `pnpm lint` and `pnpm format`
5. Test all pages manually with `pnpm tauri dev`
6. Verify state persistence across page navigation and app restart

---

## File Changes Summary

### New Files

- `lib/contexts/sidebar-context.tsx` - Sidebar state context
- `components/layout/sidebar.tsx` - Unified sidebar component
- `components/layout/sidebar-nav.tsx` - Navigation icons component
- `components/settings/nav-position-selector.tsx` - Settings UI for nav position

### Modified Files

- `src-tauri/src/state.rs` - Add `sidebar_collapsed`, `NavBarPosition`
- `src-tauri/src/commands/ui.rs` - Add new commands
- `src-tauri/src/commands/settings.rs` - Persist new settings
- `src-tauri/src/lib.rs` - Register new commands
- `types/index.ts` - Add TypeScript types
- `app/layout.tsx` - Wrap app in `SidebarProvider`
- `components/layout/app-shell.tsx` - New layout structure, internal `Sidebar`
- `components/layout/toolbar.tsx` - Add toggle button
- `app/home/page.tsx` - Migrate to new sidebar pattern
- `app/data/page.tsx` - Migrate to new sidebar pattern
- `app/processing/page.tsx` - Add sidebar (keep 3-col main content)
- `app/analysis/page.tsx` - Migrate to new sidebar pattern
- `app/ml/page.tsx` - Migrate to new sidebar pattern
- `app/settings/page.tsx` - Opt-out + add UI section

### Deleted Files

- `components/layout/nav-sidebar.tsx` - Merged into new components
- `components/layout/context-sidebar.tsx` - Replaced by `Sidebar`

---

## Design Principles (from AGENTS.md)

### Rust Supremacy (CRITICAL)

**ALL state and logic lives in Rust. TypeScript is for rendering ONLY.**

| NEVER in TypeScript | ALWAYS in Rust                            |
| ------------------- | ----------------------------------------- |
| State management    | `AppState` with `RwLock`                  |
| Toggle logic        | `toggle_sidebar` command                  |
| Width calculations  | `set_sidebar_width` command               |
| Persistence         | `tauri-plugin-store` (existing mechanism) |
| Validation          | Clamp width to min/max in Rust            |

**TypeScript's only jobs:**

1. Call `invoke()` to request state changes
2. Render UI based on state received from Rust
3. Send user events (clicks, drags) to Rust

### Desktop-First UX

- Dense information display (like VS Code, not web dashboards)
- Muted colors (`bg-muted`, `text-muted-foreground`, `bg-background`)
- No colorful badges or decorations
- Functional icons only (Lucide React)
- Fixed/resizable panels, not responsive layouts

### Component Reuse (No Duplication)

- Single `Sidebar` component used by all pages
- `SidebarNav` reusable for both horizontal and vertical layouts
- No duplicate code between expanded/collapsed modes
- Check `components/ui/` before creating any new primitive

### Separation of Concerns

| Layer                  | Responsibility                       |
| ---------------------- | ------------------------------------ |
| Rust `state.rs`        | State definitions                    |
| Rust `commands/ui.rs`  | State mutations + persistence        |
| `SidebarContext`       | Fetch from Rust, cache for rendering |
| `Sidebar` component    | Layout composition                   |
| `SidebarNav` component | Navigation icon rendering            |
| Page components        | Provide children content only        |

### Maintainability

- Adding a new page: Just wrap content in `<Sidebar>` or use `sidebar={false}`
- Changing nav items: Single array in `SidebarNav`
- Changing layout: Modify `Sidebar` component only
- All state changes: Modify Rust commands only

---

## Testing Checklist

### Core Functionality

- [ ] Sidebar expands/collapses via toggle button
- [ ] Collapsed state shows vertical nav icons
- [ ] Expanded state shows horizontal nav + content
- [ ] Navigation works in both modes (routes to correct pages)
- [ ] Active nav item has background highlight

### State Persistence (Rust Supremacy)

- [ ] Collapsed state persists across page navigation
- [ ] Collapsed state persists across app restart
- [ ] Width persists across page navigation
- [ ] Width persists across app restart

### Resize Handle

- [ ] Resize handle visible only in expanded mode
- [ ] Resize handle hidden when collapsed
- [ ] Resize handle hidden on opt-out pages
- [ ] Drag to resize works correctly
- [ ] Double-click resets to default width (280px)

### Page-Specific

- [ ] Home page: sidebar content renders correctly
- [ ] Data page: sidebar content renders correctly
- [ ] Processing page: sidebar + 2-col main content layout works
- [ ] Analysis page: placeholder sidebar renders
- [ ] ML page: placeholder sidebar renders
- [ ] Settings page: vertical nav only, no toggle button

### Settings UI (Phase 6)

- [ ] UI section appears in Settings
- [ ] NavBarPosition selector displays three options
- [ ] Selection persists to Rust/settings.json

### Code Quality

- [ ] No console errors
- [ ] `pnpm lint` passes
- [ ] `pnpm format` passes
- [ ] `cargo clippy` passes (0 warnings)
- [ ] `cargo fmt --check` passes
