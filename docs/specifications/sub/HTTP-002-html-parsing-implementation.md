# Sub-spec: HTTP-002 — HTML Parsing, Implementation Plan, and Testing

**Parent:** [HTTP-002-advanced-http-features.md](../HTTP-002-advanced-http-features.md) Sections C-End

---

### HTTP-002-C: Native HTML Parsing (Issue #43)

**Priority**: MEDIUM
**Complexity**: Medium-High
**GitHub Issue**: https://github.com/paiml/ruchy/issues/43
**Rationale**: Native HTML parsing stdlib to replace unmaintained `scraper` crate dependencies

#### Problem Statement

Downstream consumers (like paiml-mcp-agent-toolkit) need HTML parsing for E2E tests but want to avoid unmaintained dependencies. The `scraper` crate brings in `fxhash` (unmaintained) via `selectors`.

**Current Workaround** (from paiml-mcp-agent-toolkit):
```rust
// OLD (with scraper):
let document = Html::parse_document(&html_content);
let selector = Selector::parse(".stat-card").unwrap();
let stat_cards: Vec<_> = document.select(&selector).collect();

// NEW (workaround - loses functionality):
let stat_card_count = html_content.matches("stat-card").count();
```

#### Proposed Solution: Native Html Stdlib Type

Add `Html` type to Ruchy's standard library with Ruby/JavaScript-style API:

```ruby
# Ruby-style HTML parsing
html = Html.parse(content)
stat_cards = html.select(".stat-card")
puts "Found #{stat_cards.length} cards"

# Or querySelector-style (JavaScript compatibility)
element = html.query_selector("#main")
elements = html.query_selector_all(".item")

# Element access
element.text()          # Get text content
element.attr("href")    # Get attribute
element.html()          # Get inner HTML
element.parent()        # Get parent element
element.children()      # Get child elements

# Practical example
html = Html.parse(File.read("page.html"))
links = html.select("a[href]")
links.each do |link|
  puts "#{link.text()}: #{link.attr('href')}"
end
```

#### Implementation Details

**File**: `src/stdlib/html.rs` (new module)

```rust
use scraper::{Html as ScraperHtml, Selector, ElementRef};
use std::sync::Arc;

/// HTML document type for parsing and querying
#[derive(Clone)]
pub struct HtmlDocument {
    doc: Arc<ScraperHtml>,
}

/// HTML element wrapper
#[derive(Clone)]
pub struct HtmlElement {
    element: ElementRef<'static>, // Needs lifetime handling
}

impl HtmlDocument {
    /// Parse HTML from string
    pub fn parse(content: &str) -> Self {
        Self {
            doc: Arc::new(ScraperHtml::parse_document(content)),
        }
    }

    /// Query selector (returns all matches)
    pub fn select(&self, selector: &str) -> Result<Vec<HtmlElement>, String> {
        let sel = Selector::parse(selector)
            .map_err(|e| format!("Invalid selector: {}", e))?;

        Ok(self.doc
            .select(&sel)
            .map(|e| HtmlElement { element: e })
            .collect())
    }

    /// Query selector (returns first match)
    pub fn query_selector(&self, selector: &str) -> Result<Option<HtmlElement>, String> {
        let sel = Selector::parse(selector)
            .map_err(|e| format!("Invalid selector: {}", e))?;

        Ok(self.doc.select(&sel).next().map(|e| HtmlElement { element: e }))
    }

    /// Query selector all (alias for select)
    pub fn query_selector_all(&self, selector: &str) -> Result<Vec<HtmlElement>, String> {
        self.select(selector)
    }
}

impl HtmlElement {
    pub fn text(&self) -> String {
        self.element.text().collect()
    }

    pub fn attr(&self, name: &str) -> Option<String> {
        self.element.value().attr(name).map(String::from)
    }

    pub fn html(&self) -> String {
        self.element.html()
    }

    pub fn parent(&self) -> Option<HtmlElement> {
        // Implementation needed
        None
    }

    pub fn children(&self) -> Vec<HtmlElement> {
        // Implementation needed
        vec![]
    }
}
```

**Integration with Runtime** (`src/runtime/eval_builtin.rs`):

```rust
// Add Html.parse() builtin
"Html" => match method_name {
    "parse" => {
        let html_string = require_string_arg(args, 0, "Html.parse")?;
        let doc = HtmlDocument::parse(&html_string);
        Ok(Value::Html(doc))
    }
    _ => Err(InterpreterError::UndefinedMethod(/* ... */)),
},

// Add methods to Value::Html
Value::Html(doc) => match method_name {
    "select" => {
        let selector = require_string_arg(args, 0, "html.select")?;
        let elements = doc.select(&selector)
            .map_err(|e| InterpreterError::RuntimeError(e))?;
        Ok(Value::Array(Arc::from(
            elements.into_iter()
                .map(Value::HtmlElement)
                .collect::<Vec<_>>()
        )))
    }
    "query_selector" => { /* ... */ }
    "query_selector_all" => { /* ... */ }
    _ => Err(InterpreterError::UndefinedMethod(/* ... */)),
},

// Add methods to Value::HtmlElement
Value::HtmlElement(elem) => match method_name {
    "text" => Ok(Value::from_string(elem.text())),
    "attr" => {
        let attr_name = require_string_arg(args, 0, "element.attr")?;
        Ok(elem.attr(&attr_name)
            .map(Value::from_string)
            .unwrap_or(Value::Nil))
    }
    "html" => Ok(Value::from_string(elem.html())),
    _ => Err(InterpreterError::UndefinedMethod(/* ... */)),
},
```

**Add to AST** (`src/frontend/ast.rs`):

```rust
pub enum Value {
    // ... existing variants ...
    Html(HtmlDocument),
    HtmlElement(HtmlElement),
}
```

#### Test Cases

**File**: `tests/stdlib_html.rs`

```ruby
# test_html_parse
html = Html.parse("<div class='test'>Hello</div>")
assert html != nil

# test_html_select
html = Html.parse("<div class='a'>1</div><div class='a'>2</div>")
elements = html.select(".a")
assert elements.length == 2

# test_html_text
html = Html.parse("<p>Hello World</p>")
p = html.query_selector("p")
assert p.text() == "Hello World"

# test_html_attr
html = Html.parse("<a href='http://example.com'>Link</a>")
link = html.query_selector("a")
assert link.attr("href") == "http://example.com"

# test_html_complex_selector
html = Html.parse("<div><ul><li class='item'>1</li><li class='item'>2</li></ul></div>")
items = html.select("div ul li.item")
assert items.length == 2
assert items[0].text() == "1"
assert items[1].text() == "2"
```

#### Benefits

1. **No Unmaintained Dependencies**: Uses well-maintained `scraper` crate as implementation detail only
2. **Ruby-Native Syntax**: Familiar API for Ruby developers
3. **Unified Ecosystem**: HTML parsing is part of Ruchy stdlib
4. **Better Maintenance**: Single maintained crate vs. multiple downstream dependencies
5. **Type Safety**: Integrated with Ruchy's type system

#### Migration Path

Downstream consumers can replace string matching with proper HTML parsing:

```ruby
# Before (workaround):
stat_card_count = html_content.matches("stat-card").count()

# After (proper parsing):
html = Html.parse(html_content)
stat_cards = html.select(".stat-card")
puts "Found #{stat_cards.length} cards"
```

## Implementation Plan

### Phase 1: PID File Management (HTTP-002-A)
**Estimated Time**: 2-3 hours
**Complexity**: ⭐⭐☆☆☆

1. Add `pid_file` option to `Serve` command
2. Implement `PidFile` struct with RAII cleanup
3. Add Unix process management (signal handling)
4. Write comprehensive unit tests
5. Test zsh compatibility
6. Document usage

**Acceptance Criteria**:
- ✅ `ruchy serve --pid-file` creates PID file
- ✅ Kills existing process if PID file exists and valid
- ✅ Cleans up PID file on graceful shutdown
- ✅ Works in zsh with compound commands
- ✅ All tests pass (unit + integration)

### Phase 2: Bench Command (HTTP-002-B)
**Estimated Time**: 4-6 hours
**Complexity**: ⭐⭐⭐☆☆

1. Design CLI interface (HTTP/WASM/CLI targets)
2. Implement HTTP benchmarking with `reqwest`
3. Implement statistical analysis (percentiles, mean, stddev)
4. Add ApacheBench-compatible output format
5. Implement WASM and CLI benchmarking
6. Write comprehensive tests
7. Add examples and documentation

**Acceptance Criteria**:
- ✅ `ruchy bench http` benchmarks HTTP endpoints
- ✅ Supports concurrency and request count
- ✅ Outputs ApacheBench-style statistics
- ✅ `ruchy bench wasm` benchmarks WASM modules
- ✅ `ruchy bench cli` benchmarks CLI commands
- ✅ All tests pass with property tests

### Phase 3: Native HTML Parsing (HTTP-002-C)
**Estimated Time**: 6-8 hours
**Complexity**: ⭐⭐⭐⭐☆
**GitHub Issue**: #43

1. Add `HtmlDocument` and `HtmlElement` types to stdlib
2. Integrate `scraper` crate as implementation detail
3. Add `Html.parse()` builtin to runtime
4. Implement `.select()`, `.query_selector()`, `.query_selector_all()` methods
5. Implement element methods: `.text()`, `.attr()`, `.html()`
6. Add lifetime handling for `ElementRef`
7. Write comprehensive tests (unit + integration)
8. Add property tests for selector parsing
9. Document usage with examples
10. Add to language completeness tracking

**Acceptance Criteria**:
- ✅ `Html.parse(content)` creates HTML document
- ✅ `.select(selector)` returns array of elements
- ✅ `.query_selector(selector)` returns first element or nil
- ✅ Element methods work correctly (.text(), .attr(), .html())
- ✅ Complex CSS selectors work (descendant, child, attribute, etc.)
- ✅ All tests pass with property tests
- ✅ No unmaintained dependencies exposed to users
- ✅ Replaces string matching workarounds in paiml-mcp-agent-toolkit

## Dependencies

### New Crates
```toml
[dependencies]
# For bench command
reqwest = { version = "0.12", features = ["json", "blocking"] }
tokio = { version = "1", features = ["full"] }

# For scraping (HTTP-002-C)
scraper = "0.20"
select = "0.6"

# Optional: JavaScript rendering
headless_chrome = { version = "1.0", optional = true }

[target.'cfg(unix)'.dependencies]
# For PID file management
nix = { version = "0.29", features = ["signal"] }

[dev-dependencies]
tempfile = "3.8"
wiremock = "0.6"  # For HTTP bench tests
```

## Testing Strategy

### Unit Tests
- PID file creation/cleanup
- Benchmark statistics calculation
- Scraping HTML parsing

### Integration Tests
- Server lifecycle with PID files
- End-to-end HTTP benchmarking
- Scraping real websites (with mock server)

### Property Tests
- Benchmark statistics are mathematically correct
- PID file management is race-condition free
- Scraper handles malformed HTML gracefully

### Manual Testing
```bash
# Test PID file management
ruchy serve dist --pid-file /tmp/ruchy.pid &
ps aux | grep ruchy
kill $(cat /tmp/ruchy.pid)

# Test benchmarking
ruchy bench http http://localhost:8080 --requests 100 --concurrency 10

# Test scraping
ruchy scrape https://example.com --selector "h1" --output data.json
```

## Documentation

### User Documentation
- `docs/HTTP-002-server-management.md`
- `docs/HTTP-002-benchmarking.md`
- `docs/HTTP-002-web-scraping.md`

### Examples
- `examples/http_bench_example.sh`
- `examples/web_scraper_example.sh`
- `examples/server_management.sh`

## Release Notes (v3.100.0)

```markdown
# v3.100.0 - Advanced HTTP Features

## New Features

### Server Process Management
- Added `--pid-file` option to `ruchy serve` command
- Automatic process management: kills existing server on port
- Fixes zsh background execution bug
- Clean PID file management with RAII

### Benchmarking Command
- New `ruchy bench` command for performance testing
- HTTP/HTTPS endpoint benchmarking (ApacheBench-compatible)
- WASM module performance testing
- CLI command benchmarking
- Comprehensive statistics and percentiles

### Web Scraping (if completed)
- New `ruchy scrape` command for web data extraction
- CSS selector support
- Optional JavaScript rendering
- Rate limiting and batch processing
- Multiple output formats

## Bug Fixes
- Fixed zsh compound command execution bug with `ruchy serve`

## Breaking Changes
- None
```

## Success Metrics

- ✅ Zero defects in production
- ✅ All quality gates pass (PMAT TDG A-, complexity ≤10)
- ✅ Test coverage ≥80%
- ✅ Property tests for all statistical calculations
- ✅ Mutation test coverage ≥75%
- ✅ Documentation complete with working examples
- ✅ Published to crates.io

## References

- [ApacheBench Documentation](https://httpd.apache.org/docs/2.4/programs/ab.html)
- [Scraper Crate Documentation](https://docs.rs/scraper/latest/scraper/)
- [Axum Framework](https://docs.rs/axum/latest/axum/)
