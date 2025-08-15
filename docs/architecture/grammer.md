# Ruchy Language Grammar Reference v1.0
## Complete EBNF Grammar and Feature Specification

### 1. Lexical Grammar

#### 1.1 Tokens
```ebnf
token           = keyword | identifier | literal | operator | delimiter
whitespace      = ' ' | '\t' | '\r' | '\n'
comment         = line_comment | block_comment
line_comment    = '//' (!'\n' any)*
block_comment   = '/*' (!'*/' any)* '*/'
```

#### 1.2 Identifiers
```ebnf
identifier      = ident_start ident_continue*
ident_start     = 'a'..'z' | 'A'..'Z' | '_'
ident_continue  = ident_start | '0'..'9'
raw_identifier  = '`' identifier '`'  // For reserved words
```

#### 1.3 Keywords
```ebnf
keyword = 'fun' | 'let' | 'var' | 'const' | 'if' | 'else' | 'when' | 'match'
        | 'for' | 'while' | 'loop' | 'break' | 'continue' | 'return'
        | 'struct' | 'enum' | 'trait' | 'impl' | 'actor' | 'receive'
        | 'async' | 'await' | 'defer' | 'guard' | 'try' | 'catch'
        | 'import' | 'export' | 'module' | 'pub' | 'priv' | 'mut'
        | 'in' | 'is' | 'as' | 'where' | 'type' | 'alias'
```

#### 1.4 Literals
```ebnf
literal         = number | string | char | boolean
number          = integer | float
integer         = decimal | hexadecimal | octal | binary
decimal         = digit+ type_suffix?
hexadecimal     = '0x' hex_digit+
octal           = '0o' oct_digit+
binary          = '0b' bin_digit+
float           = digit+ '.' digit+ exponent? type_suffix?
exponent        = ('e' | 'E') ('+' | '-')? digit+
type_suffix     = 'i8' | 'i16' | 'i32' | 'i64' | 'i128'
                | 'u8' | 'u16' | 'u32' | 'u64' | 'u128'
                | 'f32' | 'f64'

string          = '"' string_char* '"' | '"""' multiline_char* '"""'
string_char     = !'"' (escape_seq | any)
escape_seq      = '\' ('n' | 'r' | 't' | '\' | '"' | 'x' hex hex | 'u{' hex+ '}')
interpolation   = '\(' expr ')'

char            = "'" (escape_seq | !"'" any) "'"
boolean         = 'true' | 'false'
```

### 2. Syntactic Grammar

#### 2.1 Program Structure
```ebnf
program         = item*
item            = function | struct_def | enum_def | trait_def | impl_block
                | actor_def | module_def | import_stmt | type_alias

module_def      = 'module' identifier '{' item* '}'
import_stmt     = 'import' import_path ('as' identifier)?
import_path     = identifier ('::' identifier)*
                | string_literal  // URL imports
```

#### 2.2 Functions
```ebnf
function        = attributes? visibility? 'fun' identifier 
                  generic_params? '(' params? ')' return_type? 
                  where_clause? (block | '=' expr)

params          = param (',' param)*
param           = pattern ':' type default_value?
default_value   = '=' expr
return_type     = '->' type

generic_params  = '<' generic_param (',' generic_param)* '>'
generic_param   = identifier (':' bounds)?
bounds          = bound ('+' bound)*
where_clause    = 'where' where_pred (',' where_pred)*
```

#### 2.3 Types
```ebnf
type            = primitive_type | named_type | tuple_type | array_type
                | function_type | optional_type | result_type
                | reference_type | generic_type | refined_type

primitive_type  = 'i8' | 'i16' | 'i32' | 'i64' | 'i128'
                | 'u8' | 'u16' | 'u32' | 'u64' | 'u128'
                | 'f32' | 'f64' | 'bool' | 'char' | 'String'

named_type      = identifier ('::' identifier)*
tuple_type      = '(' type (',' type)* ')'
array_type      = '[' type ']' | '[' type ';' expr ']'
function_type   = 'fun' '(' types? ')' '->' type
optional_type   = type '?'
result_type     = 'Result' '<' type (',' type)? '>'
reference_type  = '&' 'mut'? type
generic_type    = identifier '<' type (',' type)* '>'

refined_type    = '{' identifier ':' type '|' predicate '}'
predicate       = expr  // Boolean expression over identifier
```

#### 2.4 Expressions
```ebnf
expr            = assignment

assignment      = logical_or (assign_op logical_or)*
assign_op       = '=' | '+=' | '-=' | '*=' | '/=' | '%=' | '&=' | '|=' | '^='

logical_or      = logical_and ('||' logical_and)*
logical_and     = equality ('&&' equality)*
equality        = comparison (('==' | '!=') comparison)*
comparison      = bitwise_or (('<' | '<=' | '>' | '>=') bitwise_or)*
bitwise_or      = bitwise_xor ('|' bitwise_xor)*
bitwise_xor     = bitwise_and ('^' bitwise_and)*
bitwise_and     = shift ('&' shift)*
shift           = additive (('<<' | '>>') additive)*
additive        = multiplicative (('+' | '-') multiplicative)*
multiplicative  = power (('*' | '/' | '%' | '//') power)*
power           = pipeline ('**' pipeline)*
pipeline        = postfix ('|>' postfix)*
postfix         = primary postfix_op*
postfix_op      = '.' identifier | '[' expr ']' | '(' args? ')' 
                | '?' | '!' | '!!' | '..' expr?

primary         = literal | identifier | '(' expr ')' | if_expr | when_expr
                | match_expr | for_expr | while_expr | loop_expr
                | lambda | array_expr | tuple_expr | record_expr
                | block | try_expr | async_expr | actor_send

lambda          = '|' params? '|' (expr | block)
                | params '=>' (expr | block)
```

#### 2.5 Control Flow
```ebnf
if_expr         = 'if' expr block ('else' 'if' expr block)* ('else' block)?

when_expr       = 'when' expr? '{' when_arm* '}'
when_arm        = pattern ('if' expr)? '->' expr ','?

match_expr      = 'match' expr '{' match_arm* '}'
match_arm       = pattern ('if' guard)? '=>' expr ','?

for_expr        = 'for' pattern 'in' expr block
while_expr      = 'while' expr block
loop_expr       = 'loop' block

guard_stmt      = 'guard' 'let'? pattern '=' expr 'else' block
defer_stmt      = 'defer' block
```

#### 2.6 Patterns
```ebnf
pattern         = identifier | '_' | literal | tuple_pattern | array_pattern
                | struct_pattern | enum_pattern | range_pattern
                | ref_pattern | mut_pattern | or_pattern | guard_pattern

tuple_pattern   = '(' pattern (',' pattern)* ')'
array_pattern   = '[' pattern (',' pattern)* (',' '..' identifier?)? ']'
struct_pattern  = identifier '{' field_pattern (',' field_pattern)* '}'
field_pattern   = identifier (':' pattern)?
enum_pattern    = identifier '(' pattern* ')'
range_pattern   = expr '..' expr? | '..' expr | expr '..=' expr
ref_pattern     = '&' 'mut'? pattern
mut_pattern     = 'mut' pattern
or_pattern      = pattern ('|' pattern)+
guard_pattern   = pattern 'if' expr
```

### 3. Advanced Features

#### 3.1 Actor System
```ebnf
actor_def       = 'actor' identifier generic_params? '{' 
                  actor_member* 
                  'receive' '{' message_handler* '}'
                  '}'

actor_member    = field_def | method_def
message_handler = pattern ('->' | '=>') block
                | 'after' '(' duration ')' ('->' | '=>') block

actor_send      = expr '!' expr  // Fire and forget
actor_ask       = expr '?' expr  // Request-reply
```

#### 3.2 Pipeline Operators
```ebnf
pipeline_op     = '|>'   // Forward pipe
                | '<|'   // Backward pipe
                | '>>'   // Forward composition
                | '<<'   // Backward composition
                | '>>='  // Monadic bind
                | '>=>'  // Kleisli composition
                | '<$>'  // Functor map
                | '<*>'  // Applicative apply
```

#### 3.3 Null Safety Operators
```ebnf
safe_nav        = expr '?.' identifier  // Safe navigation
null_coalesce   = expr '??' expr        // Null coalescing
force_unwrap    = expr '!'              // Force unwrap
```

#### 3.4 String Interpolation
```ebnf
interp_string   = '"' (string_char | interpolation)* '"'
interpolation   = '\(' expr (':' format_spec)? ')'
format_spec     = width? precision? type_char?
```

#### 3.5 DataFrame Literals
```ebnf
dataframe       = 'df!' '[' column_def (',' column_def)* ']'
column_def      = string_literal ':' '[' expr (',' expr)* ']'
                | string_literal '=>' expr
```

#### 3.6 Async/Await
```ebnf
async_expr      = 'async' block
await_expr      = expr '.await' | 'await' expr
async_fn        = 'async' 'fun' identifier params return_type? block
```

#### 3.7 Property Testing
```ebnf
property_test   = '#[property]' function
property_attr   = '#[' 'property' '(' property_config ')' ']'
property_config = 'cases' '=' integer
                | 'max_shrinks' '=' integer
```

#### 3.8 Refinement Types
```ebnf
refined_type    = '{' binding ':' base_type '|' constraint '}'
constraint      = expr  // Boolean expression
binding         = identifier

// Examples:
// {x: i32 | x > 0}                    // Positive integers
// {xs: Vec<T> | xs.len() > 0}         // Non-empty vectors
// {p: f64 | 0.0 <= p && p <= 1.0}     // Probability
```

#### 3.9 Session Types
```ebnf
session_type    = 'session' identifier '{' session_state* '}'
session_state   = identifier ':' '{' 
                  ('send' ':' type ',')?
                  ('recv' ':' type ',')?
                  'next' ':' identifier
                  '}'
```

#### 3.10 Effect Handlers
```ebnf
effect_def      = 'effect' identifier generic_params? '{' effect_op* '}'
effect_op       = identifier ':' type '->' type

handler         = 'handler' identifier '<' effect '>' '{' 
                  ('return' pattern '=>' expr)?
                  (effect_clause)*
                  '}'
effect_clause   = identifier pattern pattern '=>' expr
```

### 4. Operators (Precedence Table)

| Precedence | Operators | Associativity | Category |
|------------|-----------|---------------|----------|
| 1 | `.` `?.` `::` | Left | Member access |
| 2 | `()` `[]` | Left | Call, index |
| 3 | `!` `~` `-` (unary) | Right | Unary |
| 4 | `**` | Right | Power |
| 5 | `*` `/` `%` `//` | Left | Multiplicative |
| 6 | `+` `-` | Left | Additive |
| 7 | `<<` `>>` | Left | Shift |
| 8 | `&` | Left | Bitwise AND |
| 9 | `^` | Left | Bitwise XOR |
| 10 | `\|` | Left | Bitwise OR |
| 11 | `==` `!=` `<` `<=` `>` `>=` | Left | Comparison |
| 12 | `is` `in` | Left | Type/membership |
| 13 | `&&` | Left | Logical AND |
| 14 | `\|\|` | Left | Logical OR |
| 15 | `..` `...` `..=` | None | Range |
| 16 | `??` | Right | Null coalescing |
| 17 | `\|>` `<\|` | Left | Pipeline |
| 18 | `>>` `<<` (composition) | Left | Composition |
| 19 | `=` `+=` `-=` etc. | Right | Assignment |

### 5. Syntax Sugar Desugarings

#### 5.1 Pipeline Operator
```rust
// Source
data |> filter(p) |> map(f) |> reduce(g)

// Desugars to
reduce(map(filter(data, p), f), g)

// Or with method syntax
data.filter(p).map(f).reduce(g)
```

#### 5.2 Safe Navigation
```rust
// Source
user?.address?.street?.name

// Desugars to
user.and_then(|u| u.address)
    .and_then(|a| a.street)
    .and_then(|s| s.name)
```

#### 5.3 String Interpolation
```rust
// Source
"Hello, \(name)! You are \(age) years old."

// Desugars to
format!("Hello, {}! You are {} years old.", name, age)
```

#### 5.4 For Comprehension
```rust
// Source
[x * 2 for x in data if x > 0]

// Desugars to
data.into_iter()
    .filter(|x| x > 0)
    .map(|x| x * 2)
    .collect()
```

#### 5.5 Actor Message Send
```rust
// Source
actor ! Message(data)

// Desugars to
actor.send(Message(data)).unwrap()

// Source (ask pattern)
let result = actor ? Query(id)

// Desugars to
let (tx, rx) = oneshot::channel();
actor.send(Query(id, tx)).unwrap();
let result = rx.await.unwrap()
```

#### 5.6 Defer Statement
```rust
// Source
defer { cleanup() }

// Desugars to
let _guard = ::scopeguard::guard((), |_| { cleanup() });
```

#### 5.7 Guard Statement
```rust
// Source
guard let Some(x) = opt else { return Err("missing") }

// Desugars to
let Some(x) = opt else { return Err("missing") };
```

### 6. Attributes and Annotations

```ebnf
attributes      = attribute+
attribute       = '#[' meta_item ']'
                | '#![' meta_item ']'  // Inner attribute

meta_item       = identifier
                | identifier '=' literal
                | identifier '(' meta_list ')'

// Common attributes
derive_attr     = '#[derive(' trait_list ')]'
test_attr       = '#[test]'
bench_attr      = '#[bench]'
inline_attr     = '#[inline(' inline_hint? ')]'
mcp_attr        = '#[mcp_tool(' string ')]'
property_attr   = '#[property(' property_config ')]'
contract_attr   = '#[requires(' expr ')]' | '#[ensures(' expr ')]'
```

### 7. Lexical Ambiguities Resolution

#### 7.1 Maximal Munch
- `>>` is always right-shift, not two closing generics
- `..=` is inclusive range, not `..` followed by `=`
- `//` is integer division in expressions, comment at line start

#### 7.2 Context-Sensitive Parsing
- `<` in `Vec<T>` vs `a < b` determined by type context
- `|` in patterns vs closures vs bitwise OR
- `.` in floating literals vs method calls

### 8. Grammar Extensions

#### 8.1 Macro System (Future)
```ebnf
macro_def       = 'macro' identifier '(' macro_params ')' '{' macro_body '}'
macro_call      = identifier '!' token_tree
```

#### 8.2 Const Generics
```ebnf
const_generic   = 'const' identifier ':' type
array_type      = '[' type ';' const_expr ']'
```

#### 8.3 Associated Types
```ebnf
assoc_type      = 'type' identifier (':' bounds)? ('=' type)?
```

### 9. Reserved for Future Use

```
abstract become box do final macro move
override priv typeof unsafe unsized virtual yield
```

### 10. Implementation Notes

#### 10.1 Parser Architecture
- Recursive descent with 2-token lookahead
- Pratt parsing for expression precedence
- Error recovery at statement boundaries
- Incremental parsing for REPL/IDE

#### 10.2 Performance Targets
- Lexing: >10M tokens/sec
- Parsing: >1M LOC/sec
- AST memory: <100 bytes/LOC average
- Error recovery: <5ms for typical errors

#### 10.3 Compatibility Rules
- All valid Rust types are valid Ruchy types
- Rust syntax subset works unchanged
- Generated Rust preserves semantics exactly
- No runtime overhead vs equivalent Rust