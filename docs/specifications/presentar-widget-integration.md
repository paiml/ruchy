# Sub-spec: Presentar First-Class Widget/UI Framework Integration

**Parent:** [trueno-aprender-stdlib-core-language-spec.md](trueno-aprender-stdlib-core-language-spec.md)
**Version:** 1.0.0
**Status:** DRAFT
**Date:** 2026-04-03

---

## 0. Prerequisites

- **Current**: presentar 0.3.1, **target**: 0.3.4 (NOT 0.3.5 -- 0.3.5 does not exist on crates.io)
- **presentar_bridge.rs**: 336 lines, mostly local `Color` helpers; ~10 lines of actual presentar re-exports behind feature gate
- **Reactive state** (`signal()`): unverified in presentar API -- may require custom reactive runtime
- **Widget syntax**: brace-delimited (e.g., `Column { ... }`) is canonical, matching Ruchy's parser

## 1. Overview

### 1.1 Current State

The presentar bridge (`src/stdlib/presentar_bridge.rs`) is a 336-line module (mostly local `Color` helpers; ~10 lines of presentar re-exports behind feature gate) exposing approximately 5% of the presentar public API. It provides:

| Component | Description |
|-----------|-------------|
| `Color` struct | RGBA color type with named constants and hex parsing |
| Re-exports | `presentar::widgets`, `presentar::layout`, `presentar::yaml` |
| Browser types | `BrowserRouter`, `RouteMatch`, `RouteMatcher` |
| Notebook types | `NotebookRuntime`, `Cell`, `CellGraph`, `CellId`, `CellOutput` |

The bridge is feature-gated behind `widgets` in `Cargo.toml` (`presentar = { version = "0.3.1", optional = true }`). Users cannot declare widget trees in Ruchy source; they must call Rust APIs directly through the bridge. No transpiler integration exists.

### 1.2 Target State

Presentar becomes a first-class UI framework within Ruchy:

1. **Declarative widget syntax** -- Ruchy source constructs widget trees that the transpiler lowers to `presentar::widgets::*` calls.
2. **Reactive state** -- `signal()` primitive for fine-grained reactivity without a virtual DOM.
3. **Brick verification** -- `@brick` decorator injects runtime assertions (accessibility, performance, contrast) that block rendering on failure (JIDOKA principle).
4. **Full widget coverage** -- 30+ widgets across 6 categories surfaced as Ruchy constructors.
5. **CLI tooling** -- `ruchy widget` subcommand for dev server, WASM build, and assertion inspection.

### 1.3 Presentar Dependency

Upgrade from `0.3.1` to `0.3.4`. The 0.3.4 release provides:

| Crate | Purpose |
|-------|---------|
| `presentar-core` | Brick trait, assertion engine, layout solver |
| `presentar-widgets` | 30+ widget implementations |
| `presentar-browser` | WASM renderer, Canvas2D/WebGL backends |
| `presentar-terminal` | TUI renderer via crossterm |
| `presentar-yaml` | YAML-to-widget-tree deserialization |
| `presentar-a11y` | WCAG 2.1 AA audit and enforcement |
| `presentar-charts` | Chart widget (line, bar, scatter, heatmap) |
| `presentar-notebook` | NotebookRuntime, Cell, CellGraph |

### 1.4 Success Criteria

1. A counter app written in Ruchy source transpiles, compiles, and runs in the browser.
2. `@brick` assertions block rendering when violated (JIDOKA verified by test).
3. All 30+ widgets are constructible from Ruchy without raw Rust escape hatches.
4. WCAG 2.1 AA audit passes for every widget in the test suite.
5. Dev server hot-reloads widget changes in under 500ms.

## 2. Declarative Widget Syntax

### 2.1 Design Principles

- **Constructor functions** -- Each widget is a Ruchy function that returns a widget node.
- **Builder pattern** -- Optional parameters use named arguments with smart defaults.
- **Composition** -- Children are passed as trailing block arguments.
- **No JSX** -- Widget trees are plain Ruchy function calls, not template syntax.

### 2.2 Transpiler Lowering

The transpiler recognizes widget constructor names and emits `presentar::widgets::*` builder calls:

| Ruchy Source | Transpiled Rust |
|-------------|-----------------|
| `Text("Hello")` | `presentar::widgets::Text::new("Hello".into())` |
| `Button("Click", on_click=handler)` | `presentar::widgets::Button::new("Click".into()).on_click(handler)` |
| `Column { ... }` | `presentar::widgets::Column::new().children(vec![...])` |
| `Row { ... }` | `presentar::widgets::Row::new().children(vec![...])` |
| `Chart(data, kind="line")` | `presentar::charts::Chart::line(&data)` |

The transpiler MUST NOT emit `unsafe` code. All widget construction uses safe builder APIs.

### 2.3 Counter App Example

> **Syntax note:** Brace-delimited form (`Column { ... }`) is canonical. Colon-indented form is syntactic sugar normalized by the parser.

```ruchy
import presentar

fun counter_app():
    let count = signal(0)

    Column {
        Text(f"Count: {count.get()}")
        Row {
            Button("-", on_click=fun(): count.set(count.get() - 1))
            Button("+", on_click=fun(): count.set(count.get() + 1))
        }
    }
```

Transpiles to:

```rust
use presentar::widgets::{Column, Row, Button, Text};
use presentar::state::Signal;

fn counter_app() -> impl Widget {
    let count = Signal::new(0);

    Column::new().children(vec![
        Box::new(Text::new(format!("Count: {}", count.get()))),
        Box::new(Row::new().children(vec![
            Box::new(Button::new("-".into()).on_click({
                let count = count.clone();
                move || count.set(count.get() - 1)
            })),
            Box::new(Button::new("+".into()).on_click({
                let count = count.clone();
                move || count.set(count.get() + 1)
            })),
        ])),
    ])
}
```

### 2.4 Builder Defaults

Every widget constructor has sensible defaults so minimal invocations work:

```ruchy
Text("Hello")                           # font_size=16, color=BLACK
Button("OK")                            # style=primary, disabled=false
Container:                              # padding=8, margin=0, border=none
    Text("inside")
```

## 3. Brick Verification Integration

### 3.1 Overview

Presentar's Brick architecture enforces runtime invariants on widgets. A `Brick` is a widget plus a set of `BrickAssertion` values that MUST hold before the widget renders. This specification wires Brick verification into Ruchy's `@brick` decorator syntax.

### 3.2 Assertion Types

| Assertion | Parameters | Description |
|-----------|------------|-------------|
| `TextVisible` | `min_font_size: f32` | Text must be at least N pixels |
| `ContrastRatio` | `min_ratio: f32` | WCAG contrast ratio (default 4.5:1 for AA) |
| `MaxLatencyMs` | `budget_ms: u64` | Render must complete within budget |
| `MinTapTarget` | `min_px: u32` | Interactive elements >= 44x44px |
| `MaxChildren` | `limit: usize` | Prevents unbounded widget trees |
| `AriaLabel` | `required: bool` | Accessibility label must be present |

### 3.3 @brick Decorator

The `@brick` decorator attaches assertions to a widget function. The transpiler injects assertion checks into the generated Rust code.

```ruchy
@brick(TextVisible(min_font_size=12), ContrastRatio(min_ratio=4.5))
fun accessible_label(text: str) -> Widget:
    Text(text, font_size=14, color="#333333", bg="#FFFFFF")
```

Transpiles to:

```rust
fn accessible_label(text: &str) -> BrickHouse<Text> {
    BrickHouse::new(
        Text::new(text.into())
            .font_size(14.0)
            .color(Color::from_hex("#333333"))
            .background(Color::from_hex("#FFFFFF")),
    )
    .assert(BrickAssertion::TextVisible { min_font_size: 12.0 })
    .assert(BrickAssertion::ContrastRatio { min_ratio: 4.5 })
}
```

### 3.4 JIDOKA Enforcement

Rendering is blocked if any assertion fails. This follows Toyota's JIDOKA principle -- stop the line on defect detection. The render loop calls `brick.verify()` before `brick.render()`:

```
verify() -> Result<(), Vec<BrickViolation>>
  OK  -> render() proceeds
  Err -> render() skipped, violations logged, dev overlay shows red border
```

In CI mode (`RUCHY_BRICK_STRICT=1`), assertion failures cause a non-zero exit code.

### 3.5 Performance Budgets

The `MaxLatencyMs` assertion enforces frame budgets:

| Target | Budget | Rationale |
|--------|--------|-----------|
| 60 fps | 16 ms | Default for interactive UIs |
| 30 fps | 33 ms | Acceptable for data dashboards |
| Custom | user-defined | Via `@brick(MaxLatencyMs(budget_ms=N))` |

When a widget exceeds its budget, the violation includes the measured duration and a suggested optimization (e.g., "reduce child count" or "defer chart rendering").

### 3.6 BrickHouse Composition

`BrickHouse` wraps a widget and its assertion set. Nesting `BrickHouse` values composes assertions -- child assertions propagate upward:

```ruchy
@brick(MaxLatencyMs(budget_ms=100))
fun dashboard():
    Column:
        accessible_label("Status")   # inherits 4.5 contrast + 12px min
        Chart(data, kind="bar")
```

The composed assertion set for `dashboard` is the union of its own assertions and all children's assertions.

## 4. Reactive State Management

### 4.1 signal() Primitive

> **Note:** Requires either `presentar::state::Signal` (unverified in 0.3.4 API) or a custom reactive runtime emitted by the transpiler. Verify before implementation.

`signal(initial_value)` creates a reactive cell. Reading triggers dependency tracking; writing triggers re-render of dependent widgets.

```ruchy
let name = signal("world")
Text(f"Hello, {name.get()}!")    # re-renders when name changes
```

### 4.2 API Surface

| Method | Signature | Description |
|--------|-----------|-------------|
| `signal(v)` | `fn signal<T>(v: T) -> Signal<T>` | Create reactive cell |
| `.get()` | `fn get(&self) -> T` | Read current value |
| `.set(v)` | `fn set(&self, v: T)` | Write new value, trigger re-render |
| `.update(f)` | `fn update(&self, f: Fn(T) -> T)` | Transform in-place |
| `derived(f)` | `fn derived<T>(f: Fn() -> T) -> Derived<T>` | Computed property |

### 4.3 Derived Signals

Derived signals recompute automatically when their dependencies change:

```ruchy
let first = signal("Ada")
let last = signal("Lovelace")
let full = derived(fun(): f"{first.get()} {last.get()}")

Text(full.get())   # "Ada Lovelace", updates when first or last change
```

### 4.4 Event Binding

Widgets expose event callbacks as named parameters:

| Event | Widgets | Callback Signature |
|-------|---------|-------------------|
| `on_click` | Button, Container, Row | `Fn()` |
| `on_change` | TextInput, Checkbox, Toggle | `Fn(value)` |
| `on_input` | TextInput | `Fn(str)` |
| `on_submit` | TextInput, Form | `Fn()` |
| `on_select` | Tabs, DataTable | `Fn(index)` |

### 4.5 Two-Way Binding

`TextInput` supports two-way binding to a signal:

```ruchy
let username = signal("")
TextInput(value=username, placeholder="Enter name")
Text(f"You typed: {username.get()}")
```

The transpiler emits both an `on_change` handler that calls `username.set()` and a value binding that reads `username.get()`.

## 5. Widget Coverage

### 5.1 Widget Catalog

| Category | Widget | Constructor | Key Parameters |
|----------|--------|-------------|----------------|
| **Basic** | Text | `Text(content)` | `font_size`, `color`, `bold`, `italic` |
| | Button | `Button(label)` | `on_click`, `style`, `disabled` |
| | Container | `Container { ... }` | `padding`, `margin`, `border`, `bg` |
| | Icon | `Icon(name)` | `size`, `color` |
| | Image | `Image(src)` | `width`, `height`, `alt` |
| | Link | `Link(text, href)` | `target`, `style` |
| **Input** | TextInput | `TextInput(value)` | `placeholder`, `on_change`, `on_input` |
| | Checkbox | `Checkbox(checked)` | `label`, `on_change` |
| | Toggle | `Toggle(on)` | `label`, `on_change` |
| | Slider | `Slider(value)` | `min`, `max`, `step`, `on_change` |
| | Select | `Select(options)` | `selected`, `on_change` |
| | Form | `Form { ... }` | `on_submit` |
| **Navigation** | Tabs | `Tabs(labels)` | `active`, `on_select` |
| | Breadcrumb | `Breadcrumb(items)` | `separator` |
| | NavBar | `NavBar { ... }` | `brand`, `links` |
| **Data** | Chart | `Chart(data)` | `kind`, `title`, `x_label`, `y_label` |
| | DataTable | `DataTable(rows)` | `columns`, `sortable`, `on_select` |
| | DataCard | `DataCard(title)` | `value`, `trend`, `icon` |
| | ModelCard | `ModelCard(model)` | auto-populated from aprender metadata |
| | Badge | `Badge(text)` | `color`, `variant` |
| | Tag | `Tag(text)` | `removable`, `on_remove` |
| **Layout** | Column | `Column { ... }` | `gap`, `align`, `justify` |
| | Row | `Row { ... }` | `gap`, `align`, `justify` |
| | Grid | `Grid { ... }` | `columns`, `rows`, `gap` |
| | Stack | `Stack { ... }` | `z_index` |
| | Spacer | `Spacer(size)` | `flex` |
| | Divider | `Divider()` | `orientation`, `thickness` |
| | ScrollView | `ScrollView { ... }` | `direction`, `max_height` |
| **Feedback** | ProgressBar | `ProgressBar(value)` | `max`, `label`, `color` |
| | Modal | `Modal(open) { ... }` | `title`, `on_close` |
| | Tooltip | `Tooltip(text) { ... }` | `position` |
| | Snackbar | `Snackbar(message)` | `duration`, `action` |
| | Spinner | `Spinner()` | `size`, `color` |

### 5.2 Chart Subtypes

The `Chart` widget supports multiple visualization modes via the `kind` parameter:

| Kind | Description |
|------|-------------|
| `"line"` | Line chart with optional interpolation |
| `"bar"` | Vertical bar chart |
| `"scatter"` | Scatter plot with optional trend line |
| `"heatmap"` | Color-coded matrix |
| `"pie"` | Pie or donut chart |
| `"sparkline"` | Inline mini chart (no axes) |

## 6. CLI Commands

### 6.1 Subcommand Structure

All widget tooling lives under `ruchy widget`:

```
ruchy widget <subcommand> [options]
```

### 6.2 Commands

| Command | Description | Key Flags |
|---------|-------------|-----------|
| `serve` | Start dev server with hot reload | `--port`, `--yaml`, `--open` |
| `build` | Compile widget app to deployable artifact | `--target wasm\|native`, `--release` |
| `test` | Run Brick assertions + WCAG audit | `--strict`, `--report json` |
| `inspect` | Print assertion tree for a widget function | `--format ascii\|json` |

### 6.3 Dev Server

```bash
timeout 60 ruchy widget serve app.ruchy --port 3000 --open
timeout 60 ruchy widget serve --yaml dashboard.yaml --port 3000
```

The dev server watches source files, recompiles on change (target: under 500ms incremental), and sends a WebSocket reload signal to the browser.

### 6.4 WASM Build

```bash
timeout 120 ruchy widget build app.ruchy --target wasm --release
```

Output: `dist/` directory containing `index.html`, `app.wasm`, `app.js` (glue), and `style.css`. The pipeline uses `wasm-bindgen` and `wasm-opt` for size optimization. Output is self-contained for static hosting.

### 6.5 Assertion Inspection

```bash
ruchy widget inspect app.ruchy --function dashboard
# dashboard()
#   MaxLatencyMs(budget_ms=100)
#   accessible_label("Status")  ->  TextVisible(12), ContrastRatio(4.5)
#   Chart(data, kind="bar")     ->  (no assertions)
```

## 7. Notebook Integration

### 7.1 Architecture

The presentar notebook crates (`NotebookRuntime`, `Cell`, `CellGraph`) are wired into Ruchy's REPL and notebook modes:

| Component | Provided By | Role |
|-----------|-------------|------|
| `NotebookRuntime` | `presentar-notebook` | Manages cell execution graph |
| `Cell` | `presentar-notebook` | Single executable unit |
| `CellGraph` | `presentar-notebook` | Dependency tracking between cells |
| `CellOutput` | `presentar-notebook` | Typed output (text, widget, chart, error) |

### 7.2 Cell Output as Widgets

When a cell expression evaluates to a widget type, the REPL renders it inline instead of printing `Debug` output:

```ruchy
# Cell 1: data preparation
let scores = [85, 92, 78, 95, 88]

# Cell 2: renders a chart widget below the cell
Chart(scores, kind="bar", title="Exam Scores")

# Cell 3: renders a data card
DataCard("Mean", value=mean(scores), trend="+3.2%")
```

### 7.3 ModelCard Auto-Population

When an aprender model object is passed to `ModelCard`, metadata is extracted automatically:

```ruchy
let model = LinearRegression()
model.fit(X_train, y_train)

ModelCard(model)
# Renders: name, algorithm, training date, feature count,
#          R-squared, MSE, feature importances
```

The transpiler detects `ModelCard` calls where the argument implements `aprender::ModelMetadata` and emits the metadata extraction code.

### 7.4 Reactive Cells

Cells connected via signals form a reactive graph. Updating a signal in Cell 1 re-executes dependent cells:

```ruchy
# Cell 1
let threshold = signal(0.5)
Slider(threshold, min=0.0, max=1.0, step=0.01)

# Cell 2 (depends on threshold)
let filtered = data.filter(fun(row): row.score > threshold.get())
DataTable(filtered)
```

## 8. Testing Requirements

### 8.1 Accessibility (WCAG 2.1 AA)

All widgets MUST pass WCAG 2.1 AA audit in CI. The `presentar-a11y` crate provides programmatic checks:

| Rule | Threshold | Enforcement |
|------|-----------|-------------|
| Color contrast (text) | 4.5:1 minimum | `ContrastRatio` assertion |
| Color contrast (large text) | 3.0:1 minimum | `ContrastRatio` assertion |
| Tap target size | 44x44 px minimum | `MinTapTarget` assertion |
| Aria labels | Required on interactive elements | `AriaLabel` assertion |
| Keyboard navigation | All interactive widgets focusable | Integration test |
| Screen reader | Widget tree serializes to accessible text | Integration test |

CI command:

```bash
timeout 120 ruchy widget test app.ruchy --strict
```

`--strict` mode treats warnings as errors. Exit code 0 means all checks pass.

### 8.2 Visual Regression

Snapshot tests via `insta` capture rendered widget output and compare against baselines. Snapshot diffs are reviewed in PR checks; new snapshots require explicit approval via `cargo insta review`.

### 8.3 Brick Budget Violations

Brick budget violations MUST fail the build in CI. All `@brick`-annotated functions are run through `verify()`. The JSON report includes per-widget timing, violation details, and suggested fixes:

```bash
timeout 120 ruchy widget test --strict --report json > brick_report.json
```

### 8.4 Property Tests

Layout engine invariants are verified via `proptest` with 10K cases. Key properties:

- **Column children fit**: Total child height never exceeds parent constraint.
- **Row no overflow**: Total child width never exceeds parent constraint.
- **Grid cell bounds**: No cell renders outside its allocated grid area.
- **Signal roundtrip**: `signal(v).get() == v` for all `v`.

### 8.5 Test Matrix

| Test Category | Count | Framework | CI Gate |
|---------------|-------|-----------|---------|
| Unit (widget constructors) | 30+ | `#[test]` | Blocking |
| Integration (transpile-compile-render) | 10+ | `assert_cmd` | Blocking |
| Accessibility (WCAG audit) | 6 rules | `presentar-a11y` | Blocking |
| Visual regression (snapshots) | 15+ | `insta` | Blocking |
| Brick assertions | per `@brick` fn | `verify()` | Blocking |
| Property (layout engine) | 10K cases | `proptest` | Blocking |
| Performance (frame budget) | per widget | `MaxLatencyMs` | Blocking |

### 8.6 Mutation Testing

Widget verification logic is a high-risk component. Run `cargo mutants --file src/stdlib/presentar_bridge.rs --timeout 300`. Minimum threshold: 75% CAUGHT/MISSED ratio for assertion verification logic.
