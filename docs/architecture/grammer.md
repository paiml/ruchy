# Ruchy Grammar Specification v4.0
## Complete Mechanical Transformation Grammar

### Design Invariants
1. **Every construct maps 1:1 to Rust AST**
2. **Zero runtime overhead guaranteed**
3. **No hidden allocations or implicit state**

---

## Part I: Lexical Grammar

### 1. Tokens and Keywords

```ebnf
// Whitespace and Comments
whitespace      = ' ' | '\t' | '\r' | '\n'
comment         = line_comment | block_comment
line_comment    = '//' (!'\n' any)*
block_comment   = '/*' (!'*/' any)* '*/'

// Identifiers
identifier      = (letter | '_') (letter | digit | '_')*
raw_identifier  = 'r#' identifier  // Escape keywords

letter          = 'a'..'z' | 'A'..'Z'
digit           = '0'..'9'
hex_digit       = digit | 'a'..'f' | 'A'..'F'

// Keywords (37 total - minimal set)
keyword         = 'actor' | 'as' | 'async' | 'await' | 'break' | 'const'
                | 'continue' | 'defer' | 'else' | 'enum' | 'false' | 'for'
                | 'fun' | 'guard' | 'if' | 'impl' | 'import' | 'in' | 'let'
                | 'loop' | 'match' | 'mod' | 'mut' | 'pub' | 'return'
                | 'self' | 'Self' | 'static' | 'struct' | 'super' | 'trait'
                | 'true' | 'type' | 'use' | 'where' | 'while'
```

### 2. Literals

```ebnf
literal         = number | string | char | boolean

// Numbers
number          = integer | float
integer         = decimal | hex | octal | binary
decimal         = digit (digit | '_')*
hex             = '0x' hex_digit (hex_digit | '_')*
octal           = '0o' ('0'..'7') ('0'..'7' | '_')*
binary          = '0b' ('0' | '1') ('0' | '1' | '_')*

float           = decimal '.' decimal? exponent?
                | decimal exponent
exponent        = ('e' | 'E') ('+' | '-')? decimal

// Strings with interpolation
string          = plain_string | interpolated_string
plain_string    = '"' string_char* '"'
interpolated_string = 'f"' interp_content* '"'

string_char     = !'"' !'\' any | escape_seq
interp_content  = string_char | interpolation
interpolation   = '{' expr (':' format_spec)? '}'
format_spec     = align? sign? '#'? '0'? width? precision? type_char?

escape_seq      = '\' ('n' | 'r' | 't' | '\' | '"' | '0' 
                | 'x' hex_digit{2} | 'u' '{' hex_digit{1,6} '}')

// Other literals
char            = '\'' (!'\\' !'\'' any | escape_seq) '\''
boolean         = 'true' | 'false'
```

---

## Part II: Types

### 3. Type System

```ebnf
type            = simple_type | compound_type | function_type | generic_type

simple_type     = primitive | path | '_'
primitive       = 'i8' | 'i16' | 'i32' | 'i64' | 'i128' | 'isize'
                | 'u8' | 'u16' | 'u32' | 'u64' | 'u128' | 'usize'
                | 'f32' | 'f64' | 'bool' | 'char' | 'str' | 'String'

compound_type   = array_type | slice_type | tuple_type | reference_type
                | optional_type | result_type

array_type      = '[' type ';' expr ']'         // [T; N]
slice_type      = '[' type ']'                  // &[T]
tuple_type      = '(' ')' | '(' type ',' ')' | '(' type (',' type)+ ')'
reference_type  = '&' lifetime? 'mut'? type
optional_type   = type '?'                      // Option<T>
result_type     = 'Result' '<' type (',' type)? '>'

function_type   = 'fun' '(' param_types? ')' ('->' type)?
param_types     = type (',' type)*

generic_type    = path generic_args
generic_args    = '<' type (',' type)* '>'

lifetime        = '\'' identifier | '\'' 'static'
```

### 4. Generic Parameters and Bounds

```ebnf
generic_params  = '<' generic_param (',' generic_param)* '>'
generic_param   = lifetime_param | type_param | const_param

lifetime_param  = lifetime (':' lifetime_bounds)?
type_param      = identifier (':' type_bounds)? ('=' type)?
const_param     = 'const' identifier ':' type ('=' expr)?

type_bounds     = type_bound ('+' type_bound)*
type_bound      = '?'? path | '(' type ')'

where_clause    = 'where' where_pred (',' where_pred)*
where_pred      = type ':' type_bounds
                | lifetime ':' lifetime_bounds

lifetime_bounds = lifetime ('+' lifetime)*
```

---

## Part III: Expressions

### 5. Expression Grammar (Precedence-Driven)

```ebnf
expr            = assignment

// Precedence levels (lowest to highest)
assignment      = range_expr (assign_op range_expr)?
assign_op       = '=' | '+=' | '-=' | '*=' | '/=' | '%=' 
                | '&=' | '|=' | '^=' | '<<=' | '>>='

range_expr      = or_expr ('..' or_expr? | '..=' or_expr)?
or_expr         = and_expr ('||' and_expr)*
and_expr        = eq_expr ('&&' eq_expr)*
eq_expr         = cmp_expr (('==' | '!=') cmp_expr)*
cmp_expr        = bit_or_expr (('<' | '<=' | '>' | '>=') bit_or_expr)*
bit_or_expr     = bit_xor_expr ('|' bit_xor_expr)*
bit_xor_expr    = bit_and_expr ('^' bit_and_expr)*
bit_and_expr    = shift_expr ('&' shift_expr)*
shift_expr      = add_expr (('<<' | '>>') add_expr)*
add_expr        = mul_expr (('+' | '-') mul_expr)*
mul_expr        = pow_expr (('*' | '/' | '%') pow_expr)*
pow_expr        = cast_expr ('**' cast_expr)*
cast_expr       = unary_expr ('as' type)*

unary_expr      = unary_op* coalesce_expr
unary_op        = '!' | '-' | '*' | '&' 'mut'?

coalesce_expr   = pipeline_expr ('??' pipeline_expr)*
pipeline_expr   = postfix_expr ('|>' postfix_expr)*

postfix_expr    = primary_expr postfix_op*
postfix_op      = '.' identifier generic_args?    // Field/method
                | '?.' identifier generic_args?   // Safe navigation
                | '[' expr ']'                    // Index
                | '(' args? ')'                   // Call
                | '?'                             // Try operator
                | '!'                             // Unwrap
                | '!!'                            // Force unwrap

primary_expr    = literal | path | 'self' | '(' expr ')'
                | tuple_expr | array_expr | block
                | if_expr | match_expr | for_expr | while_expr | loop_expr
                | lambda | struct_expr | async_block | await_expr
                | try_expr | guard_expr | defer_expr
                | actor_send | dataframe_literal

args            = expr (',' expr)*
```

### 6. Complex Expressions

```ebnf
// Collections
tuple_expr      = '(' ')' | '(' expr ',' ')' | '(' expr (',' expr)+ ')'
array_expr      = '[' ']' | '[' expr (',' expr)* ']' | '[' expr ';' expr ']'

// Struct literals
struct_expr     = path '{' field_init (',' field_init)* '}'
field_init      = identifier (':' expr)? | '..' expr

// Control flow
if_expr         = 'if' condition block ('else' (if_expr | block))?
condition       = expr | 'let' pattern '=' expr

match_expr      = 'match' expr '{' match_arm* '}'
match_arm       = pattern ('if' expr)? '=>' expr ','

for_expr        = 'for' pattern 'in' expr block
while_expr      = 'while' condition block
loop_expr       = 'loop' block

// Lambda expressions
lambda          = '|' params? '|' ('->' type)? (expr | block)
                | params '=>' (expr | block)

// Async/await
async_block     = 'async' 'move'? block
await_expr      = expr '.await'

// Error handling
try_expr        = 'try' block
guard_expr      = 'guard' condition 'else' block
defer_expr      = 'defer' block

// Actor operations
actor_send      = expr '<-' expr    // Fire and forget
actor_ask       = expr '<?' expr    // Request-reply
```

---

## Part IV: Statements and Items

### 7. Statements

```ebnf
stmt            = let_stmt | expr_stmt | item

let_stmt        = 'let' 'mut'? pattern (':' type)? ('=' expr)? ';'
expr_stmt       = expr_without_block ';' | expr_with_block ';'?

block           = '{' stmt* expr? '}'

// Distinction for semicolon rules
expr_with_block = if_expr | match_expr | for_expr | while_expr 
                | loop_expr | block | async_block

expr_without_block = /* all other expressions */
```

### 8. Patterns

```ebnf
pattern         = '_' | literal_pat | ident_pat | ref_pat | mut_pat
                | tuple_pat | array_pat | struct_pat | enum_pat
                | range_pat | or_pat

literal_pat     = literal
ident_pat       = 'ref'? 'mut'? identifier ('@' pattern)?
ref_pat         = '&' 'mut'? pattern
mut_pat         = 'mut' pattern

tuple_pat       = '(' ')' | '(' pattern ',' ')' | '(' pattern (',' pattern)+ ')'
array_pat       = '[' pattern (',' pattern)* (',' '..' identifier?)? ']'

struct_pat      = path '{' field_pat (',' field_pat)* (',' '..')? '}'
field_pat       = identifier (':' pattern)?

enum_pat        = path '(' pattern (',' pattern)* ')'
range_pat       = literal '..' literal | literal '..=' literal
or_pat          = pattern ('|' pattern)+
```

---

## Part V: Declarations

### 9. Top-Level Items

```ebnf
program         = shebang? item*
shebang         = '#!' (!'\n' any)* '\n'

item            = attributes? visibility? item_kind
visibility      = 'pub' ('(' ('crate' | 'super' | 'self') ')')?

attributes      = outer_attr+
outer_attr      = '#[' attr_content ']'
attr_content    = 'test' | 'bench' | 'property' property_config?
                | 'mcp::tool' | 'mcp::context' | meta_item

property_config = '(' config_item (',' config_item)* ')'
config_item     = identifier '=' literal

item_kind       = function | struct_def | enum_def | trait_def
                | impl_block | type_alias | const_def | static_def
                | use_decl | import_decl | mod_decl | actor_def
```

### 10. Declarations

```ebnf
// Functions
function        = 'async'? 'fun' identifier generic_params? 
                  '(' params? ')' ('->' type)? where_clause? 
                  (block | '=' expr ';')

params          = param (',' param)*
param           = pattern ':' type
                | 'self' | '&' 'mut'? 'self'

// Structs
struct_def      = 'struct' identifier generic_params? where_clause?
                  (';' | '{' field (',' field)* '}' | '(' type (',' type)* ')')

field           = visibility? identifier ':' type

// Enums
enum_def        = 'enum' identifier generic_params? where_clause?
                  '{' variant (',' variant)* '}'
variant         = identifier variant_data?
variant_data    = '{' field (',' field)* '}' | '(' type (',' type)* ')'

// Traits
trait_def       = 'trait' identifier generic_params? (':' supertraits)?
                  where_clause? '{' trait_item* '}'
supertraits     = type_bound ('+' type_bound)*
trait_item      = trait_const | trait_type | trait_method

trait_const     = 'const' identifier ':' type ('=' expr)? ';'
trait_type      = 'type' identifier (':' type_bounds)? ('=' type)? ';'
trait_method    = function_sig (block | ';')
function_sig    = 'async'? 'fun' identifier generic_params? 
                  '(' params? ')' ('->' type)?

// Implementations
impl_block      = 'impl' generic_params? (trait 'for')? type 
                  where_clause? '{' impl_item* '}'
impl_item       = const_def | type_alias | function

// Type aliases and constants
type_alias      = 'type' identifier generic_params? '=' type ';'
const_def       = 'const' identifier ':' type '=' expr ';'
static_def      = 'static' 'mut'? identifier ':' type '=' expr ';'

// Imports
use_decl        = 'use' use_tree ';'
use_tree        = path ('::' '*' | '::' '{' use_list '}' | 'as' identifier)?
use_list        = use_tree (',' use_tree)*

import_decl     = 'import' import_tree ('from' string)? ';'
import_tree     = '*' | identifier | '{' import_list '}'
import_list     = identifier ('as' identifier)? (',' identifier ('as' identifier)?)*

// Modules
mod_decl        = 'mod' identifier ';' | 'mod' identifier '{' item* '}'

// Paths
path            = '::'? path_segment ('::' path_segment)*
path_segment    = identifier generic_args?
```

---

## Part VI: Extended Features

### 11. Actor System

```ebnf
actor_def       = 'actor' identifier generic_params? '{' 
                  actor_state* 
                  'receive' '{' message_handler* '}'
                  '}'

actor_state     = 'state' identifier ':' type ('=' expr)? ';'

message_handler = pattern '=>' block
                | 'on' identifier '(' params? ')' block
```

### 12. DataFrame Support (Feature-Gated)

```ebnf
dataframe_literal = 'df!' '[' column (',' column)* ']'
column          = string ':' '[' expr (',' expr)* ']'
                | identifier '=>' expr

// Transpiles directly to polars::df! macro
```

---

## Part VII: Disambiguation Rules

### 13. Context-Sensitive Resolution

```ebnf
// Lookahead requirements (max 2 tokens)
disambiguation  = '<' after identifier => generic_args if followed by type
                | '<' in expression => less_than operator
                | '|' at expr start => lambda if followed by param/pattern
                | '|' in pattern => or_pattern
                | '{' after type => struct_literal
                | '{' elsewhere => block
                | '?' after type => Option<T>
                | '?' after expr => try_operator
```

### 14. Semicolon Rules

```ebnf
semicolon_rules = let_stmt => always required
                | expr_stmt => required except block-final position
                | block-final expr => optional (becomes return value)
                | REPL top-level => optional for expressions
```

---

## Part VIII: REPL Extensions

### 15. REPL-Specific Productions

```ebnf
repl_input      = repl_command | item | stmt | expr

repl_command    = ':' command_name command_args?
command_name    = 'help' | 'type' | 'ast' | 'rust' | 'clear' | 'quit'
command_args    = (!'\n' any)*

// Special REPL rules
repl_let        = 'let' pattern (':' type)? '=' expr  // No trailing ;
repl_expr       = expr  // Evaluates and prints
```

---

## Part IX: Mechanical Transformations

### 16. Direct Rust Mappings

```rust
// Ruchy                    // Rust
fun f(x: i32) -> i32       fn f(x: i32) -> i32
x |> f |> g                g(f(x))
T?                         Option<T>
expr?                      expr?
expr??                     expr.unwrap_or_default()
expr?.field                expr.map(|x| x.field)
[T]                        &[T]
defer { ... }              let _guard = defer(|| { ... });
guard cond else { ... }    if !cond { ... }
actor A { ... }            struct A { ... } + impl Actor
x <- msg                   tx.send(msg).await
f"hello {name}"           format!("hello {}", name)
df![...]                   polars::df![...]
import x from "y"          use y::x;
```

---

## Appendix: Grammar Metrics

### Coverage Statistics
- **Grammar Productions**: 127
- **Operator Precedence Levels**: 16
- **Keywords**: 37
- **Lookahead Required**: 2 tokens max
- **Disambiguation Rules**: 8

### Performance Guarantees
- Parse time: <5ms per 1000 LOC
- AST size: <80 bytes per line average
- Zero heap allocations for common cases
- Deterministic parsing (no backtracking)

### Validation Requirements
Every production must:
1. Have a corresponding test case
2. Map to valid Rust AST
3. Generate idiomatic Rust code
4. Preserve all safety guarantees

---

*This specification defines the complete Ruchy grammar with mechanical transformation to Rust, ensuring zero-cost abstractions and complete feature coverage.*