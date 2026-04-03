# Sub-spec: Self-Hosting Compiler — Phases 2-3: Parser & Type System

**Parent:** [ruchy-self-hosting-spec.md](../ruchy-self-hosting-spec.md) Phases 2-3

---

## Phase 2: Parser Implementation (COMPLETED - Weeks 2-3)

### Core Parser Structure
```ruchy
module ruchy::parser {
  use ruchy::lexer::{Token, TokenStream};
  use ruchy::ast::*;
  
  struct Parser {
    tokens: TokenStream,
    errors: Vec<ParseError>,
    recovery_mode: bool,
  }
  
  impl Parser {
    fn new(source: String) -> Result<Parser, LexError> {
      Ok(Parser {
        tokens: TokenStream::new(source)?,
        errors: Vec::new(),
        recovery_mode: false,
      })
    }
    
    fn parse(&mut self) -> Result<Program, Vec<ParseError>> {
      let mut items = Vec::new();
      
      while self.tokens.peek() != Some(&Token::Eof) {
        match self.parse_item() {
          Ok(item) => items.push(item),
          Err(e) => {
            self.errors.push(e);
            self.recover();
          }
        }
      }
      
      if self.errors.is_empty() {
        Ok(Program { items })
      } else {
        Err(self.errors.clone())
      }
    }
    
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
      self.parse_expr_with_precedence(0)
    }
    
    fn parse_expr_with_precedence(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
      let mut left = self.parse_prefix()?;
      
      while let Some(token) = self.tokens.peek() {
        let prec = self.get_precedence(token);
        if prec < min_prec {
          break;
        }
        
        let op = self.tokens.advance().unwrap();
        let right = self.parse_expr_with_precedence(prec + 1)?;
        left = Expr::Binary {
          left: Box::new(left),
          op: self.token_to_binop(op)?,
          right: Box::new(right),
        };
      }
      
      Ok(left)
    }
    
    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
      match self.tokens.advance() {
        Some(Token::Int(n)) => Ok(Expr::Literal(Literal::Int(n))),
        Some(Token::Float(f)) => Ok(Expr::Literal(Literal::Float(f))),
        Some(Token::String(s)) => Ok(Expr::Literal(Literal::String(s))),
        Some(Token::Bool(b)) => Ok(Expr::Literal(Literal::Bool(b))),
        Some(Token::Identifier(name)) => Ok(Expr::Identifier(name)),
        
        Some(Token::LeftParen) => {
          let expr = self.parse_expr()?;
          self.tokens.expect(Token::RightParen)?;
          Ok(expr)
        }
        
        Some(Token::LeftBracket) => self.parse_list(),
        Some(Token::LeftBrace) => self.parse_block(),
        
        Some(Token::If) => self.parse_if(),
        Some(Token::Match) => self.parse_match(),
        Some(Token::For) => self.parse_for(),
        Some(Token::While) => self.parse_while(),
        Some(Token::Fn) => self.parse_function(),
        Some(Token::Let) => self.parse_let(),
        
        Some(Token::Minus) => {
          let expr = self.parse_prefix()?;
          Ok(Expr::Unary { op: UnaryOp::Neg, operand: Box::new(expr) })
        }
        
        Some(Token::Not) => {
          let expr = self.parse_prefix()?;
          Ok(Expr::Unary { op: UnaryOp::Not, operand: Box::new(expr) })
        }
        
        Some(token) => Err(ParseError::UnexpectedToken { found: token }),
        None => Err(ParseError::UnexpectedEof),
      }
    }
    
    fn get_precedence(&self, token: &Token) -> u8 {
      match token {
        Token::Or => 1,
        Token::And => 2,
        Token::Equal | Token::NotEqual => 3,
        Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => 4,
        Token::BitOr => 5,
        Token::BitXor => 6,
        Token::BitAnd => 7,
        Token::LeftShift | Token::RightShift => 8,
        Token::Plus | Token::Minus => 9,
        Token::Star | Token::Slash | Token::Percent => 10,
        Token::Power => 11,
        Token::Dot => 12,
        _ => 0,
      }
    }
  }
}
```

## Phase 3: Type System (COMPLETED - Weeks 4-6)

### Type Inference Engine
```ruchy
module ruchy::types {
  use std::collections::HashMap;
  
  enum Type {
    Int,
    Float,
    Bool,
    String,
    Char,
    Unit,
    
    List(Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    
    Function {
      params: Vec<Type>,
      return_type: Box<Type>,
    },
    
    Struct {
      name: String,
      fields: HashMap<String, Type>,
    },
    
    Enum {
      name: String,
      variants: HashMap<String, Option<Type>>,
    },
    
    TypeVar(u32),
    Generic(String, Vec<TypeConstraint>),
  }
  
  struct TypeContext {
    bindings: HashMap<String, TypeScheme>,
    type_vars: u32,
    substitutions: HashMap<u32, Type>,
  }
  
  struct TypeScheme {
    quantified: Vec<u32>,
    ty: Type,
  }
  
  impl TypeContext {
    fn infer_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
      match expr {
        Expr::Literal(lit) => self.infer_literal(lit),
        Expr::Identifier(name) => self.lookup(name),
        Expr::Binary { left, op, right } => {
          let left_ty = self.infer_expr(left)?;
          let right_ty = self.infer_expr(right)?;
          self.infer_binary_op(op, left_ty, right_ty)
        }
        Expr::If { condition, then_branch, else_branch } => {
          let cond_ty = self.infer_expr(condition)?;
          self.unify(cond_ty, Type::Bool)?;
          
          let then_ty = self.infer_expr(then_branch)?;
          let else_ty = self.infer_expr(else_branch)?;
          self.unify(then_ty.clone(), else_ty)?;
          
          Ok(then_ty)
        }
        Expr::Let { pattern, value, body } => {
          let value_ty = self.infer_expr(value)?;
          self.bind_pattern(pattern, value_ty)?;
          self.infer_expr(body)
        }
        Expr::Function { params, body, .. } => {
          let param_types = params.iter()
            .map(|p| self.fresh_type_var())
            .collect::<Vec<_>>();
          
          for (param, ty) in params.iter().zip(&param_types) {
            self.bind(param.name.clone(), ty.clone());
          }
          
          let body_ty = self.infer_expr(body)?;
          Ok(Type::Function {
            params: param_types,
            return_type: Box::new(body_ty),
          })
        }
        Expr::Call { func, args } => {
          let func_ty = self.infer_expr(func)?;
          let arg_types = args.iter()
            .map(|arg| self.infer_expr(arg))
            .collect::<Result<Vec<_>, _>>()?;
          
          let return_ty = self.fresh_type_var();
          let expected = Type::Function {
            params: arg_types,
            return_type: Box::new(return_ty.clone()),
          };
          
          self.unify(func_ty, expected)?;
          Ok(return_ty)
        }
        _ => todo!("Implement remaining expression types"),
      }
    }
    
    fn unify(&mut self, ty1: Type, ty2: Type) -> Result<(), TypeError> {
      match (ty1, ty2) {
        (Type::TypeVar(v1), Type::TypeVar(v2)) if v1 == v2 => Ok(()),
        (Type::TypeVar(v), ty) | (ty, Type::TypeVar(v)) => {
          if self.occurs_check(v, &ty) {
            return Err(TypeError::InfiniteType);
          }
          self.substitutions.insert(v, ty);
          Ok(())
        }
        (Type::Int, Type::Int) => Ok(()),
        (Type::Float, Type::Float) => Ok(()),
        (Type::Bool, Type::Bool) => Ok(()),
        (Type::String, Type::String) => Ok(()),
        (Type::List(t1), Type::List(t2)) => self.unify(*t1, *t2),
        (Type::Function { params: p1, return_type: r1 },
         Type::Function { params: p2, return_type: r2 }) => {
          if p1.len() != p2.len() {
            return Err(TypeError::ArityMismatch);
          }
          for (t1, t2) in p1.into_iter().zip(p2) {
            self.unify(t1, t2)?;
          }
          self.unify(*r1, *r2)
        }
        (ty1, ty2) => Err(TypeError::Mismatch { expected: ty1, found: ty2 }),
      }
    }
    
    fn fresh_type_var(&mut self) -> Type {
      let var = self.type_vars;
      self.type_vars += 1;
      Type::TypeVar(var)
    }
  }
}
