//! MIR Builder - Provides a convenient API for constructing MIR

use super::types::*;
use std::collections::HashMap;

/// Builder for constructing MIR programs
pub struct MirBuilder {
    /// Current function being built
    current_function: Option<Function>,
    /// Next local variable ID
    next_local: usize,
    /// Next block ID
    next_block: usize,
    /// Mapping from names to locals
    local_map: HashMap<String, Local>,
}

impl MirBuilder {
    /// Create a new MIR builder
    pub fn new() -> Self {
        Self {
            current_function: None,
            next_local: 0,
            next_block: 0,
            local_map: HashMap::new(),
        }
    }

    /// Start building a new function
    pub fn start_function(&mut self, name: String, return_ty: Type) -> &mut Self {
        self.current_function = Some(Function {
            name,
            params: Vec::new(),
            return_ty,
            locals: Vec::new(),
            blocks: Vec::new(),
            entry_block: BlockId(0),
        });
        self.next_local = 0;
        self.next_block = 0;
        self.local_map.clear();
        self
    }

    /// Add a parameter to the current function
    pub fn add_param(&mut self, name: String, ty: Type) -> Local {
        let local = self.alloc_local(ty.clone(), true, Some(name.clone()));
        if let Some(ref mut func) = self.current_function {
            func.params.push(local);
        }
        self.local_map.insert(name, local);
        local
    }

    /// Allocate a new local variable
    pub fn alloc_local(&mut self, ty: Type, mutable: bool, name: Option<String>) -> Local {
        let id = Local(self.next_local);
        self.next_local += 1;
        
        let decl = LocalDecl {
            id,
            ty,
            mutable,
            name: name.clone(),
        };
        
        if let Some(ref mut func) = self.current_function {
            func.locals.push(decl);
        }
        
        if let Some(n) = name {
            self.local_map.insert(n, id);
        }
        
        id
    }

    /// Get a local by name
    pub fn get_local(&self, name: &str) -> Option<Local> {
        self.local_map.get(name).copied()
    }

    /// Create a new basic block
    pub fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block);
        self.next_block += 1;
        
        if let Some(ref mut func) = self.current_function {
            func.blocks.push(BasicBlock {
                id,
                statements: Vec::new(),
                terminator: Terminator::Unreachable,
            });
        }
        
        id
    }

    /// Get a mutable reference to a block
    pub fn block_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.current_function
            .as_mut()
            .and_then(|f| f.blocks.get_mut(id.0))
    }

    /// Add a statement to a block
    pub fn push_statement(&mut self, block: BlockId, stmt: Statement) {
        if let Some(bb) = self.block_mut(block) {
            bb.statements.push(stmt);
        }
    }

    /// Set the terminator for a block
    pub fn set_terminator(&mut self, block: BlockId, term: Terminator) {
        if let Some(bb) = self.block_mut(block) {
            bb.terminator = term;
        }
    }

    /// Finish building the current function
    pub fn finish_function(&mut self) -> Option<Function> {
        self.current_function.take()
    }

    /// Build an assignment statement
    pub fn assign(&mut self, block: BlockId, place: Place, rvalue: Rvalue) {
        self.push_statement(block, Statement::Assign(place, rvalue));
    }

    /// Build a storage live statement
    pub fn storage_live(&mut self, block: BlockId, local: Local) {
        self.push_statement(block, Statement::StorageLive(local));
    }

    /// Build a storage dead statement
    pub fn storage_dead(&mut self, block: BlockId, local: Local) {
        self.push_statement(block, Statement::StorageDead(local));
    }

    /// Build a goto terminator
    pub fn goto(&mut self, block: BlockId, target: BlockId) {
        self.set_terminator(block, Terminator::Goto(target));
    }

    /// Build an if terminator
    pub fn branch(&mut self, block: BlockId, cond: Operand, then_block: BlockId, else_block: BlockId) {
        self.set_terminator(block, Terminator::If {
            condition: cond,
            then_block,
            else_block,
        });
    }

    /// Build a return terminator
    pub fn return_(&mut self, block: BlockId, value: Option<Operand>) {
        self.set_terminator(block, Terminator::Return(value));
    }

    /// Build a call terminator
    pub fn call_term(&mut self, 
                     block: BlockId, 
                     func: Operand, 
                     args: Vec<Operand>, 
                     dest: Option<(Place, BlockId)>) {
        self.set_terminator(block, Terminator::Call {
            func,
            args,
            destination: dest,
        });
    }

    /// Build a switch terminator
    pub fn switch(&mut self, 
                  block: BlockId,
                  discriminant: Operand,
                  targets: Vec<(Constant, BlockId)>,
                  default: Option<BlockId>) {
        self.set_terminator(block, Terminator::Switch {
            discriminant,
            targets,
            default,
        });
    }
}

/// Helper functions for creating common patterns
impl MirBuilder {
    /// Create a binary operation and assign to a local
    pub fn binary_op(&mut self, 
                     block: BlockId,
                     dest: Local,
                     op: BinOp,
                     left: Operand,
                     right: Operand) {
        let rvalue = Rvalue::BinaryOp(op, left, right);
        self.assign(block, Place::Local(dest), rvalue);
    }

    /// Create a unary operation and assign to a local
    pub fn unary_op(&mut self,
                    block: BlockId,
                    dest: Local,
                    op: UnOp,
                    operand: Operand) {
        let rvalue = Rvalue::UnaryOp(op, operand);
        self.assign(block, Place::Local(dest), rvalue);
    }

    /// Create a function call and assign result to a local
    pub fn call(&mut self,
                block: BlockId,
                dest: Local,
                func: Operand,
                args: Vec<Operand>) -> BlockId {
        let next_block = self.new_block();
        self.call_term(block, func, args, Some((Place::Local(dest), next_block)));
        next_block
    }

    /// Create a cast and assign to a local
    pub fn cast(&mut self,
                block: BlockId,
                dest: Local,
                kind: CastKind,
                operand: Operand,
                target_ty: Type) {
        let rvalue = Rvalue::Cast(kind, operand, target_ty);
        self.assign(block, Place::Local(dest), rvalue);
    }

    /// Create a reference and assign to a local
    pub fn ref_(&mut self,
                block: BlockId,
                dest: Local,
                mutability: Mutability,
                place: Place) {
        let rvalue = Rvalue::Ref(mutability, place);
        self.assign(block, Place::Local(dest), rvalue);
    }

    /// Move a value from one place to another
    pub fn move_(&mut self,
                 block: BlockId,
                 dest: Place,
                 source: Place) {
        let rvalue = Rvalue::Use(Operand::Move(source));
        self.assign(block, dest, rvalue);
    }

    /// Copy a value from one place to another
    pub fn copy(&mut self,
                block: BlockId,
                dest: Place,
                source: Place) {
        let rvalue = Rvalue::Use(Operand::Copy(source));
        self.assign(block, dest, rvalue);
    }

    /// Assign a constant to a place
    pub fn const_(&mut self,
                  block: BlockId,
                  dest: Place,
                  constant: Constant) {
        let rvalue = Rvalue::Use(Operand::Constant(constant));
        self.assign(block, dest, rvalue);
    }
}

impl Default for MirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_build_simple_function() {
        let mut builder = MirBuilder::new();
        
        // Build: fn add(a: i32, b: i32) -> i32 { a + b }
        builder.start_function("add".to_string(), Type::I32);
        
        let a = builder.add_param("a".to_string(), Type::I32);
        let b = builder.add_param("b".to_string(), Type::I32);
        
        let entry = builder.new_block();
        let result = builder.alloc_local(Type::I32, false, Some("result".to_string()));
        
        builder.storage_live(entry, result);
        builder.binary_op(
            entry,
            result,
            BinOp::Add,
            Operand::Copy(Place::Local(a)),
            Operand::Copy(Place::Local(b))
        );
        builder.return_(entry, Some(Operand::Move(Place::Local(result))));
        
        let func = builder.finish_function().unwrap();
        assert_eq!(func.name, "add");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.blocks.len(), 1);
    }

    #[test]
    fn test_build_if_else() {
        let mut builder = MirBuilder::new();
        
        // Build: fn abs(x: i32) -> i32 { if x < 0 { -x } else { x } }
        builder.start_function("abs".to_string(), Type::I32);
        
        let x = builder.add_param("x".to_string(), Type::I32);
        
        let entry = builder.new_block();
        let then_block = builder.new_block();
        let else_block = builder.new_block();
        let merge_block = builder.new_block();
        
        // Check if x < 0
        let cond = builder.alloc_local(Type::Bool, false, None);
        builder.binary_op(
            entry,
            cond,
            BinOp::Lt,
            Operand::Copy(Place::Local(x)),
            Operand::Constant(Constant::Int(0, Type::I32))
        );
        builder.branch(entry, Operand::Copy(Place::Local(cond)), then_block, else_block);
        
        // Then branch: -x
        let neg_x = builder.alloc_local(Type::I32, false, None);
        builder.unary_op(then_block, neg_x, UnOp::Neg, Operand::Copy(Place::Local(x)));
        builder.goto(then_block, merge_block);
        
        // Else branch: x
        builder.goto(else_block, merge_block);
        
        // Merge and return
        builder.return_(merge_block, Some(Operand::Copy(Place::Local(x))));
        
        let func = builder.finish_function().unwrap();
        assert_eq!(func.blocks.len(), 4);
    }
}