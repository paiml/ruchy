//! MIR Builder - Provides a convenient API for constructing MIR
use super::types::{
    BasicBlock, BinOp, BlockId, CastKind, Constant, Function, Local, LocalDecl, Mutability,
    Operand, Place, Rvalue, Statement, Terminator, Type, UnOp,
};
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
    #[must_use]
/// # Examples
/// 
/// ```
/// use ruchy::middleend::mir::builder::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn new() -> Self {
        Self {
            current_function: None,
            next_local: 0,
            next_block: 0,
            local_map: HashMap::new(),
        }
    }
    /// Start building a new function
/// Start building a new function
///
/// # Examples
///
/// ```ignore
/// use ruchy::middleend::mir::builder::MirBuilder;
/// use ruchy::middleend::types::Type;
/// let mut builder = MirBuilder::new();
/// builder.start_function("main".to_string(), Type::Unit);
/// ```
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
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::MirBuilder;
/// let mut builder = MirBuilder::new();
/// // add_param requires parameters
/// ```
pub fn add_param(&mut self, name: String, ty: Type) -> Local {
        let local = self.alloc_local(ty, true, Some(name.clone()));
        if let Some(ref mut func) = self.current_function {
            func.params.push(local);
        }
        self.local_map.insert(name, local);
        local
    }
    /// Allocate a new local variable
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::alloc_local;
/// 
/// let result = alloc_local(true);
/// assert_eq!(result, Ok(true));
/// ```
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
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::get_local;
/// 
/// let result = get_local("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn get_local(&self, name: &str) -> Option<Local> {
        self.local_map.get(name).copied()
    }
    /// Create a new basic block
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::new_block;
/// 
/// let result = new_block(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::block_mut;
/// 
/// let result = block_mut(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn block_mut(&mut self, id: BlockId) -> Option<&mut BasicBlock> {
        self.current_function
            .as_mut()
            .and_then(|f| f.blocks.get_mut(id.0))
    }
    /// Add a statement to a block
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::push_statement;
/// 
/// let result = push_statement(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn push_statement(&mut self, block: BlockId, stmt: Statement) {
        if let Some(bb) = self.block_mut(block) {
            bb.statements.push(stmt);
        }
    }
    /// Set the terminator for a block
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::set_terminator;
/// 
/// let result = set_terminator(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn set_terminator(&mut self, block: BlockId, term: Terminator) {
        if let Some(bb) = self.block_mut(block) {
            bb.terminator = term;
        }
    }
    /// Finish building the current function
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::finish_function;
/// 
/// let result = finish_function(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn finish_function(&mut self) -> Option<Function> {
        self.current_function.take()
    }
    /// Build an assignment statement
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::assign;
/// 
/// let result = assign(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn assign(&mut self, block: BlockId, place: Place, rvalue: Rvalue) {
        self.push_statement(block, Statement::Assign(place, rvalue));
    }
    /// Build a storage live statement
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::storage_live;
/// 
/// let result = storage_live(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn storage_live(&mut self, block: BlockId, local: Local) {
        self.push_statement(block, Statement::StorageLive(local));
    }
    /// Build a storage dead statement
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::storage_dead;
/// 
/// let result = storage_dead(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn storage_dead(&mut self, block: BlockId, local: Local) {
        self.push_statement(block, Statement::StorageDead(local));
    }
    /// Build a goto terminator
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::goto;
/// 
/// let result = goto(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn goto(&mut self, block: BlockId, target: BlockId) {
        self.set_terminator(block, Terminator::Goto(target));
    }
    /// Build an if terminator
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::branch;
/// 
/// let result = branch(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn branch(
        &mut self,
        block: BlockId,
        cond: Operand,
        then_block: BlockId,
        else_block: BlockId,
    ) {
        self.set_terminator(
            block,
            Terminator::If {
                condition: cond,
                then_block,
                else_block,
            },
        );
    }
    /// Build a return terminator
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::return_;
/// 
/// let result = return_(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn return_(&mut self, block: BlockId, value: Option<Operand>) {
        self.set_terminator(block, Terminator::Return(value));
    }
    /// Build a call terminator
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::call_term;
/// 
/// let result = call_term(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn call_term(
        &mut self,
        block: BlockId,
        func: Operand,
        args: Vec<Operand>,
        dest: Option<(Place, BlockId)>,
    ) {
        self.set_terminator(
            block,
            Terminator::Call {
                func,
                args,
                destination: dest,
            },
        );
    }
    /// Build a switch terminator
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::switch;
/// 
/// let result = switch(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn switch(
        &mut self,
        block: BlockId,
        discriminant: Operand,
        targets: Vec<(Constant, BlockId)>,
        default: Option<BlockId>,
    ) {
        self.set_terminator(
            block,
            Terminator::Switch {
                discriminant,
                targets,
                default,
            },
        );
    }
}
/// Helper functions for creating common patterns
impl MirBuilder {
    /// Create a binary operation and assign to a local
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::binary_op;
/// 
/// let result = binary_op(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn binary_op(
        &mut self,
        block: BlockId,
        dest: Local,
        op: BinOp,
        left: Operand,
        right: Operand,
    ) {
        let rvalue = Rvalue::BinaryOp(op, left, right);
        self.assign(block, Place::Local(dest), rvalue);
    }
    /// Create a unary operation and assign to a local
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::unary_op;
/// 
/// let result = unary_op(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn unary_op(&mut self, block: BlockId, dest: Local, op: UnOp, operand: Operand) {
        let rvalue = Rvalue::UnaryOp(op, operand);
        self.assign(block, Place::Local(dest), rvalue);
    }
    /// Create a function call and assign result to a local
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::call;
/// 
/// let result = call(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn call(
        &mut self,
        block: BlockId,
        dest: Local,
        func: Operand,
        args: Vec<Operand>,
    ) -> BlockId {
        let next_block = self.new_block();
        self.call_term(block, func, args, Some((Place::Local(dest), next_block)));
        next_block
    }
    /// Create a cast and assign to a local
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::cast;
/// 
/// let result = cast(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn cast(
        &mut self,
        block: BlockId,
        dest: Local,
        kind: CastKind,
        operand: Operand,
        target_ty: Type,
    ) {
        let rvalue = Rvalue::Cast(kind, operand, target_ty);
        self.assign(block, Place::Local(dest), rvalue);
    }
    /// Create a reference and assign to a local
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::ref_;
/// 
/// let result = ref_(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn ref_(&mut self, block: BlockId, dest: Local, mutability: Mutability, place: Place) {
        let rvalue = Rvalue::Ref(mutability, place);
        self.assign(block, Place::Local(dest), rvalue);
    }
    /// Move a value from one place to another
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::move_;
/// 
/// let result = move_(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn move_(&mut self, block: BlockId, dest: Place, source: Place) {
        let rvalue = Rvalue::Use(Operand::Move(source));
        self.assign(block, dest, rvalue);
    }
    /// Copy a value from one place to another
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::copy;
/// 
/// let result = copy(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn copy(&mut self, block: BlockId, dest: Place, source: Place) {
        let rvalue = Rvalue::Use(Operand::Copy(source));
        self.assign(block, dest, rvalue);
    }
    /// Assign a constant to a place
/// # Examples
/// 
/// ```ignore
/// use ruchy::middleend::mir::builder::const_;
/// 
/// let result = const_(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn const_(&mut self, block: BlockId, dest: Place, constant: Constant) {
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

    /// Test basic MirBuilder creation and initialization
    #[test]
    fn test_new_builder() {
        let builder = MirBuilder::new();
        assert_eq!(builder.next_local, 0);
        assert_eq!(builder.next_block, 0);
        assert!(builder.local_map.is_empty());
        assert!(builder.current_function.is_none());
    }

    /// Test default implementation
    #[test]
    fn test_default_builder() {
        let builder1 = MirBuilder::new();
        let builder2 = MirBuilder::default();
        assert_eq!(builder1.next_local, builder2.next_local);
        assert_eq!(builder1.next_block, builder2.next_block);
    }

    /// Test function creation and parameter addition
    #[test]
    fn test_start_function() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Bool);

        let func = builder.current_function.as_ref().unwrap();
        assert_eq!(func.name, "test");
        assert_eq!(func.return_ty, Type::Bool);
        assert!(func.params.is_empty());
        assert!(func.locals.is_empty());
        assert!(func.blocks.is_empty());
        assert_eq!(func.entry_block, BlockId(0));
    }

    /// Test parameter addition with name mapping
    #[test]
    fn test_add_param() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);

        let param1 = builder.add_param("x".to_string(), Type::I32);
        let param2 = builder.add_param("y".to_string(), Type::F64);

        assert_eq!(param1, Local(0));
        assert_eq!(param2, Local(1));
        assert_eq!(builder.get_local("x"), Some(Local(0)));
        assert_eq!(builder.get_local("y"), Some(Local(1)));
        assert_eq!(builder.get_local("z"), None);

        let func = builder.current_function.as_ref().unwrap();
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.locals.len(), 2);
    }

    /// Test local variable allocation without names
    #[test]
    fn test_alloc_local_anonymous() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);

        let local1 = builder.alloc_local(Type::I32, false, None);
        let local2 = builder.alloc_local(Type::Bool, true, None);

        assert_eq!(local1, Local(0));
        assert_eq!(local2, Local(1));

        let func = builder.current_function.as_ref().unwrap();
        assert_eq!(func.locals.len(), 2);
        assert!(!func.locals[0].mutable);
        assert!(func.locals[1].mutable);
        assert!(func.locals[0].name.is_none());
        assert!(func.locals[1].name.is_none());
    }

    /// Test local variable allocation with names
    #[test]
    fn test_alloc_local_named() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);

        let local = builder.alloc_local(Type::String, true, Some("var".to_string()));
        assert_eq!(local, Local(0));
        assert_eq!(builder.get_local("var"), Some(Local(0)));

        let func = builder.current_function.as_ref().unwrap();
        assert_eq!(func.locals[0].name, Some("var".to_string()));
        assert_eq!(func.locals[0].ty, Type::String);
        assert!(func.locals[0].mutable);
    }

    /// Test basic block creation
    #[test]
    fn test_new_block() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);

        let block1 = builder.new_block();
        let block2 = builder.new_block();

        assert_eq!(block1, BlockId(0));
        assert_eq!(block2, BlockId(1));

        let func = builder.current_function.as_ref().unwrap();
        assert_eq!(func.blocks.len(), 2);
        assert_eq!(func.blocks[0].id, block1);
        assert_eq!(func.blocks[1].id, block2);
    }

    /// Test block mutation and access
    #[test]
    fn test_block_mut() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();

        // Test valid block access
        let block_ref = builder.block_mut(block);
        assert!(block_ref.is_some());
        assert_eq!(block_ref.unwrap().id, block);

        // Test invalid block access
        let invalid_block = builder.block_mut(BlockId(999));
        assert!(invalid_block.is_none());
    }

    /// Test statement addition to blocks
    #[test]
    fn test_push_statement() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();
        let local = builder.alloc_local(Type::I32, false, None);

        let stmt = Statement::StorageLive(local);
        builder.push_statement(block, stmt);

        let func = builder.current_function.as_ref().unwrap();
        assert_eq!(func.blocks[0].statements.len(), 1);
        assert!(matches!(func.blocks[0].statements[0], Statement::StorageLive(_)));
    }

    /// Test terminator setting
    #[test]
    fn test_set_terminator() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();

        builder.set_terminator(block, Terminator::Return(None));

        let func = builder.current_function.as_ref().unwrap();
        assert!(matches!(func.blocks[0].terminator, Terminator::Return(None)));
    }

    /// Test function completion
    #[test]
    fn test_finish_function() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::I32);

        let func = builder.finish_function();
        assert!(func.is_some());
        assert_eq!(func.unwrap().name, "test");
        assert!(builder.current_function.is_none());

        // Test finishing when no function is active
        let no_func = builder.finish_function();
        assert!(no_func.is_none());
    }

    /// Test assignment statement creation
    #[test]
    fn test_assign() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();
        let local = builder.alloc_local(Type::I32, false, None);

        let rvalue = Rvalue::Use(Operand::Constant(Constant::Int(42, Type::I32)));
        builder.assign(block, Place::Local(local), rvalue);

        let func = builder.current_function.as_ref().unwrap();
        assert_eq!(func.blocks[0].statements.len(), 1);
        assert!(matches!(func.blocks[0].statements[0], Statement::Assign(_, _)));
    }

    /// Test storage live statement
    #[test]
    fn test_storage_live() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();
        let local = builder.alloc_local(Type::I32, false, None);

        builder.storage_live(block, local);

        let func = builder.current_function.as_ref().unwrap();
        assert!(matches!(func.blocks[0].statements[0], Statement::StorageLive(_)));
    }

    /// Test storage dead statement
    #[test]
    fn test_storage_dead() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();
        let local = builder.alloc_local(Type::I32, false, None);

        builder.storage_dead(block, local);

        let func = builder.current_function.as_ref().unwrap();
        assert!(matches!(func.blocks[0].statements[0], Statement::StorageDead(_)));
    }

    /// Test goto terminator
    #[test]
    fn test_goto() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block1 = builder.new_block();
        let block2 = builder.new_block();

        builder.goto(block1, block2);

        let func = builder.current_function.as_ref().unwrap();
        assert!(matches!(func.blocks[0].terminator, Terminator::Goto(_)));
        if let Terminator::Goto(target) = &func.blocks[0].terminator {
            assert_eq!(*target, block2);
        }
    }

    /// Test conditional branch terminator
    #[test]
    fn test_branch() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let entry = builder.new_block();
        let then_block = builder.new_block();
        let else_block = builder.new_block();

        let cond = Operand::Constant(Constant::Bool(true));
        builder.branch(entry, cond, then_block, else_block);

        let func = builder.current_function.as_ref().unwrap();
        if let Terminator::If { condition: _, then_block: tb, else_block: eb } = &func.blocks[0].terminator {
            assert_eq!(*tb, then_block);
            assert_eq!(*eb, else_block);
        } else {
            panic!("Expected If terminator");
        }
    }

    /// Test return terminator
    #[test]
    fn test_return() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();

        // Test return with value
        let operand = Operand::Constant(Constant::Int(42, Type::I32));
        builder.return_(block, Some(operand));

        let func = builder.current_function.as_ref().unwrap();
        assert!(matches!(func.blocks[0].terminator, Terminator::Return(Some(_))));
    }

    /// Test return terminator without value
    #[test]
    fn test_return_unit() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();

        builder.return_(block, None);

        let func = builder.current_function.as_ref().unwrap();
        assert!(matches!(func.blocks[0].terminator, Terminator::Return(None)));
    }

    /// Test call terminator
    #[test]
    fn test_call_term() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block1 = builder.new_block();
        let block2 = builder.new_block();
        let local = builder.alloc_local(Type::I32, false, None);

        let func_operand = Operand::Constant(Constant::String("callee".to_string()));
        let args = vec![Operand::Constant(Constant::Int(1, Type::I32))];
        builder.call_term(block1, func_operand, args, Some((Place::Local(local), block2)));

        let func = builder.current_function.as_ref().unwrap();
        if let Terminator::Call { func: _, args, destination } = &func.blocks[0].terminator {
            assert_eq!(args.len(), 1);
            assert!(destination.is_some());
        } else {
            panic!("Expected Call terminator");
        }
    }

    /// Test switch terminator
    #[test]
    fn test_switch() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let entry = builder.new_block();
        let case1 = builder.new_block();
        let case2 = builder.new_block();
        let default = builder.new_block();

        let discriminant = Operand::Constant(Constant::Int(0, Type::I32));
        let targets = vec![
            (Constant::Int(1, Type::I32), case1),
            (Constant::Int(2, Type::I32), case2),
        ];
        builder.switch(entry, discriminant, targets, Some(default));

        let func = builder.current_function.as_ref().unwrap();
        if let Terminator::Switch { targets: t, default: d, .. } = &func.blocks[0].terminator {
            assert_eq!(t.len(), 2);
            assert_eq!(*d, Some(default));
        } else {
            panic!("Expected Switch terminator");
        }
    }

    /// Test binary operation helper
    #[test]
    fn test_binary_op() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();
        let dest = builder.alloc_local(Type::I32, false, None);

        let left = Operand::Constant(Constant::Int(5, Type::I32));
        let right = Operand::Constant(Constant::Int(3, Type::I32));
        builder.binary_op(block, dest, BinOp::Add, left, right);

        let func = builder.current_function.as_ref().unwrap();
        if let Statement::Assign(place, rvalue) = &func.blocks[0].statements[0] {
            assert_eq!(*place, Place::Local(dest));
            assert!(matches!(rvalue, Rvalue::BinaryOp(BinOp::Add, _, _)));
        } else {
            panic!("Expected Assign statement");
        }
    }

    /// Test unary operation helper
    #[test]
    fn test_unary_op() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();
        let dest = builder.alloc_local(Type::I32, false, None);

        let operand = Operand::Constant(Constant::Int(-5, Type::I32));
        builder.unary_op(block, dest, UnOp::Neg, operand);

        let func = builder.current_function.as_ref().unwrap();
        if let Statement::Assign(place, rvalue) = &func.blocks[0].statements[0] {
            assert_eq!(*place, Place::Local(dest));
            assert!(matches!(rvalue, Rvalue::UnaryOp(UnOp::Neg, _)));
        } else {
            panic!("Expected Assign statement");
        }
    }

    /// Test function call helper
    #[test]
    fn test_call() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();
        let dest = builder.alloc_local(Type::I32, false, None);

        let func_operand = Operand::Constant(Constant::String("helper".to_string()));
        let args = vec![Operand::Constant(Constant::Int(42, Type::I32))];
        let next_block = builder.call(block, dest, func_operand, args);

        assert_eq!(next_block, BlockId(1));

        let func = builder.current_function.as_ref().unwrap();
        assert_eq!(func.blocks.len(), 2);
        if let Terminator::Call { destination, .. } = &func.blocks[0].terminator {
            assert_eq!(destination, &Some((Place::Local(dest), next_block)));
        } else {
            panic!("Expected Call terminator");
        }
    }

    /// Test cast helper
    #[test]
    fn test_cast() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();
        let dest = builder.alloc_local(Type::F64, false, None);

        let operand = Operand::Constant(Constant::Int(42, Type::I32));
        builder.cast(block, dest, CastKind::Numeric, operand, Type::F64);

        let func = builder.current_function.as_ref().unwrap();
        if let Statement::Assign(place, rvalue) = &func.blocks[0].statements[0] {
            assert_eq!(*place, Place::Local(dest));
            if let Rvalue::Cast(kind, _, target_ty) = rvalue {
                assert_eq!(*kind, CastKind::Numeric);
                assert_eq!(*target_ty, Type::F64);
            } else {
                panic!("Expected Cast rvalue");
            }
        } else {
            panic!("Expected Assign statement");
        }
    }

    /// Test reference creation helper
    #[test]
    fn test_ref() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();
        let source = builder.alloc_local(Type::I32, true, None);
        let dest = builder.alloc_local(Type::Ref(Box::new(Type::I32), Mutability::Mutable), false, None);

        builder.ref_(block, dest, Mutability::Mutable, Place::Local(source));

        let func = builder.current_function.as_ref().unwrap();
        if let Statement::Assign(place, rvalue) = &func.blocks[0].statements[0] {
            assert_eq!(*place, Place::Local(dest));
            assert!(matches!(rvalue, Rvalue::Ref(Mutability::Mutable, _)));
        } else {
            panic!("Expected Assign statement");
        }
    }

    /// Test move helper
    #[test]
    fn test_move() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();
        let source = builder.alloc_local(Type::String, false, None);
        let dest = builder.alloc_local(Type::String, false, None);

        builder.move_(block, Place::Local(dest), Place::Local(source));

        let func = builder.current_function.as_ref().unwrap();
        if let Statement::Assign(place, rvalue) = &func.blocks[0].statements[0] {
            assert_eq!(*place, Place::Local(dest));
            assert!(matches!(rvalue, Rvalue::Use(Operand::Move(_))));
        } else {
            panic!("Expected Assign statement");
        }
    }

    /// Test copy helper
    #[test]
    fn test_copy() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();
        let source = builder.alloc_local(Type::I32, false, None);
        let dest = builder.alloc_local(Type::I32, false, None);

        builder.copy(block, Place::Local(dest), Place::Local(source));

        let func = builder.current_function.as_ref().unwrap();
        if let Statement::Assign(place, rvalue) = &func.blocks[0].statements[0] {
            assert_eq!(*place, Place::Local(dest));
            assert!(matches!(rvalue, Rvalue::Use(Operand::Copy(_))));
        } else {
            panic!("Expected Assign statement");
        }
    }

    /// Test constant assignment helper
    #[test]
    fn test_const() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();
        let dest = builder.alloc_local(Type::Bool, false, None);

        builder.const_(block, Place::Local(dest), Constant::Bool(true));

        let func = builder.current_function.as_ref().unwrap();
        if let Statement::Assign(place, rvalue) = &func.blocks[0].statements[0] {
            assert_eq!(*place, Place::Local(dest));
            assert!(matches!(rvalue, Rvalue::Use(Operand::Constant(Constant::Bool(true)))));
        } else {
            panic!("Expected Assign statement");
        }
    }

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
            Operand::Copy(Place::Local(b)),
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
            Operand::Constant(Constant::Int(0, Type::I32)),
        );
        builder.branch(
            entry,
            Operand::Copy(Place::Local(cond)),
            then_block,
            else_block,
        );
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

    /// Test edge case: operations without active function
    #[test]
    fn test_operations_without_function() {
        let mut builder = MirBuilder::new();

        // These operations should not panic, but won't have effect
        let local = builder.alloc_local(Type::I32, false, None);
        let block = builder.new_block();
        builder.push_statement(block, Statement::StorageLive(local));
        builder.set_terminator(block, Terminator::Return(None));

        // Verify no function was created
        assert!(builder.current_function.is_none());
    }

    /// Test complex control flow with switch
    #[test]
    fn test_complex_switch() {
        let mut builder = MirBuilder::new();
        builder.start_function("test_switch".to_string(), Type::I32);
        let param = builder.add_param("value".to_string(), Type::I32);

        let entry = builder.new_block();
        let case1 = builder.new_block();
        let case2 = builder.new_block();
        let case3 = builder.new_block();
        let default = builder.new_block();

        // Create switch with multiple cases
        let discriminant = Operand::Copy(Place::Local(param));
        let targets = vec![
            (Constant::Int(1, Type::I32), case1),
            (Constant::Int(2, Type::I32), case2),
            (Constant::Int(3, Type::I32), case3),
        ];
        builder.switch(entry, discriminant, targets, Some(default));

        // Add returns to each case
        builder.return_(case1, Some(Operand::Constant(Constant::Int(10, Type::I32))));
        builder.return_(case2, Some(Operand::Constant(Constant::Int(20, Type::I32))));
        builder.return_(case3, Some(Operand::Constant(Constant::Int(30, Type::I32))));
        builder.return_(default, Some(Operand::Constant(Constant::Int(0, Type::I32))));

        let func = builder.finish_function().unwrap();
        assert_eq!(func.blocks.len(), 5);

        // Verify switch structure
        if let Terminator::Switch { targets, default: d, .. } = &func.blocks[0].terminator {
            assert_eq!(targets.len(), 3);
            assert_eq!(*d, Some(default));
        } else {
            panic!("Expected Switch terminator");
        }
    }

    /// Test various binary operations
    #[test]
    fn test_all_binary_ops() {
        let mut builder = MirBuilder::new();
        builder.start_function("test_binops".to_string(), Type::Unit);
        let block = builder.new_block();

        let ops = vec![
            BinOp::Add, BinOp::Sub, BinOp::Mul, BinOp::Div, BinOp::Rem, BinOp::Pow,
            BinOp::BitAnd, BinOp::BitOr, BinOp::BitXor, BinOp::Shl, BinOp::Shr,
            BinOp::Eq, BinOp::Ne, BinOp::Lt, BinOp::Le, BinOp::Gt, BinOp::Ge,
            BinOp::And, BinOp::Or, BinOp::NullCoalesce,
        ];

        for (i, op) in ops.iter().enumerate() {
            let dest = builder.alloc_local(Type::Bool, false, None);
            let left = Operand::Constant(Constant::Int(i as i128, Type::I32));
            let right = Operand::Constant(Constant::Int((i + 1) as i128, Type::I32));
            builder.binary_op(block, dest, *op, left, right);
        }

        let func = builder.finish_function().unwrap();
        assert_eq!(func.blocks[0].statements.len(), ops.len());
    }

    /// Test all unary operations
    #[test]
    fn test_all_unary_ops() {
        let mut builder = MirBuilder::new();
        builder.start_function("test_unops".to_string(), Type::Unit);
        let block = builder.new_block();

        let ops = vec![UnOp::Neg, UnOp::Not, UnOp::BitNot, UnOp::Ref];

        for op in ops {
            let dest = builder.alloc_local(Type::I32, false, None);
            let operand = Operand::Constant(Constant::Int(42, Type::I32));
            builder.unary_op(block, dest, op, operand);
        }

        let func = builder.finish_function().unwrap();
        assert_eq!(func.blocks[0].statements.len(), 4);
    }

    /// Test all cast kinds
    #[test]
    fn test_all_cast_kinds() {
        let mut builder = MirBuilder::new();
        builder.start_function("test_casts".to_string(), Type::Unit);
        let block = builder.new_block();

        let cast_kinds = vec![CastKind::Numeric, CastKind::Pointer, CastKind::Unsize];

        for kind in cast_kinds {
            let dest = builder.alloc_local(Type::F64, false, None);
            let operand = Operand::Constant(Constant::Int(42, Type::I32));
            builder.cast(block, dest, kind, operand, Type::F64);
        }

        let func = builder.finish_function().unwrap();
        assert_eq!(func.blocks[0].statements.len(), 3);
    }

    /// Test mutability variants
    #[test]
    fn test_mutability_variants() {
        let mut builder = MirBuilder::new();
        builder.start_function("test_mutability".to_string(), Type::Unit);
        let block = builder.new_block();
        let source = builder.alloc_local(Type::I32, true, None);

        // Test immutable reference
        let immut_ref = builder.alloc_local(Type::Ref(Box::new(Type::I32), Mutability::Immutable), false, None);
        builder.ref_(block, immut_ref, Mutability::Immutable, Place::Local(source));

        // Test mutable reference
        let mut_ref = builder.alloc_local(Type::Ref(Box::new(Type::I32), Mutability::Mutable), false, None);
        builder.ref_(block, mut_ref, Mutability::Mutable, Place::Local(source));

        let func = builder.finish_function().unwrap();
        assert_eq!(func.blocks[0].statements.len(), 2);
    }

    /// Test call terminator without destination
    #[test]
    fn test_call_no_destination() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let block = builder.new_block();

        let func_operand = Operand::Constant(Constant::String("print".to_string()));
        let args = vec![Operand::Constant(Constant::String("hello".to_string()))];
        builder.call_term(block, func_operand, args, None);

        let func = builder.current_function.as_ref().unwrap();
        if let Terminator::Call { destination, .. } = &func.blocks[0].terminator {
            assert!(destination.is_none());
        } else {
            panic!("Expected Call terminator");
        }
    }

    /// Test switch without default case
    #[test]
    fn test_switch_no_default() {
        let mut builder = MirBuilder::new();
        builder.start_function("test".to_string(), Type::Unit);
        let entry = builder.new_block();
        let case1 = builder.new_block();

        let discriminant = Operand::Constant(Constant::Int(1, Type::I32));
        let targets = vec![(Constant::Int(1, Type::I32), case1)];
        builder.switch(entry, discriminant, targets, None);

        let func = builder.current_function.as_ref().unwrap();
        if let Terminator::Switch { default, .. } = &func.blocks[0].terminator {
            assert!(default.is_none());
        } else {
            panic!("Expected Switch terminator");
        }
    }
}
#[cfg(test)]
mod property_tests_builder {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: MirBuilder creation never panics
        #[test]
        fn test_new_never_panics(_: u32) {
            let _builder = MirBuilder::new();
        }

        /// Property: Function names are preserved correctly
        #[test]
        fn test_function_name_preserved(name: String, return_type_idx: usize) {
            prop_assume!(!name.is_empty() && name.len() <= 100);

            let types = vec![Type::Unit, Type::Bool, Type::I32, Type::F64, Type::String];
            let return_type = types[return_type_idx % types.len()].clone();

            let mut builder = MirBuilder::new();
            builder.start_function(name.clone(), return_type.clone());

            let func = builder.finish_function().unwrap();
            prop_assert_eq!(func.name, name);
            prop_assert_eq!(func.return_ty, return_type);
        }

        /// Property: Local allocation IDs are sequential
        #[test]
        fn test_local_allocation_sequential(count: usize) {
            prop_assume!(count <= 50); // Reasonable limit for property tests

            let mut builder = MirBuilder::new();
            builder.start_function("test".to_string(), Type::Unit);

            let mut locals = Vec::new();
            for i in 0..count {
                let local = builder.alloc_local(Type::I32, i % 2 == 0, None);
                locals.push(local);
            }

            // Verify sequential allocation
            for (i, local) in locals.iter().enumerate() {
                prop_assert_eq!(local.0, i);
            }
        }

        /// Property: Block allocation IDs are sequential
        #[test]
        fn test_block_allocation_sequential(count: usize) {
            prop_assume!(count <= 50); // Reasonable limit for property tests

            let mut builder = MirBuilder::new();
            builder.start_function("test".to_string(), Type::Unit);

            let mut blocks = Vec::new();
            for _ in 0..count {
                let block = builder.new_block();
                blocks.push(block);
            }

            // Verify sequential allocation
            for (i, block) in blocks.iter().enumerate() {
                prop_assert_eq!(block.0, i);
            }
        }

        /// Property: Named locals are findable by name
        #[test]
        fn test_named_locals_findable(name: String) {
            prop_assume!(!name.is_empty() && name.len() <= 50);

            let mut builder = MirBuilder::new();
            builder.start_function("test".to_string(), Type::Unit);

            let local = builder.alloc_local(Type::I32, false, Some(name.clone()));
            prop_assert_eq!(builder.get_local(&name), Some(local));
        }

        /// Property: Parameters are correctly added and numbered
        #[test]
        fn test_params_numbered_correctly(param_names: Vec<String>) {
            prop_assume!(param_names.len() <= 10);
            prop_assume!(param_names.iter().all(|n| !n.is_empty() && n.len() <= 50));

            let mut builder = MirBuilder::new();
            builder.start_function("test".to_string(), Type::Unit);

            let mut params = Vec::new();
            for name in &param_names {
                let param = builder.add_param(name.clone(), Type::I32);
                params.push(param);
            }

            // Verify sequential numbering
            for (i, param) in params.iter().enumerate() {
                prop_assert_eq!(param.0, i);
            }

            // Verify params are findable by name
            for name in &param_names {
                prop_assert!(builder.get_local(name).is_some());
            }
        }

        /// Property: Statement count increases with each push
        #[test]
        fn test_statement_count_increases(stmt_count: usize) {
            prop_assume!(stmt_count <= 20); // Reasonable limit

            let mut builder = MirBuilder::new();
            builder.start_function("test".to_string(), Type::Unit);
            let block = builder.new_block();

            for i in 0..stmt_count {
                let local = builder.alloc_local(Type::I32, false, None);
                builder.push_statement(block, Statement::StorageLive(local));

                let func = builder.current_function.as_ref().unwrap();
                prop_assert_eq!(func.blocks[0].statements.len(), i + 1);
            }
        }

        /// Property: Binary operations preserve operator type
        #[test]
        fn test_binary_op_preserves_operator(op_idx: usize) {
            let ops = vec![BinOp::Add, BinOp::Sub, BinOp::Mul, BinOp::Eq, BinOp::Lt];
            let op = ops[op_idx % ops.len()];

            let mut builder = MirBuilder::new();
            builder.start_function("test".to_string(), Type::Unit);
            let block = builder.new_block();
            let dest = builder.alloc_local(Type::Bool, false, None);

            let left = Operand::Constant(Constant::Int(1, Type::I32));
            let right = Operand::Constant(Constant::Int(2, Type::I32));
            builder.binary_op(block, dest, op, left, right);

            let func = builder.finish_function().unwrap();
            if let Statement::Assign(_, Rvalue::BinaryOp(actual_op, _, _)) = &func.blocks[0].statements[0] {
                prop_assert_eq!(*actual_op, op);
            } else {
                return Err(proptest::test_runner::TestCaseError::fail("Expected binary op assignment"));
            }
        }

        /// Property: Unary operations preserve operator type
        #[test]
        fn test_unary_op_preserves_operator(op_idx: usize) {
            let ops = vec![UnOp::Neg, UnOp::Not, UnOp::BitNot];
            let op = ops[op_idx % ops.len()];

            let mut builder = MirBuilder::new();
            builder.start_function("test".to_string(), Type::Unit);
            let block = builder.new_block();
            let dest = builder.alloc_local(Type::I32, false, None);

            let operand = Operand::Constant(Constant::Int(42, Type::I32));
            builder.unary_op(block, dest, op, operand);

            let func = builder.finish_function().unwrap();
            if let Statement::Assign(_, Rvalue::UnaryOp(actual_op, _)) = &func.blocks[0].statements[0] {
                prop_assert_eq!(*actual_op, op);
            } else {
                return Err(proptest::test_runner::TestCaseError::fail("Expected unary op assignment"));
            }
        }

        /// Property: Function can be built without current function set
        #[test]
        fn test_no_function_operations_safe(op_count: usize) {
            prop_assume!(op_count <= 10);

            let mut builder = MirBuilder::new();
            // Don't start a function

            // These should not panic
            for _ in 0..op_count {
                let local = builder.alloc_local(Type::I32, false, None);
                let block = builder.new_block();
                builder.push_statement(block, Statement::StorageLive(local));
                builder.set_terminator(block, Terminator::Return(None));
            }

            prop_assert!(builder.current_function.is_none());
        }

        /// Property: Block mutation is consistent
        #[test]
        fn test_block_mutation_consistent(valid_block_count: usize, invalid_id: usize) {
            prop_assume!(valid_block_count <= 10);
            prop_assume!(invalid_id >= 100); // Ensure it's invalid

            let mut builder = MirBuilder::new();
            builder.start_function("test".to_string(), Type::Unit);

            let mut valid_blocks = Vec::new();
            for _ in 0..valid_block_count {
                let block = builder.new_block();
                valid_blocks.push(block);
            }

            // Valid blocks should be accessible
            for &block in &valid_blocks {
                prop_assert!(builder.block_mut(block).is_some());
            }

            // Invalid block should not be accessible
            let invalid_block = BlockId(invalid_id);
            prop_assert!(builder.block_mut(invalid_block).is_none());
        }
    }
}
