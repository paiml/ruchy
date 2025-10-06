# WASM F-String (String Interpolation) Support Specification

## Five Whys Root Cause Analysis

1. **Why do 14/17 LANG-COMP tests fail?** → Example files contain f-strings like `println(f"Value: {x}")`
2. **Why do f-strings fail in WASM?** → `StringInterpolation` expressions not handled in WASM backend
3. **Why weren't f-strings implemented in WASM?** → Initial WASM work focused on basic expressions
4. **Why was this not caught earlier?** → 15-tool validation exposed the gap
5. **ROOT CAUSE**: Incremental WASM development without comprehensive feature coverage validation

## Defect Classification

**DEFECT TYPE**: Missing language feature in WASM backend
**SEVERITY**: HIGH - Blocks 14/17 (82%) of LANG-COMP tests
**IMPACT**: Users cannot use f-strings when compiling to WASM
**TOYOTA WAY RESPONSE**: STOP THE LINE - implement immediately, no deferrals

## Technical Analysis

### Current State

**Transpiler Implementation** (src/backend/transpiler/expressions.rs:68):
```rust
pub fn transpile_string_interpolation(&self, parts: &[StringPart]) -> Result<TokenStream> {
    let mut format_string = String::new();
    let mut args = Vec::new();
    for part in parts {
        match part {
            StringPart::Text(s) => format_string.push_str(&s.replace('{', "{{").replace('}', "}}")),
            StringPart::Expr(expr) => {
                format_string.push_str("{}");
                args.push(self.transpile_expr(expr)?);
            }
            StringPart::ExprWithFormat { expr, format_spec } => {
                format_string.push('{');
                format_string.push_str(format_spec);
                format_string.push('}');
                args.push(self.transpile_expr(expr)?);
            }
        }
    }
    Ok(quote! { format!(#format_string #(, #args)*) })
}
```

**Transpiler generates**: `format!("Value: {}", x)` → Works for Rust compilation

**WASM Current State**: NO handling of `ExprKind::StringInterpolation` → Compilation fails

### String Interpolation AST Structure

```rust
pub enum StringPart {
    Text(String),           // Literal text: "Hello "
    Expr(Box<Expr>),        // Expression: {name}
    ExprWithFormat {        // Formatted: {value:.2}
        expr: Box<Expr>,
        format_spec: String,
    },
}
```

### WASM String Representation

WASM has NO native string type. Strings must be:
1. Stored in linear memory (byte array)
2. Passed as `(ptr, len)` pairs (i32, i32)
3. Manipulated via host functions

### Implementation Strategy

F-strings must be **desugared into string concatenation** at WASM lowering time:

```
f"Value: {x}"
→ "Value: " + x.to_string()
→ WASM: (call $string_concat (i32.const ptr_value) (call $to_string (local.get $x)))
```

## Implementation Plan (EXTREME TDD)

### Phase 1: String Concatenation Support (RED→GREEN→REFACTOR)

**RED**: Test that fails
```rust
#[test]
fn test_wasm_string_concatenation() {
    let mut parser = Parser::new(r#"let x = "Hello" + "World""#);
    let ast = parser.parse().unwrap();
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).unwrap();
    assert!(wasmparser::validate(&wasm_bytes).is_ok());
}
```

**GREEN**: Implement string concatenation in WASM
- Add `string_concat` import from env
- Lower `BinaryOp::Add` with string operands to concatenation call

**REFACTOR**: Ensure complexity ≤10, A- grade

### Phase 2: to_string() Support (RED→GREEN→REFACTOR)

**RED**: Test that fails
```rust
#[test]
fn test_wasm_integer_to_string() {
    let mut parser = Parser::new(r#"let x = 10; "Value: " + x.to_string()"#);
    let ast = parser.parse().unwrap();
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).unwrap();
    assert!(wasmparser::validate(&wasm_bytes).is_ok());
}
```

**GREEN**: Implement to_string() method calls
- Add `i32_to_string` import from env
- Lower `MethodCall { method: "to_string", .. }` to appropriate conversion

**REFACTOR**: Ensure complexity ≤10, A- grade

### Phase 3: F-String Desugaring (RED→GREEN→REFACTOR)

**RED**: Test that fails
```rust
#[test]
fn test_wasm_fstring_simple() {
    let mut parser = Parser::new(r#"let x = 10; println(f"Value: {x}")"#);
    let ast = parser.parse().unwrap();
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).unwrap();
    assert!(wasmparser::validate(&wasm_bytes).is_ok());
}
```

**GREEN**: Implement `lower_string_interpolation()`
```rust
fn lower_string_interpolation(&self, parts: &[StringPart]) -> Result<Vec<Instruction>, String> {
    // Desugar f"Hello {name}" → "Hello " + name.to_string()
    // Build up string concatenations left-to-right
}
```

**REFACTOR**: Ensure complexity ≤10, A- grade

### Phase 4: Complex F-Strings (RED→GREEN→REFACTOR)

**RED**: Test with multiple expressions
```rust
#[test]
fn test_wasm_fstring_multiple_expressions() {
    let code = r#"let a = 10; let b = 3; println(f"Result: {a} + {b} = {a + b}")"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).unwrap();
    assert!(wasmparser::validate(&wasm_bytes).is_ok());
}
```

**GREEN**: Handle multiple concatenations

**REFACTOR**: Ensure complexity ≤10, A- grade

## Acceptance Criteria

✅ **Phase 1**: String concatenation compiles to WASM
✅ **Phase 2**: Integer/float .to_string() works in WASM
✅ **Phase 3**: Simple f-strings (single expression) compile to WASM
✅ **Phase 4**: Complex f-strings (multiple expressions) compile to WASM
✅ **LANG-COMP**: All 17 LANG-COMP tests pass with 15-tool validation
✅ **PMAT TDG**: A- minimum (≥85 points)
✅ **Complexity**: ≤10 per function
✅ **Mutation Coverage**: ≥75% for new code

## Host Function Requirements

WASM requires these host functions (imported from `env`):

```wat
(import "env" "println" (func $println (param i32)))
(import "env" "string_concat" (func $string_concat (param i32 i32) (result i32)))
(import "env" "i32_to_string" (func $i32_to_string (param i32) (result i32)))
(import "env" "f64_to_string" (func $f64_to_string (param f64) (result i32)))
```

**Note**: Actual string data representation (ptr/len pairs) may require more sophisticated encoding.

## Quality Gates

- **PMAT TDG**: `pmat tdg src/backend/wasm/mod.rs --min-grade A-`
- **Complexity**: `pmat analyze complexity src/backend/wasm/mod.rs --max-cyclomatic 10`
- **SATD**: `pmat analyze satd src/backend/wasm/mod.rs --fail-on-violation`
- **Tests**: `cargo test wasm_fstring --lib`
- **15-Tool Validation**: `cargo test fifteen_tool_validation -- --test-threads=1`

## Timeline

- **Phase 1**: 30 minutes (string concatenation)
- **Phase 2**: 30 minutes (to_string support)
- **Phase 3**: 45 minutes (f-string desugaring)
- **Phase 4**: 30 minutes (complex f-strings)
- **Total**: ~2.5 hours

## Success Metrics

- ✅ 17/17 LANG-COMP tests passing
- ✅ Zero WASM validation errors on f-string examples
- ✅ A- TDG score maintained
- ✅ ≤10 complexity per function
- ✅ Documented with working examples
