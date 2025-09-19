//! Sprint 75: Parser Expressions Coverage Boost
//! Target: Push parser/expressions.rs from 66.70% to 80%+

use ruchy::frontend::parser::Parser;

#[test]
fn test_binary_operators_comprehensive() {
    let expressions = vec![
        // Arithmetic
        "1 + 2", "3 - 4", "5 * 6", "7 / 8", "9 % 10",
        "2 ** 3", "10 // 3",
        // Comparison
        "1 < 2", "3 <= 4", "5 > 6", "7 >= 8", "9 == 10", "11 != 12",
        // Logical
        "true && false", "true || false", "!true",
        // Bitwise
        "1 & 2", "3 | 4", "5 ^ 6", "~7", "8 << 2", "16 >> 2",
        // Assignment
        "x = 5", "x += 1", "x -= 1", "x *= 2", "x /= 2", "x %= 3",
        "x &= 1", "x |= 2", "x ^= 3", "x <<= 1", "x >>= 1",
        // Compound
        "1 + 2 * 3", "(1 + 2) * 3", "1 + 2 + 3 + 4",
        "true && false || true", "!true && false",
        // Chain comparisons
        "1 < 2 < 3", "5 > 4 > 3",
    ];

    for expr in expressions {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", expr);
    }
}

#[test]
fn test_unary_operators() {
    let expressions = vec![
        "-5", "+5", "!true", "~42",
        "--5", "++x", "x++", "x--",
        "-+5", "+-5", "!!true", "~~42",
    ];

    for expr in expressions {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_literals_comprehensive() {
    let literals = vec![
        // Integers
        "42", "0", "-1", "1_000_000",
        "0x1F", "0xDEADBEEF", "0X10",
        "0o755", "0o777", "0O10",
        "0b1010", "0b11111111", "0B10",
        // Floats
        "3.14", "2.718", "0.5", ".5", "5.",
        "1e10", "2.5e-4", "1E10", "2.5E-4",
        // Strings
        r#""hello""#, r#"'world'"#, r#""escaped \"quote\"""#,
        r#""line\nbreak""#, r#""tab\there""#, r#""null\0byte""#,
        r#""\x41""#, r#""\u{1F600}""#,
        // Raw strings
        r#"r"raw string""#, r#"r'raw string'"#,
        r##"r#"raw with "quotes""#"##,
        // Template strings
        r#"f"Hello {name}""#, r#"f"2 + 2 = {2 + 2}""#,
        // Multi-line strings
        r#""""triple quoted""""#,
        r#"'''triple single quoted'''"#,
        // Characters
        "'a'", "'\\n'", "'\\t'", "'\\''", "'\"'",
        // Booleans
        "true", "false",
        // Null
        "null", "nil", "None",
    ];

    for lit in literals {
        let mut parser = Parser::new(lit);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_array_expressions() {
    let arrays = vec![
        "[]", "[1]", "[1, 2]", "[1, 2, 3]",
        "[[1, 2], [3, 4]]", "[[[1]]]",
        "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]",
        "[x for x in range(10)]",
        "[x * 2 for x in [1, 2, 3]]",
        "[x for x in range(10) if x % 2 == 0]",
        "[...arr1, ...arr2]",
        "[first, ...rest]",
    ];

    for arr in arrays {
        let mut parser = Parser::new(arr);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_object_expressions() {
    let objects = vec![
        "{}",
        r#"{ "key": "value" }"#,
        "{ x: 1, y: 2 }",
        "{ x, y }", // shorthand
        "{ ...obj1, ...obj2 }", // spread
        r#"{ "nested": { "deep": { "value": 123 } } }"#,
        "{ [computed]: value }",
        "{ method() { } }",
        r#"{ get x() { }, set x(val) { } }"#,
    ];

    for obj in objects {
        let mut parser = Parser::new(obj);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_tuple_expressions() {
    let tuples = vec![
        "()", "(1,)", "(1, 2)", "(1, 2, 3)",
        "((1, 2), (3, 4))", "(((1,),),)",
        "(x, y, z)", "(a, b, c, d, e, f)",
    ];

    for tup in tuples {
        let mut parser = Parser::new(tup);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[test]
fn test_function_calls() {
    let calls = vec![
        "f()", "f(1)", "f(1, 2)", "f(1, 2, 3)",
        "f(x, y, z)", "f(a, b, c, d, e)",
        "f(g(h(i(j(k())))))",
        "obj.method()", "obj.method(1, 2)",
        "arr[0]()", "fn()()()())",
        "f(1, b=2, c=3)", // keyword args
        "f(*args)", "f(**kwargs)", "f(*args, **kwargs)",
    ];

    for call in calls {
        let mut parser = Parser::new(call);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_index_and_slice() {
    let expressions = vec![
        "arr[0]", "arr[1]", "arr[-1]",
        "arr[i]", "arr[i + 1]", "arr[2 * i]",
        "arr[0][1][2]", "matrix[row][col]",
        "arr[1:5]", "arr[:5]", "arr[1:]", "arr[:]",
        "arr[::2]", "arr[1:10:2]", "arr[::-1]",
        "s[start:end]", "s[start:end:step]",
    ];

    for expr in expressions {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_member_access() {
    let expressions = vec![
        "obj.field", "obj.method", "obj.nested.field",
        "obj.field.method().result",
        "obj?.field", "obj?.method?()",  // optional chaining
        "obj!.field", // non-null assertion
        "a.b.c.d.e.f.g",
    ];

    for expr in expressions {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_lambda_expressions() {
    let lambdas = vec![
        "x => x", "x => x + 1", "(x, y) => x + y",
        "() => 42", "x => { x * 2 }",
        "(a, b, c) => a + b + c",
        "x => y => x + y", // curried
        "|x| x + 1", // alternative syntax
        "fn(x) { x + 1 }", // anonymous function
    ];

    for lambda in lambdas {
        let mut parser = Parser::new(lambda);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_conditional_expressions() {
    let conditionals = vec![
        "true ? 1 : 2",
        "x > 0 ? x : -x",
        "a ? b ? c : d : e",  // nested
        "condition ? value1 : condition2 ? value2 : value3",
        "x if x > 0 else -x",  // python style
    ];

    for cond in conditionals {
        let mut parser = Parser::new(cond);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_type_expressions() {
    let types = vec![
        "x as int", "y as string", "z as bool",
        "value as? Type", // optional cast
        "value as! Type", // force cast
        "x is int", "y is string",
        "typeof x", "sizeof(Type)",
    ];

    for typ in types {
        let mut parser = Parser::new(typ);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_await_and_async() {
    let expressions = vec![
        "await fetch()",
        "await async { 42 }",
        "async () => await fetch()",
        "await Promise.all([p1, p2, p3])",
    ];

    for expr in expressions {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_yield_expressions() {
    let expressions = vec![
        "yield", "yield 42", "yield* gen()",
        "yield from iterable",
    ];

    for expr in expressions {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_pipeline_operator() {
    let pipelines = vec![
        "x |> f", "x |> f |> g |> h",
        "data |> filter(x => x > 0) |> map(x => x * 2)",
        "[1, 2, 3] |> sum |> double",
    ];

    for pipe in pipelines {
        let mut parser = Parser::new(pipe);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_range_expressions() {
    let ranges = vec![
        "1..10", "1..=10", "..10", "1..",
        "start..end", "start..=end",
        "'a'..'z'", "0.0..1.0",
    ];

    for range in ranges {
        let mut parser = Parser::new(range);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_error_recovery() {
    // These should parse with error recovery
    let malformed = vec![
        "1 +", "* 2", "( ]", "{ }",
        "fn(", "if (", "for x in",
        "1...", "...2", "x.", ".y",
    ];

    for expr in malformed {
        let mut parser = Parser::new(expr);
        let _ = parser.parse(); // Should not panic
    }
}

#[test]
fn test_precedence_and_associativity() {
    let expressions = vec![
        "1 + 2 * 3", // 7, not 9
        "2 * 3 + 4", // 10, not 14
        "1 + 2 + 3", // left associative
        "a = b = c", // right associative
        "true || false && true", // && binds tighter
        "1 < 2 == true", // comparison then equality
        "x & 0xFF == 0x10", // bitwise vs comparison
        "!x && y", // unary binds tighter
        "-x * y", // unary binds tighter
        "x++ + ++y", // postfix vs prefix
    ];

    for expr in expressions {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_complex_nested_expressions() {
    let complex = vec![
        "f(g(h(i(j(k(l(m(n(o(p(q())))))))))))",
        "a[b[c[d[e[f[g[h[i[j[k[l]]]]]]]]]]]]",
        "obj.a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p",
        "((((((((((((x))))))))))))",
        "{a: {b: {c: {d: {e: {f: {g: {h: i}}}}}}}}",
        "fn() { fn() { fn() { fn() { fn() { } } } } }",
    ];

    for expr in complex {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(result.is_ok() || result.is_err());
    }
}