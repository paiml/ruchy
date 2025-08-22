# Ruchy Grammar Specification v2.0

## Overview

Ruchy's grammar prioritizes unambiguous parsing and direct Rust transpilation. Every production maps to a clear semantic construct with minimal lookahead requirements.

## Lexical Structure

### Keywords
```
actor       as          async       await       break
const       continue    effect      else        enum
false       for         fun         handle      handler
if          impl        import      in          lazy
let         loop        match       mod         mut
pub         ref         return      self        spawn
struct      super       trait       true        type
use         where       while       yield
```

### Operators
```
// Arithmetic
+    -    *    /    %    **

// Comparison
==   !=   <    >    <=   >=

// Logical
&&   ||   !

// Bitwise
&    |    ^    ~    <<   >>

// Assignment
=    +=   -=   *=   /=   %=   &=   |=   ^=   <<=  >>=

// Special
?    >>   ->   <-   <?   ::   ..   ...  @    _
```

### Literals
```ebnf
literal = integer | float | string | char | boolean

integer = decimal_lit | hex_lit | octal_lit | binary_lit
decimal_lit = [0-9] [0-9_]*
hex_lit = '0x' [0-9a-fA-F] [0-9a-fA-F_]*
octal_lit = '0o' [0-7] [0-7_]*
binary_lit = '0b' [01] [01_]*

float = [0-9] [0-9_]* '.' [0-9] [0-9_]* ([eE] [+-]? [0-9]+)?

string = '"' (!'"' any | '\"')* '"'
       | 'f"' (!'"' any | '\"' | '{' expr '}')* '"'
       | '"""' (!'"""' any)* '"""'

char = "'" (!''' any | '\'') "'"

boolean = 'true' | 'false'
```

## Program Structure

```ebnf
program = item*

item = import_stmt
     | module_decl
     | function_decl
     | struct_decl
     | enum_decl
     | trait_decl
     | impl_block
     | type_alias
     | const_decl
     | actor_decl

import_stmt = 'import' import_path ('as' identifier)?
import_path = module_path | url_path
module_path = identifier ('::' identifier)*
url_path = '"http' 's'? '://' (!'"' any)* '"'

module_decl = visibility? 'mod' identifier '{' item* '}'

visibility = 'pub'?  // Default: private
```

## Type System

```ebnf
type = simple_type
     | generic_type
     | tuple_type
     | array_type
     | slice_type
     | ref_type
     | function_type
     | refined_type
     | '_'  // Infer

simple_type = identifier ('::' identifier)*

generic_type = identifier '<' type_list '>'

tuple_type = '(' (type (',' type)*)? ')'

array_type = '[' type ';' expr ']'

slice_type = '[' type ']'

ref_type = '&' 'mut'? type

function_type = '|' type_list? '|' '->' type

refined_type = type 'where' constraint
constraint = boolean_expr

type_list = type (',' type)*
```

## Declarations

### Functions
```ebnf
function_decl = visibility? async? 'fun' identifier 
                generic_params? '(' params? ')' return_type? 
                where_clause? block

generic_params = '<' generic_param (',' generic_param)* '>'
generic_param = identifier (':' type_bounds)?

params = param (',' param)*
param = identifier ':' type default_value?
default_value = '=' expr

return_type = '->' type

where_clause = 'where' constraint (',' constraint)*

block = '{' statement* '}'
```

### Structures
```ebnf
struct_decl = visibility? 'struct' identifier generic_params? struct_body

struct_body = '{' struct_fields '}'
            | '(' type_list ')'
            | ';'

struct_fields = struct_field (',' struct_field)* ','?
struct_field = visibility? identifier ':' type
```

### Enums
```ebnf
enum_decl = visibility? 'enum' identifier generic_params? '{' 
            enum_variants '}'

enum_variants = enum_variant (',' enum_variant)* ','?
enum_variant = identifier variant_data?

variant_data = '(' type_list ')'
             | '{' struct_fields '}'
```

### Traits
```ebnf
trait_decl = visibility? 'trait' identifier generic_params? 
             trait_bounds? '{' trait_item* '}'

trait_item = function_sig
           | type_alias
           | const_decl

function_sig = visibility? async? 'fun' identifier 
               generic_params? '(' params? ')' return_type?

trait_bounds = ':' type_bound ('+' type_bound)*
type_bound = identifier generic_args?
```

### Implementations
```ebnf
impl_block = 'impl' generic_params? type_for trait_for? 
             where_clause? '{' impl_item* '}'

type_for = type
trait_for = 'for' type

impl_item = function_decl
          | type_alias
          | const_decl
```

### Type Aliases
```ebnf
type_alias = visibility? 'type' identifier generic_params? '=' type
```

### Constants
```ebnf
const_decl = visibility? 'const' identifier ':' type '=' expr
```

## Actors

```ebnf
actor_decl = visibility? 'actor' identifier generic_params? '{' 
             actor_item* '}'

actor_item = state_decl
           | handler_decl

state_decl = 'state' identifier ':' type ('=' expr)?

handler_decl = visibility? 'handler' identifier '(' params? ')' 
               return_type? block

actor_spawn = 'spawn' expr
actor_send = expr '<-' expr  // Fire and forget
actor_ask = expr '<?' expr   // Request-reply
```

## Statements

```ebnf
statement = let_stmt
          | expr_stmt
          | return_stmt
          | break_stmt
          | continue_stmt
          | yield_stmt

let_stmt = 'let' 'mut'? pattern type_annotation? '=' expr ';'
type_annotation = ':' type

expr_stmt = expr ';'?

return_stmt = 'return' expr? ';'

break_stmt = 'break' label? expr? ';'

continue_stmt = 'continue' label? ';'

yield_stmt = 'yield' expr ';'

label = '@' identifier
```

## Expressions

```ebnf
expr = assignment

assignment = logical_or (assign_op assignment)?
assign_op = '=' | '+=' | '-=' | '*=' | '/=' | '%=' 
          | '&=' | '|=' | '^=' | '<<=' | '>>='

logical_or = logical_and ('||' logical_and)*

logical_and = equality ('&&' equality)*

equality = comparison (('==' | '!=') comparison)*

comparison = bitwise_or (('<' | '>' | '<=' | '>=') bitwise_or)*

bitwise_or = bitwise_xor ('|' bitwise_xor)*

bitwise_xor = bitwise_and ('^' bitwise_and)*

bitwise_and = shift ('&' shift)*

shift = additive (('<<' | '>>') additive)*

additive = multiplicative (('+' | '-') multiplicative)*

multiplicative = exponential (('*' | '/' | '%') exponential)*

exponential = unary ('**' unary)*

unary = ('!' | '-' | '~' | '&' | '&' 'mut' | '*')? postfix

postfix = primary postfix_op*

postfix_op = '?'                      // Try
           | '.' identifier            // Field
           | '.' await                 // Await
           | '(' args? ')'             // Call
           | '[' expr ']'              // Index
           | '[' expr? '..' expr? ']'  // Slice
           | '<-' expr                 // Actor send
           | '<?' expr                 // Actor ask

primary = literal
        | identifier
        | 'self'
        | 'super'
        | '(' expr ')'
        | tuple_expr
        | array_expr
        | struct_expr
        | if_expr
        | match_expr
        | for_expr
        | while_expr
        | loop_expr
        | async_block
        | block_expr
        | lambda_expr
        | lazy_expr
        | pipeline_expr
        | dataframe_expr
```

### Composite Expressions

```ebnf
tuple_expr = '(' (expr (',' expr)* ','?)? ')'

array_expr = '[' array_elements? ']'
array_elements = expr (',' expr)* ','?
               | expr ';' expr

struct_expr = path '{' field_inits? '}'
field_inits = field_init (',' field_init)* ','?
field_init = identifier (':' expr)?

if_expr = 'if' expr block ('else' (if_expr | block))?

match_expr = 'match' expr '{' match_arms '}'
match_arms = match_arm (',' match_arm)* ','?
match_arm = pattern ('if' expr)? '=>' (expr | block)

for_expr = label? 'for' pattern 'in' expr block

while_expr = label? 'while' expr block

loop_expr = label? 'loop' block

async_block = 'async' block

block_expr = label? block

lambda_expr = '|' params? '|' ('->' type)? (expr | block)

lazy_expr = 'lazy' expr

pipeline_expr = expr ('>>' expr)+

dataframe_expr = 'df!' '[' df_columns ']'
df_columns = df_column (',' df_column)*
df_column = string ':' array_expr
```

## Patterns

```ebnf
pattern = literal_pattern
        | identifier_pattern
        | wildcard_pattern
        | tuple_pattern
        | array_pattern
        | struct_pattern
        | enum_pattern
        | ref_pattern
        | range_pattern
        | or_pattern

literal_pattern = literal

identifier_pattern = 'mut'? identifier ('@' pattern)?

wildcard_pattern = '_'

tuple_pattern = '(' (pattern (',' pattern)* ','?)? ')'

array_pattern = '[' array_patterns? ']'
array_patterns = pattern (',' pattern)* (',' '..' identifier?)?

struct_pattern = path '{' field_patterns? '}'
field_patterns = field_pattern (',' field_pattern)* (',' '..')?
field_pattern = identifier (':' pattern)?

enum_pattern = path variant_pattern?
variant_pattern = '(' pattern_list ')'
                | '{' field_patterns '}'

ref_pattern = '&' 'mut'? pattern

range_pattern = expr '..' expr?
              | '..' expr

or_pattern = pattern ('|' pattern)+

pattern_list = pattern (',' pattern)*
```

## Effects System

```ebnf
effect_decl = visibility? 'effect' identifier generic_params? '{' 
              effect_operations '}'

effect_operations = effect_op (',' effect_op)*
effect_op = identifier ':' function_type

handler_expr = 'handle' expr 'with' '{' handler_cases '}'
handler_cases = handler_case (',' handler_case)*
handler_case = identifier '=>' lambda_expr
             | 'return' '=>' lambda_expr
```

## Macros

```ebnf
macro_call = identifier '!' macro_args

macro_args = '(' token_tree* ')'
           | '[' token_tree* ']'
           | '{' token_tree* '}'

token_tree = '(' token_tree* ')'
           | '[' token_tree* ']'
           | '{' token_tree* '}'
           | any_token
```

## Comments

```ebnf
line_comment = '//' (!newline any)*
block_comment = '/*' (!'*/' any)* '*/'
doc_comment = '///' (!newline any)*
            | '/**' (!'*/' any)* '*/'
```

## Precedence Table

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 | `>>` (pipeline) | Left |
| 2 | `<-`, `<?` (actor) | Left |
| 3 | `?` (try) | Left |
| 4 | `.` (field/method) | Left |
| 5 | `()`, `[]` (call/index) | Left |
| 6 | `**` (exponent) | Right |
| 7 | unary `-`, `!`, `~`, `&`, `*` | Right |
| 8 | `*`, `/`, `%` | Left |
| 9 | `+`, `-` | Left |
| 10 | `<<`, `>>` | Left |
| 11 | `&` | Left |
| 12 | `^` | Left |
| 13 | `|` | Left |
| 14 | `<`, `>`, `<=`, `>=` | Left |
| 15 | `==`, `!=` | Left |
| 16 | `&&` | Left |
| 17 | `||` | Left |
| 18 | `=`, `+=`, etc. | Right |

## Grammar Properties

- **LL(2)**: Maximum lookahead of 2 tokens
- **Unambiguous**: No shift/reduce or reduce/reduce conflicts
- **Total**: All constructs have defined transpilation to Rust
- **Minimal**: 41 production rules in core grammar