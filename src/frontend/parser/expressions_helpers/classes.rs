//! Class definition parsing
//!
//! Handles parsing of class (OOP-style type) definitions:
//! - Class declarations: `class MyClass { fields, methods }`
//! - Inheritance: `class Child : Parent + Trait1 + Trait2`
//! - Constructors: `new() { ... }`
//! - Methods: instance and static methods
//! - Constants and properties
//! - Visibility modifiers
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{
    ClassConstant, ClassMethod, ClassProperty, Constructor, Decorator, Expr, ExprKind, Param, PropertySetter, SelfType, Span, StructField, Visibility,
};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, parse_expr_recursive, utils, ParserState, Result};

pub(in crate::frontend::parser) fn parse_class_definition(
    state: &mut ParserState,
    name: String,
    type_params: Vec<String>,
    start_span: Span,
) -> Result<Expr> {
    let (superclass, traits) = parse_inheritance(state)?;
    let (fields, constructors, methods, constants, properties) = parse_class_body(state)?;

    Ok(Expr::new(
        ExprKind::Class {
            name,
            type_params,
            superclass,
            traits,
            fields,
            constructors,
            methods,
            constants,
            properties,
            derives: Vec::new(),
            decorators: Vec::new(),
            is_pub: false,
            is_sealed: false,
            is_abstract: false,
        },
        start_span,
    ))
}

fn parse_inheritance(state: &mut ParserState) -> Result<(Option<String>, Vec<String>)> {
    if !matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        return Ok((None, Vec::new()));
    }

    state.tokens.advance(); // consume ':'

    let superclass = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let mut name = name.clone();
        state.tokens.advance();

        // Parse generic type parameters if present (e.g., Parent<i32>)
        if matches!(state.tokens.peek(), Some((Token::Less, _))) {
            state.tokens.advance(); // consume '<'
            name.push('<');

            // Parse type parameters
            loop {
                let type_param = utils::parse_type(state)?;
                name.push_str(&format!("{:?}", type_param)); // Format type as string

                if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance();
                    name.push_str(", ");
                } else {
                    break;
                }
            }

            state.tokens.expect(&Token::Greater)?;
            name.push('>');
        }

        Some(name)
    } else {
        None
    };

    let mut traits = Vec::new();
    while matches!(state.tokens.peek(), Some((Token::Plus, _))) {
        state.tokens.advance();
        if let Some((Token::Identifier(trait_name), _)) = state.tokens.peek() {
            let mut name = trait_name.clone();
            state.tokens.advance();

            // Parse generic type parameters if present (e.g., Trait<i32>)
            if matches!(state.tokens.peek(), Some((Token::Less, _))) {
                state.tokens.advance(); // consume '<'
                name.push('<');

                // Parse type parameters
                loop {
                    let type_param = utils::parse_type(state)?;
                    name.push_str(&format!("{:?}", type_param)); // Format type as string

                    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                        state.tokens.advance();
                        name.push_str(", ");
                    } else {
                        break;
                    }
                }

                state.tokens.expect(&Token::Greater)?;
                name.push('>');
            }

            traits.push(name);
        } else {
            bail!("Expected trait name after '+'");
        }
    }

    Ok((superclass, traits))
}
fn parse_class_body(
    state: &mut ParserState,
) -> Result<(
    Vec<StructField>,
    Vec<Constructor>,
    Vec<ClassMethod>,
    Vec<ClassConstant>,
    Vec<ClassProperty>,
)> {
    state.tokens.expect(&Token::LeftBrace)?;

    let mut fields = Vec::new();
    let mut constructors = Vec::new();
    let mut methods = Vec::new();
    let mut constants = Vec::new();
    let mut properties = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        parse_class_member(
            state,
            &mut fields,
            &mut constructors,
            &mut methods,
            &mut constants,
            &mut properties,
        )?;
        consume_optional_separator(state);
    }

    state.tokens.expect(&Token::RightBrace)?;
    Ok((fields, constructors, methods, constants, properties))
}

/// Parse a single class member (field, constructor, method, constant, or property) - complexity: 9
fn parse_class_member(
    state: &mut ParserState,
    fields: &mut Vec<StructField>,
    constructors: &mut Vec<Constructor>,
    methods: &mut Vec<ClassMethod>,
    constants: &mut Vec<ClassConstant>,
    properties: &mut Vec<ClassProperty>,
) -> Result<()> {
    let decorators = parse_member_decorators(state)?;

    if try_parse_class_constant(state, constants)? {
        return Ok(());
    }

    if try_parse_class_property(state, properties)? {
        return Ok(());
    }

    validate_no_unsupported_features(state)?;

    if try_parse_operator_method(state, methods)? {
        return Ok(());
    }

    parse_member_and_dispatch(state, fields, constructors, methods, decorators)
}

fn parse_member_decorators(state: &mut ParserState) -> Result<Vec<Decorator>> {
    if matches!(state.tokens.peek(), Some((Token::At, _))) {
        parse_decorators(state)
    } else {
        Ok(Vec::new())
    }
}

fn try_parse_class_constant(
    state: &mut ParserState,
    constants: &mut Vec<ClassConstant>,
) -> Result<bool> {
    if matches!(state.tokens.peek(), Some((Token::Const, _))) {
        state.tokens.advance();
        let constant = parse_class_constant(state)?;
        constants.push(constant);
        Ok(true)
    } else {
        Ok(false)
    }
}

fn try_parse_class_property(
    state: &mut ParserState,
    properties: &mut Vec<ClassProperty>,
) -> Result<bool> {
    if matches!(state.tokens.peek(), Some((Token::Property, _))) {
        state.tokens.advance();
        let property = parse_class_property(state)?;
        properties.push(property);
        Ok(true)
    } else {
        Ok(false)
    }
}

fn validate_no_unsupported_features(state: &mut ParserState) -> Result<()> {
    if matches!(state.tokens.peek(), Some((Token::Impl, _))) {
        bail!("Impl blocks inside classes are not yet supported");
    }
    if matches!(state.tokens.peek(), Some((Token::Class, _))) {
        bail!("Nested classes are not yet supported");
    }
    Ok(())
}

fn try_parse_operator_method(
    state: &mut ParserState,
    methods: &mut Vec<ClassMethod>,
) -> Result<bool> {
    if matches!(state.tokens.peek(), Some((Token::Operator, _))) {
        state.tokens.advance();
        let operator_method = parse_operator_method(state)?;
        methods.push(operator_method);
        Ok(true)
    } else {
        Ok(false)
    }
}

fn parse_member_and_dispatch(
    state: &mut ParserState,
    fields: &mut Vec<StructField>,
    constructors: &mut Vec<Constructor>,
    methods: &mut Vec<ClassMethod>,
    decorators: Vec<Decorator>,
) -> Result<()> {
    let (visibility, is_mut) = parse_class_modifiers(state)?;
    let (is_static, is_override, is_final, is_abstract, is_async) = parse_member_flags(state)?;

    match state.tokens.peek() {
        Some((Token::Identifier(name), _)) if name == "new" || name == "init" => {
            parse_and_add_constructor(state, constructors, visibility)
        }
        Some((Token::Fun | Token::Fn, _)) => parse_and_add_method(
            state,
            methods,
            MethodModifiers {
                is_pub: visibility.is_public(),
                is_static,
                is_override,
                is_final,
                is_abstract,
                is_async,
            },
        ),
        // Support field declaration with 'let' keyword
        Some((Token::Let, _)) => {
            state.tokens.advance(); // consume 'let'
            parse_and_add_field(state, fields, visibility, is_mut, decorators)
        }
        Some((Token::Identifier(_), _)) if !is_static => {
            parse_and_add_field(state, fields, visibility, is_mut, decorators)
        }
        _ => bail!("Expected field, constructor, method, or constant in class body"),
    }
}

fn parse_and_add_constructor(
    state: &mut ParserState,
    constructors: &mut Vec<Constructor>,
    visibility: Visibility,
) -> Result<()> {
    validate_constructor_modifiers(false, false)?;
    let mut constructor = parse_constructor(state)?;
    constructor.is_pub = visibility.is_public();
    constructors.push(constructor);
    Ok(())
}

struct MethodModifiers {
    is_pub: bool,
    is_static: bool,
    is_override: bool,
    is_final: bool,
    is_abstract: bool,
    is_async: bool,
}

fn parse_and_add_method(
    state: &mut ParserState,
    methods: &mut Vec<ClassMethod>,
    modifiers: MethodModifiers,
) -> Result<()> {
    let mut method = parse_class_method(state, modifiers.is_abstract)?;
    apply_method_modifiers(&mut method, modifiers)?;
    methods.push(method);
    Ok(())
}

fn parse_and_add_field(
    state: &mut ParserState,
    fields: &mut Vec<StructField>,
    visibility: Visibility,
    is_mut: bool,
    decorators: Vec<Decorator>,
) -> Result<()> {
    let (field_name, field_type, default_value) = super::structs::parse_single_struct_field(state)?;
    fields.push(StructField {
        name: field_name,
        ty: field_type,
        visibility,
        is_mut,
        default_value,
        decorators,
    });
    Ok(())
}

/// Parse operator overloading: operator+(self, other: T) -> R { ... }
fn parse_operator_method(state: &mut ParserState) -> Result<ClassMethod> {
    // Parse the operator symbol (+, -, *, /, ==, etc.)
    let operator_name = match state.tokens.peek() {
        Some((Token::Plus, _)) => {
            state.tokens.advance();
            "add"
        }
        Some((Token::Minus, _)) => {
            state.tokens.advance();
            "sub"
        }
        Some((Token::Star, _)) => {
            state.tokens.advance();
            "mul"
        }
        Some((Token::Slash, _)) => {
            state.tokens.advance();
            "div"
        }
        Some((Token::EqualEqual, _)) => {
            state.tokens.advance();
            "eq"
        }
        Some((Token::NotEqual, _)) => {
            state.tokens.advance();
            "ne"
        }
        Some((Token::Less, _)) => {
            state.tokens.advance();
            "lt"
        }
        Some((Token::Greater, _)) => {
            state.tokens.advance();
            "gt"
        }
        Some((Token::LessEqual, _)) => {
            state.tokens.advance();
            "le"
        }
        Some((Token::GreaterEqual, _)) => {
            state.tokens.advance();
            "ge"
        }
        Some((Token::Percent, _)) => {
            state.tokens.advance();
            "rem"
        }
        Some((Token::LeftBracket, _)) => {
            state.tokens.advance();
            state.tokens.expect(&Token::RightBracket)?;
            "index"
        }
        _ => bail!("Expected operator symbol after 'operator' keyword"),
    };

    // Parse parameters
    let params = utils::parse_params(state)?;

    // Parse return type
    let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance();
        Some(utils::parse_type(state)?)
    } else {
        None
    };

    // Parse method body
    let body = if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
        Box::new(parse_expr_recursive(state)?)
    } else {
        bail!("Expected method body after operator signature")
    };

    Ok(ClassMethod {
        name: format!("op_{operator_name}"),
        params,
        return_type,
        body,
        is_pub: true,
        is_static: false,
        is_override: false,
        is_final: false,
        is_abstract: false,
        is_async: false,
        self_type: SelfType::Borrowed, // Most operators take &self
    })
}

/// Parse a decorator argument value (Integer, Float, String, or boolean)
fn parse_decorator_value(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Integer(n), _)) => {
            let v = n.clone();
            state.tokens.advance();
            Ok(v)
        }
        Some((Token::Float(f), _)) => {
            let v = f.to_string();
            state.tokens.advance();
            Ok(v)
        }
        Some((Token::String(s), _)) => {
            let v = s.clone();
            state.tokens.advance();
            Ok(v)
        }
        Some((Token::Identifier(id), _)) if id == "true" || id == "false" => {
            let v = id.clone();
            state.tokens.advance();
            Ok(v)
        }
        _ => bail!("Expected value after '=' in decorator argument"),
    }
}

/// Parse decorator: @Name or @Name("args", ...)
fn parse_decorator(state: &mut ParserState) -> Result<Decorator> {
    // Expect @ token
    state.tokens.expect(&Token::At)?;

    // Parse decorator name
    let name = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            name
        }
        _ => bail!("Expected decorator name after '@'"),
    };

    // Check for arguments
    let args = if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        state.tokens.advance(); // consume (
        let mut args = Vec::new();

        while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
            // Support both positional string literals and named arguments
            let arg = match state.tokens.peek() {
                Some((Token::String(s), _)) => {
                    let value = s.clone();
                    state.tokens.advance();
                    value
                }
                Some((Token::Identifier(key), _)) => {
                    // Check for named argument: key=value
                    let key = key.clone();
                    state.tokens.advance();

                    if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
                        state.tokens.advance(); // consume '='
                        let value = parse_decorator_value(state)?;
                        format!("{key}={value}")
                    } else {
                        // Just an identifier (positional)
                        key
                    }
                }
                _ => bail!("Expected argument in decorator"),
            };

            args.push(arg);

            // Check for comma
            if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                state.tokens.advance();
            } else if !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                bail!("Expected ',' or ')' in decorator arguments");
            }
        }

        state.tokens.expect(&Token::RightParen)?;
        args
    } else {
        Vec::new()
    };

    Ok(Decorator { name, args })
}

/// Parse decorators for classes/fields
pub(in crate::frontend::parser) fn parse_decorators(state: &mut ParserState) -> Result<Vec<Decorator>> {
    let mut decorators = Vec::new();

    while matches!(state.tokens.peek(), Some((Token::At, _))) {
        decorators.push(parse_decorator(state)?);
    }

    Ok(decorators)
}

/// Parse class constant: const NAME: TYPE = VALUE
fn parse_class_constant(state: &mut ParserState) -> Result<ClassConstant> {
    // Parse name
    let name = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            name
        }
        _ => bail!("Expected constant name after 'const'"),
    };

    // Expect colon
    state.tokens.expect(&Token::Colon)?;

    // Parse type
    let ty = utils::parse_type(state)?;

    // Expect equals
    state.tokens.expect(&Token::Equal)?;

    // Parse value expression
    let value = parse_expr_recursive(state)?;

    Ok(ClassConstant {
        name,
        ty,
        value,
        is_pub: true, // Constants are public by default in classes
    })
}

/// Parse class property: property NAME: TYPE { get => expr, set(param) => expr }
fn parse_class_property(state: &mut ParserState) -> Result<ClassProperty> {
    let name = parse_property_name(state)?;
    state.tokens.expect(&Token::Colon)?;
    let ty = utils::parse_type(state)?;
    state.tokens.expect(&Token::LeftBrace)?;

    let (getter, setter) = parse_property_accessors(state)?;

    state.tokens.expect(&Token::RightBrace)?;

    Ok(ClassProperty {
        name,
        ty,
        getter,
        setter,
        is_pub: true,
    })
}

fn parse_property_name(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            Ok(name)
        }
        _ => bail!("Expected property name after 'property'"),
    }
}

fn parse_property_accessors(
    state: &mut ParserState,
) -> Result<(Option<Box<Expr>>, Option<PropertySetter>)> {
    let mut getter = None;
    let mut setter = None;

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        match state.tokens.peek() {
            Some((Token::Identifier(keyword), _)) if keyword == "get" => {
                getter = Some(parse_property_getter(state)?);
            }
            Some((Token::Identifier(keyword), _)) if keyword == "set" => {
                setter = Some(parse_property_setter(state)?);
            }
            _ => bail!("Expected 'get' or 'set' in property body"),
        }

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }

    Ok((getter, setter))
}

fn parse_property_getter(state: &mut ParserState) -> Result<Box<Expr>> {
    state.tokens.advance(); // consume 'get'
    state.tokens.expect(&Token::FatArrow)?;
    let body = parse_expr_recursive(state)?;
    Ok(Box::new(body))
}

fn parse_property_setter(state: &mut ParserState) -> Result<PropertySetter> {
    state.tokens.advance(); // consume 'set'
    state.tokens.expect(&Token::LeftParen)?;

    let param_name = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            name
        }
        _ => bail!("Expected parameter name for setter"),
    };

    state.tokens.expect(&Token::RightParen)?;
    state.tokens.expect(&Token::FatArrow)?;
    let body = parse_expr_recursive(state)?;

    Ok(PropertySetter {
        param_name,
        body: Box::new(body),
    })
}

/// Parse visibility modifiers (pub, private, protected, mut) - complexity: 4
fn parse_class_modifiers(state: &mut ParserState) -> Result<(Visibility, bool)> {
    let mut visibility = try_parse_visibility_modifier(state)?;
    let is_mut = try_parse_mut_modifier(state);

    // Also check reverse order: mut pub/private/protected
    if matches!(visibility, Visibility::Private) {
        let second_visibility = try_parse_visibility_modifier(state)?;
        if !matches!(second_visibility, Visibility::Private) {
            visibility = second_visibility;
        }
    }

    Ok((visibility, is_mut))
}

fn try_parse_visibility_modifier(state: &mut ParserState) -> Result<Visibility> {
    match state.tokens.peek() {
        Some((Token::Private, _)) => {
            state.tokens.advance();
            Ok(Visibility::Private)
        }
        Some((Token::Protected, _)) => {
            state.tokens.advance();
            Ok(Visibility::Protected)
        }
        Some((Token::Pub, _)) => {
            state.tokens.advance();
            parse_pub_scope_modifier(state)
        }
        _ => Ok(Visibility::Private),
    }
}

fn parse_pub_scope_modifier(state: &mut ParserState) -> Result<Visibility> {
    if !matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        return Ok(Visibility::Public);
    }

    state.tokens.advance(); // consume (
    let visibility = match state.tokens.peek() {
        Some((Token::Crate, _)) => {
            state.tokens.advance();
            Visibility::PubCrate
        }
        Some((Token::Super, _)) => {
            state.tokens.advance();
            Visibility::PubSuper
        }
        Some((Token::Identifier(scope), _)) => {
            let scope = scope.clone();
            state.tokens.advance();
            state.tokens.expect(&Token::RightParen)?;
            bail!("Unsupported visibility scope: pub({scope}) - only pub(crate) and pub(super) are supported");
        }
        _ => bail!("Expected 'crate', 'super', or identifier after 'pub('"),
    };
    state.tokens.expect(&Token::RightParen)?;
    Ok(visibility)
}

fn try_parse_mut_modifier(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

/// Parse member flags (static, override) - complexity: 4
fn parse_member_flags(state: &mut ParserState) -> Result<(bool, bool, bool, bool, bool)> {
    let is_static = matches!(state.tokens.peek(), Some((Token::Static, _)));
    if is_static {
        state.tokens.advance();
    }

    let is_override = matches!(state.tokens.peek(), Some((Token::Override, _)));
    if is_override {
        state.tokens.advance();
    }

    let is_final = matches!(state.tokens.peek(), Some((Token::Final, _)));
    if is_final {
        state.tokens.advance();
    }

    let is_abstract = matches!(state.tokens.peek(), Some((Token::Abstract, _)));
    if is_abstract {
        state.tokens.advance();
    }

    let is_async = matches!(state.tokens.peek(), Some((Token::Async, _)));
    if is_async {
        state.tokens.advance();
    }

    Ok((is_static, is_override, is_final, is_abstract, is_async))
}

/// Validate constructor modifiers - complexity: 2
fn validate_constructor_modifiers(is_static: bool, is_override: bool) -> Result<()> {
    if is_static {
        bail!("Constructors cannot be static");
    }
    if is_override {
        bail!("Constructors cannot be override");
    }
    Ok(())
}

/// Apply modifiers to method - complexity: 3
fn apply_method_modifiers(method: &mut ClassMethod, modifiers: MethodModifiers) -> Result<()> {
    method.is_pub = modifiers.is_pub;
    method.is_static = modifiers.is_static;
    method.is_override = modifiers.is_override;
    method.is_final = modifiers.is_final;
    method.is_abstract = modifiers.is_abstract;
    method.is_async = modifiers.is_async;

    if modifiers.is_static {
        method.self_type = SelfType::None;
        if modifiers.is_override {
            bail!("Static methods cannot be override");
        }
    }
    if modifiers.is_final && modifiers.is_override {
        bail!("Methods cannot be both final and override");
    }
    if modifiers.is_abstract && modifiers.is_final {
        bail!("Methods cannot be both abstract and final");
    }
    if modifiers.is_abstract && modifiers.is_static {
        bail!("Static methods cannot be abstract");
    }
    Ok(())
}

/// Consume optional separator - complexity: 1
fn consume_optional_separator(state: &mut ParserState) {
    if matches!(
        state.tokens.peek(),
        Some((Token::Comma | Token::Semicolon, _))
    ) {
        state.tokens.advance();
    }
}

/// Parse constructor: new [name](params) { body } - complexity: <10
/// Supports named constructors like: new square(size)
/// Expect 'new' keyword for constructor
/// Complexity: 2 (Toyota Way: <10 ✓)
fn expect_new_keyword(state: &mut ParserState) -> Result<()> {
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        if name == "new" || name == "init" {
            state.tokens.advance();
            Ok(())
        } else {
            bail!("Expected 'new' or 'init' keyword");
        }
    } else {
        bail!("Expected 'new' or 'init' keyword");
    }
}

/// Parse optional constructor name (for named constructors)
/// Complexity: 4 (Toyota Way: <10 ✓)
fn parse_optional_constructor_name(state: &mut ParserState) -> Option<String> {
    if !matches!(state.tokens.peek(), Some((Token::Identifier(_), _))) {
        return None;
    }

    // Peek ahead to see if next is identifier followed by (
    let saved_pos = state.tokens.position();
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        // Check if followed by (
        if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
            // This is a named constructor
            Some(name)
        } else {
            // Not a named constructor, restore position
            state.tokens.set_position(saved_pos);
            None
        }
    } else {
        None
    }
}

/// Parse constructor: new(...) or new name(...)
/// Complexity: 4 (Toyota Way: <10 ✓) [Reduced from 10]
fn parse_constructor(state: &mut ParserState) -> Result<Constructor> {
    // Expect 'new' keyword
    expect_new_keyword(state)?;

    // Check for optional constructor name (for named constructors)
    let constructor_name = parse_optional_constructor_name(state);

    // Parse parameter list (params)
    let params = utils::parse_params(state)?;

    // Parse optional return type (usually omitted for constructors)
    let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance();
        Some(utils::parse_type(state)?)
    } else {
        None
    };

    // Parse body { ... }
    let body = Box::new(parse_expr_recursive(state)?);

    Ok(Constructor {
        name: constructor_name,
        params,
        return_type,
        body,
        is_pub: false, // Will be set by class body parsing
    })
}

/// Parse class method: fn `method_name(self_param`, `other_params`) -> `return_type` { body } - complexity: <10
fn parse_class_method(state: &mut ParserState, is_abstract: bool) -> Result<ClassMethod> {
    // Expect 'fun' or 'fn' keyword
    match state.tokens.peek() {
        Some((Token::Fun, _)) => {
            state.tokens.advance();
        }
        Some((Token::Fn, _)) => {
            state.tokens.advance();
        }
        _ => bail!("Expected 'fun' or 'fn' keyword for method definition"),
    }

    // Parse method name (accept keywords that can be method names)
    let method_name = match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();
            name
        }
        // Allow keyword method names (impl From has fn from method)
        Some((Token::From, _)) => {
            state.tokens.advance();
            "from".to_string()
        }
        Some((Token::Default, _)) => {
            state.tokens.advance();
            "default".to_string()
        }
        _ => bail!("Expected method name after 'fn'"),
    };

    // Parse parameter list starting with self parameter
    let params = utils::parse_params(state)?;

    // Determine self type from first parameter
    let self_type = determine_self_type_from_params(&params);

    // Parse optional return type
    let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance();
        Some(utils::parse_type(state)?)
    } else {
        None
    };

    // Parse method body (abstract methods have no body)
    let body = if is_abstract {
        // Abstract methods don't have a body - use empty block as placeholder
        Box::new(Expr::new(ExprKind::Block(Vec::new()), Span::default()))
    } else {
        Box::new(parse_expr_recursive(state)?)
    };

    Ok(ClassMethod {
        name: method_name,
        params,
        return_type,
        body,
        is_pub: false, // Will be set by class body parsing
        is_static: matches!(self_type, SelfType::None),
        is_override: false, // Will be set by class body parsing
        is_final: false,    // Will be set by class body parsing
        is_abstract: false, // Will be set by class body parsing
        is_async: false,    // Will be set by class body parsing
        self_type,
    })
}

/// Determine self type from method parameters
fn determine_self_type_from_params(params: &[Param]) -> SelfType {
    if !params.is_empty() && params[0].name() == "self" {
        use crate::frontend::ast::TypeKind;
        match &params[0].ty.kind {
            TypeKind::Reference { is_mut: true, .. } => SelfType::MutBorrowed,
            TypeKind::Reference { is_mut: false, .. } => SelfType::Borrowed,
            _ => SelfType::Owned,
        }
    } else {
        SelfType::None // No self parameter = static method
    }
}


#[cfg(test)]
mod tests {
    
    use crate::frontend::parser::Parser;

    #[test]
    fn test_basic_class() {
        let code = "class MyClass { }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Basic class should parse");
    }

    #[test]
    fn test_class_with_fields() {
        let code = "class Point { x: f64 y: f64 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Class with fields should parse");
    }

    #[test]
    fn test_class_with_inheritance() {
        let code = "class Child : Parent { }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Class with inheritance should parse");
    }

    #[test]
    fn test_class_with_traits() {
        let code = "class MyClass : ParentClass + Trait1 + Trait2 { }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Class with traits should parse");
    }

    #[test]
    fn test_class_with_constructor() {
        let code = "class Point { new(x: f64, y: f64) { self.x = x; self.y = y } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Class with constructor should parse");
    }

    #[test]
    fn test_class_with_method() {
        let code = "class Point { fun distance(&self) -> f64 { 0.0 } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Class with method should parse");
    }

    #[test]
    fn test_generic_class() {
        let code = "class Container<T> { value: T }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Generic class should parse");
    }
}
