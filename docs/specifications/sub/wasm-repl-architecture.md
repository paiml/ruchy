# Sub-spec: WASM REPL -- Architecture

**Parent:** [wasm-repl-spec.md](../wasm-repl-spec.md) Architecture Section

---

## Architecture

### Bytecode VM

```rust
// Stack-based VM with register optimization
#[repr(u8)]
pub enum OpCode {
    // Stack operations
    Push(Value),
    Pop,
    Dup,
    
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
    
    // Memory
    LoadLocal(u16),
    StoreLocal(u16),
    LoadGlobal(u16),
    StoreGlobal(u16),
    
    // Control flow
    Jump(i32),
    JumpIf(i32),
    Call(u16),
    Return,
    
    // DataFrame ops
    Select(u16),  // Column indices
    Filter(u16),  // Predicate function
    Project(u16), // Transform function
}

pub struct BytecodeVM {
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    globals: Vec<Value>,
    constants: Vec<Value>,
}

impl BytecodeVM {
    pub fn execute(&mut self, chunk: &Chunk) -> Result<Value> {
        let mut ip = 0;
        let mut frame = CallFrame::new(chunk);
        
        loop {
            let op = chunk.code[ip];
            ip += 1;
            
            match op {
                OpCode::Push(v) => self.stack.push(v),
                OpCode::Add => {
                    let b = self.stack.pop()?;
                    let a = self.stack.pop()?;
                    self.stack.push(a.add(b)?);
                }
                OpCode::LoadGlobal(idx) => {
                    self.stack.push(self.globals[idx as usize].clone());
                }
                OpCode::StoreGlobal(idx) => {
                    let val = self.stack.pop()?;
                    if idx as usize >= self.globals.len() {
                        self.globals.resize(idx as usize + 1, Value::Nil);
                    }
                    self.globals[idx as usize] = val;
                }
                OpCode::Return => {
                    return Ok(self.stack.pop()?);
                }
                _ => todo!()
            }
        }
    }
}
```

### Memory Architecture

```rust
// Zero-copy arena with explicit promotion
pub struct CellMemory {
    transient: BumpArena<256_KB>,  // Reset after execution
    persistent: SlabAllocator,      // Promoted values only
}

impl CellMemory {
    pub fn allocate(&mut self, size: usize) -> *mut u8 {
        // Allocate in transient by default
        self.transient.alloc(size)
    }
    
    pub fn promote(&mut self, value: &Value) -> PromotedRef {
        // Explicitly copy to persistent storage
        match value {
            Value::DataFrame(df) => {
                // Zero-copy promotion for Arrow buffers
                PromotedRef::DataFrame(self.persistent.adopt(df))
            }
            _ => {
                // Deep copy for other types
                PromotedRef::Value(self.persistent.copy(value))
            }
        }
    }
}

// Explicit global promotion
pub struct GlobalRegistry {
    values: HashMap<String, PromotedRef>,
    
    // Track which cell defined each global
    provenance: HashMap<String, CellId>,
}

impl GlobalRegistry {
    pub fn export(&mut self, name: String, value: PromotedRef, cell: CellId) {
        self.provenance.insert(name.clone(), cell);
        self.values.insert(name, value);
    }
}
```

### Apache Arrow DataFrame

```rust
// Arrow-compatible columnar layout
#[repr(C)]
pub struct DataFrame {
    schema: Arc<Schema>,
    columns: Vec<ArrayRef>,  // Arrow arrays
    row_count: usize,
}

pub type ArrayRef = Arc<dyn Array>;

pub trait Array: Send + Sync {
    fn data_type(&self) -> &DataType;
    fn len(&self) -> usize;
    fn slice(&self, offset: usize, length: usize) -> ArrayRef;
    fn buffer(&self) -> &Buffer;  // Raw bytes for zero-copy
}

// Concrete array types
pub struct Float64Array {
    data: Buffer,
    null_bitmap: Option<Bitmap>,
}

impl Array for Float64Array {
    fn slice(&self, offset: usize, length: usize) -> ArrayRef {
        // Zero-copy slice
        Arc::new(Float64Array {
            data: self.data.slice(offset * 8, length * 8),
            null_bitmap: self.null_bitmap.as_ref().map(|b| b.slice(offset, length)),
        })
    }
}

impl DataFrame {
    pub fn select(&self, columns: &[&str]) -> DataFrame {
        let selected: Vec<ArrayRef> = self.schema.fields()
            .iter()
            .zip(&self.columns)
            .filter(|(field, _)| columns.contains(&field.name.as_str()))
            .map(|(_, col)| col.clone())
            .collect();
        
        DataFrame {
            schema: Arc::new(self.schema.project(columns)),
            columns: selected,
            row_count: self.row_count,
        }
    }
    
    pub fn filter_indices(&self, indices: &[usize]) -> DataFrame {
        // Build once, apply to all columns
        let columns = self.columns.iter()
            .map(|col| take(col, indices))
            .collect();
        
        DataFrame {
            schema: self.schema.clone(),
            columns,
            row_count: indices.len(),
        }
    }
}

// Efficient gather operation
fn take(array: &ArrayRef, indices: &[usize]) -> ArrayRef {
    match array.data_type() {
        DataType::Float64 => {
            let arr = array.as_any().downcast_ref::<Float64Array>().unwrap();
            let mut builder = Float64Builder::new(indices.len());
            for &idx in indices {
                builder.append_value(arr.value(idx));
            }
            Arc::new(builder.finish())
        }
        _ => todo!()
    }
}
```

### Structured Error Handling

```rust
#[derive(Serialize, Deserialize)]
pub struct ExecutionError {
    kind: ErrorKind,
    message: String,
    span: Option<Span>,
    backtrace: Vec<Frame>,
    suggestion: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub enum ErrorKind {
    ParseError,
    TypeError { expected: String, found: String },
    NameError { name: String },
    IndexError { index: usize, length: usize },
    RuntimeError,
}

#[wasm_bindgen]
impl NotebookRuntime {
    pub fn execute(&mut self, cell_id: &str, code: &str) -> Result<JsValue, JsValue> {
        match self.execute_internal(cell_id, code) {
            Ok(value) => Ok(serialize_value(&value)),
            Err(err) => {
                // Structured error for rich display
                let error_obj = ExecutionError {
                    kind: err.kind(),
                    message: err.to_string(),
                    span: err.span(),
                    backtrace: err.backtrace(),
                    suggestion: self.suggest_fix(&err),
                };
                Err(serde_wasm_bindgen::to_value(&error_obj)?)
            }
        }
    }
    
    fn suggest_fix(&self, error: &Error) -> Option<String> {
        match error.kind() {
            ErrorKind::NameError { name } => {
                // Levenshtein distance for typo suggestions
                let closest = self.find_similar_names(name);
                if !closest.is_empty() {
                    Some(format!("Did you mean: {}?", closest.join(", ")))
                } else {
                    None
                }
            }
            _ => None
        }
    }
}
```

## Robust Demo Conversion

```rust
// Simplified AST for structural parsing only
pub enum DemoNode {
    Comment(String),
    Code(Vec<Statement>),
    Section(String),
}

pub struct DemoParser {
    lexer: Lexer,
}

impl DemoParser {
    pub fn parse(&mut self, input: &str) -> Result<Vec<DemoNode>> {
        self.lexer.init(input);
        let mut nodes = Vec::new();
        
        while !self.lexer.is_at_end() {
            if self.lexer.peek_comment() {
                nodes.push(self.parse_comment_block());
            } else if self.lexer.peek_section_marker() {
                nodes.push(self.parse_section());
            } else {
                nodes.push(self.parse_code_block());
            }
        }
        
        Ok(nodes)
    }
    
    fn parse_code_block(&mut self) -> DemoNode {
        let mut statements = Vec::new();
        
        // Parse until we hit a comment or section
        while !self.lexer.peek_comment() && !self.lexer.peek_section_marker() && !self.lexer.is_at_end() {
            if let Ok(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                // Skip unparseable lines gracefully
                self.lexer.skip_line();
            }
        }
        
        DemoNode::Code(statements)
    }
}

pub struct NotebookConverter {
    parser: DemoParser,
}

impl NotebookConverter {
    pub fn convert(&self, demo: &str) -> Result<Notebook> {
        let nodes = self.parser.parse(demo)?;
        let mut cells = Vec::new();
        
        for node in nodes {
            match node {
                DemoNode::Section(title) => {
                    cells.push(Cell::Markdown(format!("# {}", title)));
                }
                DemoNode::Comment(text) => {
                    cells.push(Cell::Markdown(text));
                }
                DemoNode::Code(statements) => {
                    // Group related statements
                    let groups = self.group_statements(statements);
                    for group in groups {
                        cells.push(Cell::Code(self.statements_to_code(group)));
                    }
                }
            }
        }
        
        Ok(Notebook { cells })
    }
    
    fn group_statements(&self, statements: Vec<Statement>) -> Vec<Vec<Statement>> {
        // Group by logical units (function definitions, data operations, etc.)
        let mut groups = Vec::new();
        let mut current = Vec::new();
        
        for stmt in statements {
            match stmt {
                Statement::FunctionDef(_) if !current.is_empty() => {
                    // Start new group for functions
                    groups.push(current);
                    current = vec![stmt];
                }
                Statement::Import(_) if !current.is_empty() => {
                    // Imports get their own cell
                    groups.push(current);
                    groups.push(vec![stmt]);
                    current = Vec::new();
                }
                _ => current.push(stmt),
            }
        }
        
        if !current.is_empty() {
            groups.push(current);
        }
        
        groups
    }
}
```

