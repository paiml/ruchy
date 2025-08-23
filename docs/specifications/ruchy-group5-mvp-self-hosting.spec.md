## Language Syntax Specification

### Lambda Expression Duality
Ruchy supports two lambda syntaxes for ecosystem compatibility:

```ruchy
// Rust-style (pipes) - Preferred for systems code
let double = |x| x * 2
let sum = |x, y| x + y
[1, 2, 3].map(|x| x * 2)

// JavaScript-style (fat arrow) - Familiar for scripting
let double = x => x * 2  
let sum = (x, y) => x + y
[1, 2, 3].map(x => x * 2)

// Both compile to identical AST nodes
assert_eq!(parse("|x| x * 2"), parse("x => x * 2"))
```

### Syntax Grammar Extensions
```ebnf
lambda_expr = pipe_lambda | arrow_lambda

pipe_lambda = "|" param_list "|" expression
arrow_lambda = arrow_params "=>" expression

arrow_params = identifier | "(" param_list ")"
param_list = identifier ("," identifier)*
```

### Design Rationale
- **Pipes** signal Rust heritage, zero-cost abstractions
- **Fat arrows** lower cognitive friction for polyglot developers
- **No semantic difference** ensures refactoring safety
- **Parser complexity**: +15 LOC for 2x developer accessibility# Ruchy Self-Hosting Sprint: Minimal Viable Compiler

## Executive Summary

The path to self-hosting requires semantic correctness over feature breadth. Bootstrap demands only enough functionality to compile the compiler itself—approximately 2,000 lines of carefully chosen Ruchy code that exercises the essential compilation pipeline.

## Critical Path Analysis

Self-hosting bottleneck: semantic fidelity of core constructs, not feature completeness. The compiler must correctly handle a minimal subset with zero tolerance for incorrectness.

## Week 1: Semantic Foundation (40 hours)

### Day 1: Lambda Syntax Unification (DX-001)
**Critical**: Fat arrow syntax enables JavaScript-familiar ergonomics.

```rust
// src/frontend/lexer.rs - Add FatArrow token
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Arrow,        // -> (type annotations)
    FatArrow,     // => (lambda expressions)
    // ...
}

impl Lexer {
    fn next_token(&mut self) -> Result<Token> {
        '=' => {
            if self.peek() == Some('>') {
                self.advance();
                Ok(Token::FatArrow)
            } else if self.peek() == Some('=') {
                self.advance();
                Ok(Token::EqualEqual)
            } else {
                Ok(Token::Equal)
            }
        }
    }
}

// src/frontend/parser/expressions.rs - Dual syntax support
impl Parser {
    fn parse_lambda(&mut self) -> Result<Expr> {
        // Support both: |x| x * 2 and x => x * 2
        if self.check(&Token::Pipe) {
            self.parse_closure()
        } else if self.peek_fat_arrow_lambda() {
            let params = if self.check(&Token::LeftParen) {
                self.parse_parameter_list()?  // (x, y) => ...
            } else {
                vec![self.parse_identifier()?]  // x => ...
            };
            self.expect(Token::FatArrow)?;
            let body = Box::new(self.parse_expression()?);
            Ok(Expr::Lambda { params, body })
        } else {
            self.error("Expected lambda expression")
        }
    }
}
```

### Day 2: Parser AST Completeness
**Problem**: Parser accepts syntax but produces incomplete AST nodes.

```rust
// Fix in src/frontend/parser/mod.rs
impl Parser {
    // Current: parse_actor returns Expr::Placeholder
    // Required: Expr::Actor { name, handlers, state }
    fn parse_actor(&mut self) -> Result<Expr> {
        self.expect(Token::Actor)?;
        let name = self.parse_identifier()?;
        self.expect(Token::LeftBrace)?;
        
        let mut handlers = Vec::new();
        let mut state = None;
        
        while !self.check(&Token::RightBrace) {
            if self.match_token(&Token::State) {
                state = Some(self.parse_state_block()?);
            } else if self.match_token(&Token::On) {
                handlers.push(self.parse_message_handler()?);
            }
        }
        
        Ok(Expr::Actor { name, handlers, state })
    }
}
```

### Day 3-4: Type Inference Core
**Strategy**: Algorithm W, no effects, no row types.

```rust
// New: src/middleend/types/infer.rs
pub struct TypeInference {
    unifier: Unifier,
    env: TypeEnv,
}

impl TypeInference {
    pub fn infer(&mut self, expr: &Expr) -> Result<Type> {
        match expr {
            Expr::Literal(lit) => Ok(self.literal_type(lit)),
            
            Expr::Lambda { params, body } => {
                let param_types = params.iter()
                    .map(|_| self.fresh_var())
                    .collect::<Vec<_>>();
                
                let env_snapshot = self.env.clone();
                for (param, ty) in params.iter().zip(&param_types) {
                    self.env.bind(param, ty.clone());
                }
                
                let body_type = self.infer(body)?;
                self.env = env_snapshot;
                
                Ok(Type::Function(param_types, Box::new(body_type)))
            }
            
            Expr::Apply { func, args } => {
                let func_type = self.infer(func)?;
                let arg_types = args.iter()
                    .map(|arg| self.infer(arg))
                    .collect::<Result<Vec<_>>>()?;
                
                let ret_type = self.fresh_var();
                self.unify(
                    func_type,
                    Type::Function(arg_types, Box::new(ret_type.clone()))
                )?;
                Ok(ret_type)
            }
            
            _ => self.fallback_inference(expr)
        }
    }
}
```

### Day 5: Codegen Minimalism
**Focus**: Direct Rust mapping, no optimization.

```rust
// Simplify src/backend/transpiler/codegen.rs
impl CodeGen {
    fn gen_expr(&self, expr: &TypedExpr) -> String {
        match expr {
            TypedExpr::Let { name, value, .. } => {
                format!("let {} = {};", name, self.gen_expr(value))
            }
            
            TypedExpr::Function { name, params, body, ret_type } => {
                let params_str = params.iter()
                    .map(|(n, t)| format!("{}: {}", n, self.type_to_rust(t)))
                    .collect::<Vec<_>>()
                    .join(", ");
                    
                format!("fn {}({}) -> {} {{\n{}\n}}", 
                    name, params_str, 
                    self.type_to_rust(ret_type),
                    self.gen_expr(body))
            }
            
            TypedExpr::Match { expr, arms } => {
                let arms_str = arms.iter()
                    .map(|(pat, body)| {
                        format!("{} => {},", 
                            self.gen_pattern(pat), 
                            self.gen_expr(body))
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                    
                format!("match {} {{\n{}\n}}", 
                    self.gen_expr(expr), arms_str)
            }
            
            _ => panic!("Unimplemented: {:?}", expr)
        }
    }
}
```

## Week 2: Bootstrap Implementation (40 hours)

### Day 6-7: Minimal Standard Library
```ruchy
// stdlib/core.ruchy
module Core {
    // FFI to Rust via inline directives
    fn print(s: String) = @rust("println!(\"{}\", s)")
    fn format(fmt: String, args: List<Any>) = @rust("format!(\"{}\", args)")
    
    // Collections
    fn vec<T>() -> Vec<T> = @rust("Vec::new()")
    fn push<T>(v: &mut Vec<T>, item: T) = @rust("v.push(item)")
    
    // Error handling
    type Result<T, E> = Ok(T) | Err(E)
    fn unwrap<T, E>(r: Result<T, E>) -> T = 
        match r {
            Ok(v) => v,
            Err(e) => panic("unwrap failed: {}", e)
        }
}
```

### Day 8-9: Compiler in Ruchy
```ruchy
// src/compiler.ruchy - Using fat arrow syntax
module Compiler {
    use Core::*
    
    type Token = 
        | Ident(String)
        | Number(i64)
        | String(String)
        | LeftParen | RightParen
        | FatArrow | Pipe
        | Let | Fn | Match
        
    fn tokenize(input: String) -> Result<Vec<Token>, String> {
        let tokens = vec()
        let chars = input.chars()
        let i = 0
        
        while i < chars.len() {
            match chars[i] {
                ' ' | '\n' | '\t' => i += 1,
                '(' => { push(&mut tokens, LeftParen); i += 1 },
                ')' => { push(&mut tokens, RightParen); i += 1 },
                '=' if chars[i+1] == '>' => {
                    push(&mut tokens, FatArrow)
                    i += 2
                },
                '|' => { push(&mut tokens, Pipe); i += 1 },
                '"' => {
                    let (str, next_i) = read_string(chars, i + 1)?
                    push(&mut tokens, String(str))
                    i = next_i
                },
                c if c.is_alphabetic() => {
                    let (ident, next_i) = read_ident(chars, i)
                    push(&mut tokens, keyword_or_ident(ident))
                    i = next_i
                },
                _ => return Err(format("Unexpected char: {}", c))
            }
        }
        Ok(tokens)
    }
    
    fn parse(tokens: Vec<Token>) -> Result<Ast, String> {
        Parser::new(tokens).parse_program()
    }
    
    fn parse_lambda(tokens: &[Token], pos: usize) -> Result<(Expr, usize), String> {
        // Handle both syntaxes
        match tokens[pos] {
            Pipe => parse_pipe_lambda(tokens, pos),
            Ident(name) if tokens[pos+1] == FatArrow => {
                // x => body
                let body = parse_expr(tokens, pos + 2)?
                Ok((Lambda { params: vec![name], body }, body.1))
            },
            LeftParen if contains_fat_arrow(tokens, pos) => {
                // (x, y) => body
                let (params, next) = parse_params_until(tokens, pos, FatArrow)?
                let (body, final) = parse_expr(tokens, next)?
                Ok((Lambda { params, body }, final))
            },
            _ => Err("Expected lambda expression")
        }
    }
    
    fn compile(source: String) -> Result<String, String> {
        tokenize(source)
            .and_then(parse)
            .and_then(infer_types)
            .map(generate_rust)
    }
}
```

### Day 10: Bootstrap Harness
```rust
// bootstrap/stage0.rs
fn main() -> Result<(), Box<dyn Error>> {
    // Stage 0: Rust compiler compiles initial Ruchy compiler
    let ruchy_compiler_source = fs::read_to_string("src/compiler.ruchy")?;
    
    // Use existing Rust implementation
    let ast = ruchy::parse(&ruchy_compiler_source)?;
    let typed = ruchy::typecheck(&ast)?;
    let rust_code = ruchy::codegen(&typed)?;
    
    fs::write("target/stage1_compiler.rs", rust_code)?;
    
    // Compile stage1 compiler
    Command::new("rustc")
        .args(&["target/stage1_compiler.rs", "-o", "target/stage1"])
        .status()?;
    
    // Stage 1: Ruchy compiler compiles itself
    let output = Command::new("target/stage1")
        .arg("src/compiler.ruchy")
        .output()?;
    
    fs::write("target/stage2_compiler.rs", output.stdout)?;
    
    // Verify fixed point
    let stage2_output = Command::new("rustc")
        .args(&["target/stage2_compiler.rs", "-o", "target/stage2"])
        .status()?;
    
    // Binary comparison
    let stage1_binary = fs::read("target/stage1")?;
    let stage2_binary = fs::read("target/stage2")?;
    
    if stage1_binary == stage2_binary {
        println!("✓ Bootstrap successful: Fixed point achieved");
    } else {
        println!("✗ Bootstrap failed: Binaries differ");
        process::exit(1);
    }
    
    Ok(())
}
```

## Critical Success Factors

### Minimal Feature Set for Self-Hosting
```
REQUIRED:
✓ Functions with parameters
✓ Let bindings  
✓ Pattern matching (basic)
✓ Struct definitions
✓ Module system (basic)
✓ Type inference (Algorithm W)
✓ String/Int/Bool literals
✓ Vec and HashMap
✓ Lambda expressions (both |x| and x => syntax)
✓ Method calls on collections

DEFERRED:
- Actors (use channels)
- Effects system
- Async/await
- Property testing
- Refinement types
- JIT compilation
- MCP protocol
```

### Test-Driven Bootstrap
```bash
# Test harness for each phase
make test-lexer     # Tokenize compiler.ruchy
make test-parser    # Parse to AST
make test-types     # Type inference succeeds
make test-codegen   # Generate valid Rust
make test-compile   # Rust compiles
make test-bootstrap # Fixed point reached
```

### Escape Hatches
```ruchy
// Use @rust directive for missing features
fn complex_operation(x: i32) -> i32 = 
    @rust("x.wrapping_mul(2).saturating_add(1)")

// Direct FFI for stdlib gaps
fn file_read(path: String) -> Result<String, IoError> =
    @rust("std::fs::read_to_string(path)")
```

## Metrics & Validation

### Week 1 Exit Criteria
- [ ] Parse 100% of compiler.ruchy syntax
- [ ] Type inference for 80% of expressions
- [ ] Generate compilable Rust for 60% of constructs

### Week 2 Exit Criteria
- [ ] compiler.ruchy compiles with stage0
- [ ] stage1 produces valid Rust
- [ ] Binary equivalence (or semantic equivalence)

### Performance Targets
- Bootstrap time: < 30 seconds
- Compiler size: < 5MB
- Memory usage: < 100MB
- LOC ratio: 1:1.5 (Ruchy:Rust)

## Risk Mitigation Matrix

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Type inference complexity | High | High | Use explicit annotations |
| Codegen bugs | Medium | High | Generate verbose but correct Rust |
| Missing stdlib | Low | Medium | @rust escape hatch |
| Bootstrap divergence | Medium | Critical | Binary diff at each stage |

## Daily Standup Questions

1. What AST nodes are still incomplete?
2. What type inference cases fail?
3. What Rust output doesn't compile?
4. How close is the fixed point?

## Conclusion

Self-hosting isn't about feature completeness—it's about semantic fidelity for a minimal subset. The compiler only needs to understand enough Ruchy to compile itself. Every feature beyond that threshold can be added post-bootstrap, using the self-hosted compiler to compile enhanced versions of itself.

The critical insight: **Bootstrap first, optimize later.**