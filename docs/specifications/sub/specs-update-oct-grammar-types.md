# Sub-spec: Specifications Update Oct — Grammar Definition & Type System

**Parent:** [SPECIFICATIONS-UPDATE-OCT.md](../SPECIFICATIONS-UPDATE-OCT.md) Sections 2-3

---
## 2. Grammar Definition

### 2.1 Formal Grammar (EBNF)

```ebnf
program         = item*
item            = function | struct_def | enum_def | trait_def 
                | impl_block | actor_def | import_stmt | type_alias

// Functions
function        = 'fun' identifier generic_params? '(' params? ')' 
                  return_type? where_clause? (block | '=' expr)
params          = param (',' param)*
param           = identifier ':' type default_value?
default_value   = '=' expr
return_type     = '->' type

// Expressions
expr            = assignment
assignment      = pipeline ('=' assignment)?
pipeline        = logical_or ('>>' pipeline)*    // Note: >> not |>
logical_or      = logical_and ('||' logical_and)*
logical_and     = equality ('&&' equality)*
equality        = comparison (('==' | '!=') comparison)*
comparison      = term (('<' | '<=' | '>' | '>=') term)*
term            = factor (('+' | '-') factor)*
factor          = unary (('*' | '/' | '%' | '**') unary)*
unary           = ('!' | '-' | 'await')? postfix
postfix         = primary suffix*
suffix          = '.' identifier | '[' expr ']' | '(' args? ')' | '?'

primary         = literal | identifier | '(' expr ')' | if_expr 
                | match_expr | for_expr | while_expr | loop_expr
                | lambda | array_expr | tuple_expr | try_expr
                | actor_send | actor_ask | string_interp

// Lambda - single canonical form
lambda          = '|' params? '|' ('->' type)? (expr | block)

// Try-catch
try_expr        = 'try' block catch_clause+ finally_clause?
catch_clause    = 'catch' pattern ('if' expr)? block
finally_clause  = 'finally' block

// Pattern matching
match_expr      = 'match' expr '{' match_arm (',' match_arm)* '}'
match_arm       = pattern ('if' expr)? '=>' expr

// Actor operations
actor_send      = expr '<-' expr    // Fire and forget
actor_ask       = expr '<?' expr    // Request-reply

// String interpolation
string_interp   = 'f"' (text | '{' expr '}')* '"'

// Patterns
pattern         = literal | identifier | '_' | tuple_pattern 
                | array_pattern | struct_pattern | enum_pattern

// Types
type            = primitive | array_type | tuple_type | function_type
                | generic_type | reference_type
primitive       = 'i8' | 'i16' | 'i32' | 'i64' | 'i128'
                | 'u8' | 'u16' | 'u32' | 'u64' | 'u128'
                | 'f32' | 'f64' | 'bool' | 'char' | 'String'
array_type      = '[' type ']'
function_type   = 'fun' '(' types? ')' '->' type
reference_type  = '&' 'mut'? type
```

### 2.2 Keywords

```
fun let var const if else match for while loop break continue
return struct enum trait impl actor receive send async await
try catch finally throw import export module pub mut
type alias where in is as true false null
df col mean std quantile filter groupby agg sort select
```

### 2.3 Operator Precedence

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 | `.` `?.` | Left |
| 2 | `()` `[]` | Left |
| 3 | `!` `-` (unary) `await` | Right |
| 4 | `**` | Right |
| 5 | `*` `/` `%` | Left |
| 6 | `+` `-` | Left |
| 7 | `<<` `>>` (shift) | Left |
| 8 | `<` `<=` `>` `>=` | Left |
| 9 | `==` `!=` | Left |
| 10 | `&&` | Left |
| 11 | `\|\|` | Left |
| 12 | `>>` (pipeline) | Left |
| 13 | `=` `+=` `-=` | Right |

## 3. Type System

### 3.1 Type Categories

```rust
// Primitive types
i8, i16, i32, i64, i128
u8, u16, u32, u64, u128
f32, f64
bool, char, String, ()

// Collection types (default to Polars)
[T]                  // → Series
[[T]]               // → DataFrame
Vec<T>              // Explicit Vec only
HashMap<K,V>        // Explicit HashMap only

// Composite types
(T1, T2, ...)       // Tuples
Option<T>           // Nullable
Result<T, E>        // Error handling

// Mathematical types
DataFrame           // Polars DataFrame
LazyFrame          // Lazy evaluation
Series             // Column data
Matrix<T>          // nalgebra
```

### 3.2 Type Inference

Bidirectional type checking with Hindley-Milner inference:

```rust
impl TypeChecker {
    fn infer(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Lambda { params, body, .. } => {
                let param_types = params.iter()
                    .map(|p| self.fresh_type_var())
                    .collect();
                let body_type = self.infer(body);
                Type::Function(param_types, Box::new(body_type))
            }
            Expr::Pipeline { left, right } => {
                // x >> f infers as f(x)
                let left_type = self.infer(left);
                let func_type = self.infer(right);
                self.apply_function(func_type, left_type)
            }
            _ => self.infer_standard(expr),
        }
    }
}
```

### 3.3 Collection Type Mapping

Arrays and array literals default to Polars types:

```rust
[1, 2, 3]           // → Series::new("", &[1, 2, 3])
[[1, 2], [3, 4]]    // → df!["c0" => [1,3], "c1" => [2,4]]

// Explicit Rust collections require type annotation
let v: Vec<i32> = vec![1, 2, 3];
```

