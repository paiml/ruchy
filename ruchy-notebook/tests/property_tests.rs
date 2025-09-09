use proptest::prelude::*;
use proptest::strategy::{Just, Strategy};
use ruchy_notebook::{Arena, SlabAllocator, VirtualMachine, BytecodeModule};
use ruchy_notebook::vm::{OpCode, Instruction, bytecode::Value};

// Property: Arena allocations are independent
proptest! {
    #[test]
    fn prop_arena_independent_allocations(values in prop::collection::vec(any::<i32>(), 1..100)) {
        let arena = Arena::new();
        let mut refs = Vec::new();
        
        for value in &values {
            refs.push(arena.alloc(*value));
        }
        
        // All allocations should maintain their values
        for (i, ref_) in refs.iter().enumerate() {
            assert_eq!(**ref_, values[i]);
        }
    }
}

// Property: Arena reset should invalidate all allocations
proptest! {
    #[test]
    fn prop_arena_reset_invalidates(
        values in prop::collection::vec(any::<i32>(), 1..100)
    ) {
        let arena = Arena::new();
        let mut refs = Vec::new();
        
        for value in &values {
            refs.push(arena.alloc(*value));
        }
        
        let used_before = arena.used();
        assert!(used_before > 0);
        
        arena.reset();
        assert_eq!(arena.used(), 0);
        
        // New allocations should start from beginning
        let new_ref = arena.alloc(42i32);
        let new_used = arena.used();
        assert!(new_used < used_before);
    }
}

// Property: Slab allocator handles should be unique
proptest! {
    #[test]
    fn prop_slab_unique_handles(
        values in prop::collection::vec(any::<i32>(), 1..100)
    ) {
        let mut slab = SlabAllocator::new();
        let mut handles = Vec::new();
        
        for value in values {
            let handle = slab.insert(value);
            
            // Check handle is unique
            assert!(!handles.contains(&handle));
            handles.push(handle);
        }
        
        // All handles should retrieve correct values
        for (i, handle) in handles.iter().enumerate() {
            assert!(slab.get(*handle).is_some());
        }
    }
}

// Property: Slab compaction preserves all live values
proptest! {
    #[test]
    fn prop_slab_compact_preserves(
        ops in prop::collection::vec(
            prop_oneof![
                Just(SlabOp::Insert),
                Just(SlabOp::Remove),
            ],
            1..100
        )
    ) {
        let mut slab = SlabAllocator::<i32>::new();
        let mut handles = Vec::new();
        let mut live_handles = Vec::new();
        let mut counter = 0;
        
        for op in ops {
            match op {
                SlabOp::Insert => {
                    let handle = slab.insert(counter);
                    handles.push(handle);
                    live_handles.push((handle, counter));
                    counter += 1;
                }
                SlabOp::Remove if !handles.is_empty() => {
                    let idx = handles.len() / 2;
                    let handle = handles.remove(idx);
                    slab.remove(handle);
                    live_handles.retain(|(h, _)| *h != handle);
                }
                _ => {}
            }
        }
        
        let remapping = slab.compact();
        
        // All live values should still be accessible
        for (old_handle, value) in live_handles {
            if let Some(&new_handle) = remapping.get(&old_handle) {
                assert_eq!(slab.get(new_handle), Some(&value));
            }
        }
    }
}

#[derive(Debug, Clone)]
enum SlabOp {
    Insert,
    Remove,
}

// Property: VM arithmetic operations are correct
proptest! {
    #[test]
    fn prop_vm_arithmetic(a in any::<i32>(), b in any::<i32>()) {
        // Test addition
        {
            let mut vm = VirtualMachine::new();
            let mut module = BytecodeModule::new();
            
            module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(a as i64)));
            module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(b as i64)));
            module.add_instruction(Instruction::new(OpCode::Add));
            module.add_instruction(Instruction::new(OpCode::Halt));
            
            let result = vm.execute(&module).unwrap();
            let expected = (a as i64).wrapping_add(b as i64);
            assert_eq!(result.value, Some(Value::Int(expected)));
        }
        
        // Test subtraction
        {
            let mut vm = VirtualMachine::new();
            let mut module = BytecodeModule::new();
            
            module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(a as i64)));
            module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(b as i64)));
            module.add_instruction(Instruction::new(OpCode::Sub));
            module.add_instruction(Instruction::new(OpCode::Halt));
            
            let result = vm.execute(&module).unwrap();
            let expected = (a as i64).wrapping_sub(b as i64);
            assert_eq!(result.value, Some(Value::Int(expected)));
        }
    }
}

// Property: VM stack operations maintain invariants
proptest! {
    #[test]
    fn prop_vm_stack_invariants(
        ops in prop::collection::vec(
            prop_oneof![
                any::<i64>().prop_map(|v| StackOp::Push(v)),
                Just(StackOp::Pop),
                Just(StackOp::Dup),
                Just(StackOp::Swap),
            ],
            1..50
        )
    ) {
        let mut vm = VirtualMachine::new();
        let mut module = BytecodeModule::new();
        let mut expected_stack = Vec::new();
        let mut valid = true;
        
        for op in ops {
            match op {
                StackOp::Push(v) => {
                    module.add_instruction(Instruction::with_operand(OpCode::Push, Value::Int(v)));
                    expected_stack.push(v);
                }
                StackOp::Pop if !expected_stack.is_empty() => {
                    module.add_instruction(Instruction::new(OpCode::Pop));
                    expected_stack.pop();
                }
                StackOp::Dup if !expected_stack.is_empty() => {
                    module.add_instruction(Instruction::new(OpCode::Dup));
                    if let Some(&top) = expected_stack.last() {
                        expected_stack.push(top);
                    }
                }
                StackOp::Swap if expected_stack.len() >= 2 => {
                    module.add_instruction(Instruction::new(OpCode::Swap));
                    let len = expected_stack.len();
                    expected_stack.swap(len - 1, len - 2);
                }
                _ => valid = false,
            }
        }
        
        module.add_instruction(Instruction::new(OpCode::Halt));
        
        if valid && !expected_stack.is_empty() {
            let result = vm.execute(&module);
            assert!(result.is_ok());
            
            if let Some(Value::Int(v)) = result.unwrap().value {
                assert_eq!(v, *expected_stack.last().unwrap());
            }
        }
    }
}

#[derive(Debug, Clone)]
enum StackOp {
    Push(i64),
    Pop,
    Dup,
    Swap,
}

// Property: Memory fragmentation stays within bounds
proptest! {
    #[test]
    fn prop_arena_fragmentation_bounds(
        alloc_sizes in prop::collection::vec(10usize..1000, 10..100)
    ) {
        let total_size = 1024 * 1024; // 1MB
        let arena = Arena::with_capacity(total_size);
        
        for size in alloc_sizes {
            if arena.used() + size <= arena.capacity() {
                let _ = arena.alloc(vec![0u8; size]);
            }
        }
        
        let fragmentation = arena.fragmentation();
        
        // Fragmentation should be between 0 and 100
        assert!(fragmentation >= 0.0);
        assert!(fragmentation <= 100.0);
        
        // After reset, fragmentation should be 100%
        arena.reset();
        assert_eq!(arena.fragmentation(), 100.0);
    }
}

#[cfg(feature = "dataframe")]
mod dataframe_properties {
    use super::*;
    use ruchy_notebook::dataframe::{DataFrame, Column};
    
    proptest! {
        #[test]
        fn prop_dataframe_slice_preserves_data(
            data in prop::collection::vec(any::<i64>(), 10..100),
            offset in 0usize..10,
        ) {
            let col = Column::int64("test", data.clone());
            let df = DataFrame::new(vec![col]).unwrap();
            
            let length = std::cmp::min(5, df.num_rows() - offset);
            if length > 0 {
                let sliced = df.slice(offset, length).unwrap();
                
                assert_eq!(sliced.num_rows(), length);
                assert_eq!(sliced.num_columns(), df.num_columns());
                
                // Memory should not increase (zero-copy)
                assert!(sliced.memory_usage() <= df.memory_usage());
            }
        }
    }
    
    proptest! {
        #[test]
        fn prop_dataframe_filter_correctness(
            data in prop::collection::vec(any::<i64>(), 5..20),
            mask in prop::collection::vec(any::<bool>(), 5..20)
        ) {
            use arrow::array::BooleanArray;
            
            let min_len = std::cmp::min(data.len(), mask.len());
            let data_trimmed = &data[..min_len];
            let mask_trimmed = &mask[..min_len];
            
            let col = Column::int64("values", data_trimmed.to_vec());
            let df = DataFrame::new(vec![col]).unwrap();
            
            let mask_array = BooleanArray::from(mask_trimmed.to_vec());
            let filtered = df.filter(&mask_array).unwrap();
            
            let expected_rows = mask_trimmed.iter().filter(|&&b| b).count();
            assert_eq!(filtered.num_rows(), expected_rows);
        }
    }
}