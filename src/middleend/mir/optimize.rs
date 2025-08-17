//! MIR optimization passes

use super::types::*;
use std::collections::{HashSet, HashMap};

/// Dead Code Elimination pass
pub struct DeadCodeElimination {
    /// Set of live locals
    live_locals: HashSet<Local>,
    /// Set of live blocks
    live_blocks: HashSet<BlockId>,
}

impl DeadCodeElimination {
    /// Create a new DCE pass
    pub fn new() -> Self {
        Self {
            live_locals: HashSet::new(),
            live_blocks: HashSet::new(),
        }
    }

    /// Run DCE on a function
    pub fn run(&mut self, func: &mut Function) {
        // Mark live locals and blocks
        self.mark_live(func);
        
        // Remove dead statements
        self.remove_dead_statements(func);
        
        // Remove dead blocks
        self.remove_dead_blocks(func);
        
        // Remove dead locals
        self.remove_dead_locals(func);
    }

    /// Mark live locals and blocks
    fn mark_live(&mut self, func: &Function) {
        self.live_locals.clear();
        self.live_blocks.clear();
        
        // Start from entry block
        let mut worklist = vec![func.entry_block];
        self.live_blocks.insert(func.entry_block);
        
        while let Some(block_id) = worklist.pop() {
            if let Some(block) = func.blocks.iter().find(|b| b.id == block_id) {
                // Mark locals used in statements
                for stmt in &block.statements {
                    self.mark_statement_live(stmt);
                }
                
                // Mark locals used in terminator and add successor blocks
                self.mark_terminator_live(&block.terminator, &mut worklist);
            }
        }
        
        // Mark parameters as live
        for param in &func.params {
            self.live_locals.insert(*param);
        }
    }

    /// Mark locals in a statement as live
    fn mark_statement_live(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Assign(place, rvalue) => {
                self.mark_place_live(place);
                self.mark_rvalue_live(rvalue);
            }
            Statement::StorageLive(local) | Statement::StorageDead(local) => {
                self.live_locals.insert(*local);
            }
            Statement::Nop => {}
        }
    }

    /// Mark locals in a place as live
    fn mark_place_live(&mut self, place: &Place) {
        match place {
            Place::Local(local) => {
                self.live_locals.insert(*local);
            }
            Place::Field(base, _) => {
                self.mark_place_live(base);
            }
            Place::Index(base, index) => {
                self.mark_place_live(base);
                self.mark_place_live(index);
            }
            Place::Deref(base) => {
                self.mark_place_live(base);
            }
        }
    }

    /// Mark locals in an rvalue as live
    fn mark_rvalue_live(&mut self, rvalue: &Rvalue) {
        match rvalue {
            Rvalue::Use(operand) => {
                self.mark_operand_live(operand);
            }
            Rvalue::BinaryOp(_, left, right) => {
                self.mark_operand_live(left);
                self.mark_operand_live(right);
            }
            Rvalue::UnaryOp(_, operand) => {
                self.mark_operand_live(operand);
            }
            Rvalue::Ref(_, place) => {
                self.mark_place_live(place);
            }
            Rvalue::Aggregate(_, operands) => {
                for operand in operands {
                    self.mark_operand_live(operand);
                }
            }
            Rvalue::Call(func, args) => {
                self.mark_operand_live(func);
                for arg in args {
                    self.mark_operand_live(arg);
                }
            }
            Rvalue::Cast(_, operand, _) => {
                self.mark_operand_live(operand);
            }
        }
    }

    /// Mark locals in an operand as live
    fn mark_operand_live(&mut self, operand: &Operand) {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                self.mark_place_live(place);
            }
            Operand::Constant(_) => {}
        }
    }

    /// Mark locals in terminator as live and add successor blocks
    fn mark_terminator_live(&mut self, terminator: &Terminator, worklist: &mut Vec<BlockId>) {
        match terminator {
            Terminator::Goto(target) => {
                if self.live_blocks.insert(*target) {
                    worklist.push(*target);
                }
            }
            Terminator::If { condition, then_block, else_block } => {
                self.mark_operand_live(condition);
                if self.live_blocks.insert(*then_block) {
                    worklist.push(*then_block);
                }
                if self.live_blocks.insert(*else_block) {
                    worklist.push(*else_block);
                }
            }
            Terminator::Switch { discriminant, targets, default } => {
                self.mark_operand_live(discriminant);
                for (_, target) in targets {
                    if self.live_blocks.insert(*target) {
                        worklist.push(*target);
                    }
                }
                if let Some(default_block) = default {
                    if self.live_blocks.insert(*default_block) {
                        worklist.push(*default_block);
                    }
                }
            }
            Terminator::Return(operand) => {
                if let Some(op) = operand {
                    self.mark_operand_live(op);
                }
            }
            Terminator::Call { func, args, destination } => {
                self.mark_operand_live(func);
                for arg in args {
                    self.mark_operand_live(arg);
                }
                if let Some((place, target)) = destination {
                    self.mark_place_live(place);
                    if self.live_blocks.insert(*target) {
                        worklist.push(*target);
                    }
                }
            }
            Terminator::Unreachable => {}
        }
    }

    /// Remove dead statements from blocks
    fn remove_dead_statements(&self, func: &mut Function) {
        for block in &mut func.blocks {
            if !self.live_blocks.contains(&block.id) {
                continue;
            }
            
            block.statements.retain(|stmt| {
                match stmt {
                    Statement::Assign(place, _) => {
                        self.is_place_live(place)
                    }
                    Statement::StorageLive(local) | Statement::StorageDead(local) => {
                        self.live_locals.contains(local)
                    }
                    Statement::Nop => false, // Always remove nops
                }
            });
        }
    }

    /// Remove dead blocks
    fn remove_dead_blocks(&self, func: &mut Function) {
        func.blocks.retain(|block| self.live_blocks.contains(&block.id));
    }

    /// Remove dead locals
    fn remove_dead_locals(&self, func: &mut Function) {
        func.locals.retain(|local_decl| self.live_locals.contains(&local_decl.id));
    }

    /// Check if a place is live
    fn is_place_live(&self, place: &Place) -> bool {
        match place {
            Place::Local(local) => self.live_locals.contains(local),
            Place::Field(base, _) => self.is_place_live(base),
            Place::Index(base, index) => self.is_place_live(base) || self.is_place_live(index),
            Place::Deref(base) => self.is_place_live(base),
        }
    }
}

/// Constant Propagation pass
pub struct ConstantPropagation {
    /// Map from locals to their constant values
    constants: HashMap<Local, Constant>,
}

impl ConstantPropagation {
    /// Create a new constant propagation pass
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
        }
    }

    /// Run constant propagation on a function
    pub fn run(&mut self, func: &mut Function) {
        self.constants.clear();
        
        // Find constant assignments
        for block in &func.blocks {
            for stmt in &block.statements {
                if let Statement::Assign(Place::Local(local), rvalue) = stmt {
                    if let Some(constant) = self.extract_constant(rvalue) {
                        self.constants.insert(*local, constant);
                    }
                }
            }
        }
        
        // Replace uses of constants
        for block in &mut func.blocks {
            for stmt in &mut block.statements {
                self.propagate_in_statement(stmt);
            }
            self.propagate_in_terminator(&mut block.terminator);
        }
    }

    /// Extract constant from rvalue if possible
    fn extract_constant(&self, rvalue: &Rvalue) -> Option<Constant> {
        match rvalue {
            Rvalue::Use(Operand::Constant(c)) => Some(c.clone()),
            Rvalue::BinaryOp(op, left, right) => {
                self.eval_binary_op(*op, left, right)
            }
            Rvalue::UnaryOp(op, operand) => {
                self.eval_unary_op(*op, operand)
            }
            _ => None,
        }
    }

    /// Evaluate binary operation on constants
    fn eval_binary_op(&self, op: BinOp, left: &Operand, right: &Operand) -> Option<Constant> {
        let left_val = self.get_constant_value(left)?;
        let right_val = self.get_constant_value(right)?;
        
        match (op, &left_val, &right_val) {
            (BinOp::Add, Constant::Int(a, ty), Constant::Int(b, _)) => {
                Some(Constant::Int(a + b, ty.clone()))
            }
            (BinOp::Sub, Constant::Int(a, ty), Constant::Int(b, _)) => {
                Some(Constant::Int(a - b, ty.clone()))
            }
            (BinOp::Mul, Constant::Int(a, ty), Constant::Int(b, _)) => {
                Some(Constant::Int(a * b, ty.clone()))
            }
            (BinOp::Eq, Constant::Int(a, _), Constant::Int(b, _)) => {
                Some(Constant::Bool(a == b))
            }
            (BinOp::Lt, Constant::Int(a, _), Constant::Int(b, _)) => {
                Some(Constant::Bool(a < b))
            }
            (BinOp::And, Constant::Bool(a), Constant::Bool(b)) => {
                Some(Constant::Bool(*a && *b))
            }
            (BinOp::Or, Constant::Bool(a), Constant::Bool(b)) => {
                Some(Constant::Bool(*a || *b))
            }
            _ => None,
        }
    }

    /// Evaluate unary operation on constant
    fn eval_unary_op(&self, op: UnOp, operand: &Operand) -> Option<Constant> {
        let val = self.get_constant_value(operand)?;
        
        match (op, &val) {
            (UnOp::Neg, Constant::Int(i, ty)) => {
                Some(Constant::Int(-i, ty.clone()))
            }
            (UnOp::Not, Constant::Bool(b)) => {
                Some(Constant::Bool(!b))
            }
            _ => None,
        }
    }

    /// Get constant value for an operand
    fn get_constant_value(&self, operand: &Operand) -> Option<Constant> {
        match operand {
            Operand::Constant(c) => Some(c.clone()),
            Operand::Copy(Place::Local(local)) | Operand::Move(Place::Local(local)) => {
                self.constants.get(local).cloned()
            }
            _ => None,
        }
    }

    /// Propagate constants in a statement
    fn propagate_in_statement(&self, stmt: &mut Statement) {
        match stmt {
            Statement::Assign(_, rvalue) => {
                self.propagate_in_rvalue(rvalue);
            }
            _ => {}
        }
    }

    /// Propagate constants in an rvalue
    fn propagate_in_rvalue(&self, rvalue: &mut Rvalue) {
        match rvalue {
            Rvalue::Use(operand) => {
                self.propagate_in_operand(operand);
            }
            Rvalue::BinaryOp(_, left, right) => {
                self.propagate_in_operand(left);
                self.propagate_in_operand(right);
            }
            Rvalue::UnaryOp(_, operand) => {
                self.propagate_in_operand(operand);
            }
            Rvalue::Call(func, args) => {
                self.propagate_in_operand(func);
                for arg in args {
                    self.propagate_in_operand(arg);
                }
            }
            _ => {}
        }
    }

    /// Propagate constants in an operand
    fn propagate_in_operand(&self, operand: &mut Operand) {
        if let Some(constant) = self.get_constant_value(operand) {
            *operand = Operand::Constant(constant);
        }
    }

    /// Propagate constants in a terminator
    fn propagate_in_terminator(&self, terminator: &mut Terminator) {
        match terminator {
            Terminator::If { condition, .. } => {
                self.propagate_in_operand(condition);
            }
            Terminator::Switch { discriminant, .. } => {
                self.propagate_in_operand(discriminant);
            }
            Terminator::Return(Some(operand)) => {
                self.propagate_in_operand(operand);
            }
            Terminator::Call { func, args, .. } => {
                self.propagate_in_operand(func);
                for arg in args {
                    self.propagate_in_operand(arg);
                }
            }
            _ => {}
        }
    }
}

/// Common Subexpression Elimination pass
pub struct CommonSubexpressionElimination {
    /// Map from expressions to locals that compute them
    expressions: HashMap<String, Local>,
}

impl CommonSubexpressionElimination {
    /// Create a new CSE pass
    pub fn new() -> Self {
        Self {
            expressions: HashMap::new(),
        }
    }

    /// Run CSE on a function
    pub fn run(&mut self, func: &mut Function) {
        self.expressions.clear();
        
        for block in &mut func.blocks {
            for stmt in &mut block.statements {
                self.process_statement(stmt);
            }
        }
    }

    /// Process a statement for CSE
    fn process_statement(&mut self, stmt: &mut Statement) {
        if let Statement::Assign(Place::Local(local), rvalue) = stmt {
            let expr_key = self.rvalue_key(rvalue);
            
            if let Some(existing_local) = self.expressions.get(&expr_key) {
                // Replace with copy from existing local
                *stmt = Statement::Assign(
                    Place::Local(*local),
                    Rvalue::Use(Operand::Copy(Place::Local(*existing_local)))
                );
            } else {
                // Record this expression
                self.expressions.insert(expr_key, *local);
            }
        }
    }

    /// Generate a key for an rvalue
    fn rvalue_key(&self, rvalue: &Rvalue) -> String {
        match rvalue {
            Rvalue::Use(operand) => format!("use({})", self.operand_key(operand)),
            Rvalue::BinaryOp(op, left, right) => {
                format!("binop({:?}, {}, {})", op, self.operand_key(left), self.operand_key(right))
            }
            Rvalue::UnaryOp(op, operand) => {
                format!("unop({:?}, {})", op, self.operand_key(operand))
            }
            _ => format!("{rvalue:?}"), // Fallback
        }
    }

    /// Generate a key for an operand
    fn operand_key(&self, operand: &Operand) -> String {
        match operand {
            Operand::Copy(place) => format!("copy({})", self.place_key(place)),
            Operand::Move(place) => format!("move({})", self.place_key(place)),
            Operand::Constant(c) => format!("const({c:?})"),
        }
    }

    /// Generate a key for a place
    fn place_key(&self, place: &Place) -> String {
        match place {
            Place::Local(local) => format!("local({})", local.0),
            Place::Field(base, field) => format!("field({}, {})", self.place_key(base), field.0),
            Place::Index(base, index) => format!("index({}, {})", self.place_key(base), self.place_key(index)),
            Place::Deref(base) => format!("deref({})", self.place_key(base)),
        }
    }
}

/// Run all optimization passes on a function
pub fn optimize_function(func: &mut Function) {
    let mut dce = DeadCodeElimination::new();
    let mut const_prop = ConstantPropagation::new();
    let mut cse = CommonSubexpressionElimination::new();
    
    // Run multiple rounds for better results
    for _ in 0..3 {
        const_prop.run(func);
        cse.run(func);
        dce.run(func);
    }
}

/// Run all optimization passes on a program
pub fn optimize_program(program: &mut Program) {
    for function in program.functions.values_mut() {
        optimize_function(function);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    #[test]
    fn test_dead_code_elimination() {
        // TODO: Create test function with dead code and verify removal
    }

    #[test]
    fn test_constant_propagation() {
        // TODO: Create test with constants and verify propagation
    }

    #[test]
    fn test_common_subexpression_elimination() {
        // TODO: Create test with duplicate expressions and verify elimination
    }
}