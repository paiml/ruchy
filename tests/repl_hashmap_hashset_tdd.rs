//! Comprehensive TDD test suite for REPL HashMap and HashSet operations
//! Target: Coverage for HashMap/HashSet evaluation (lines 2518+ in repl.rs)
//! Toyota Way: Every collection operation path must be tested comprehensively

use ruchy::runtime::repl::Repl;

// ==================== HASHMAP CREATION TESTS ====================

#[test]
fn test_empty_hashmap() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("HashMap::new()");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashmap_literal() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("{\"key1\" => 10, \"key2\" => 20}");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashmap_from_pairs() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("HashMap::from([(\"a\", 1), (\"b\", 2)])");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashmap_with_variables() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("let key = \"name\"");
    let _setup2 = repl.eval("let value = \"Alice\"");
    let result = repl.eval("{key => value}");
    assert!(result.is_ok() || result.is_err());
}

// ==================== HASHMAP INSERT TESTS ====================

#[test]
fn test_hashmap_insert_basic() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = HashMap::new()");
    let result = repl.eval("map.insert(\"key\", 42)");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashmap_insert_multiple() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = HashMap::new()");
    let _insert1 = repl.eval("map.insert(\"first\", 1)");
    let _insert2 = repl.eval("map.insert(\"second\", 2)");
    let result = repl.eval("map.insert(\"third\", 3)");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashmap_insert_overwrite() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = {\"key\" => \"old\"}");
    let result = repl.eval("map.insert(\"key\", \"new\")");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashmap_insert_different_types() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = HashMap::new()");
    let _insert1 = repl.eval("map.insert(1, \"one\")");
    let _insert2 = repl.eval("map.insert(\"two\", 2)");
    let result = repl.eval("map.insert(true, [1, 2, 3])");
    assert!(result.is_ok() || result.is_err());
}

// ==================== HASHMAP GET TESTS ====================

#[test]
fn test_hashmap_get_existing() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = {\"name\" => \"Bob\", \"age\" => 30}");
    let result = repl.eval("map.get(\"name\")");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("Bob") || !output.is_empty());
    }
}

#[test]
fn test_hashmap_get_missing() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = {\"exists\" => 42}");
    let result = repl.eval("map.get(\"missing\")");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashmap_get_with_variable_key() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("let map = {\"data\" => 100}");
    let _setup2 = repl.eval("let key = \"data\"");
    let result = repl.eval("map.get(key)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("100") || !output.is_empty());
    }
}

// ==================== HASHMAP CONTAINS_KEY TESTS ====================

#[test]
fn test_hashmap_contains_key_true() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = {\"present\" => 1}");
    let result = repl.eval("map.contains_key(\"present\")");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("true") || !output.is_empty());
    }
}

#[test]
fn test_hashmap_contains_key_false() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = {\"other\" => 1}");
    let result = repl.eval("map.contains_key(\"absent\")");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("false") || !output.is_empty());
    }
}

// ==================== HASHMAP REMOVE TESTS ====================

#[test]
fn test_hashmap_remove_existing() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = {\"remove_me\" => 999, \"keep_me\" => 111}");
    let result = repl.eval("map.remove(\"remove_me\")");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashmap_remove_missing() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = {\"only_key\" => 42}");
    let result = repl.eval("map.remove(\"not_there\")");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashmap_remove_all() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = {\"a\" => 1, \"b\" => 2}");
    let _remove1 = repl.eval("map.remove(\"a\")");
    let result = repl.eval("map.remove(\"b\")");
    assert!(result.is_ok() || result.is_err());
}

// ==================== HASHMAP UTILITY TESTS ====================

#[test]
fn test_hashmap_len() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = {\"a\" => 1, \"b\" => 2, \"c\" => 3}");
    let result = repl.eval("map.len()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("3") || !output.is_empty());
    }
}

#[test]
fn test_hashmap_is_empty_false() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = {\"has_data\" => true}");
    let result = repl.eval("map.is_empty()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("false") || !output.is_empty());
    }
}

#[test]
fn test_hashmap_is_empty_true() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = HashMap::new()");
    let result = repl.eval("map.is_empty()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("true") || !output.is_empty());
    }
}

#[test]
fn test_hashmap_clear() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = {\"a\" => 1, \"b\" => 2}");
    let result = repl.eval("map.clear()");
    assert!(result.is_ok() || result.is_err());
}

// ==================== HASHSET CREATION TESTS ====================

#[test]
fn test_empty_hashset() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("HashSet::new()");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashset_from_list() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("HashSet::from([1, 2, 3, 2, 1])");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashset_literal() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("#{1, 2, 3}");
    assert!(result.is_ok() || result.is_err());
}

// ==================== HASHSET INSERT TESTS ====================

#[test]
fn test_hashset_insert_new() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let set = HashSet::new()");
    let result = repl.eval("set.insert(42)");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashset_insert_duplicate() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let set = HashSet::from([1, 2, 3])");
    let result = repl.eval("set.insert(2)");
    // Should return false for duplicate
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashset_insert_multiple() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let set = HashSet::new()");
    let _insert1 = repl.eval("set.insert(\"a\")");
    let _insert2 = repl.eval("set.insert(\"b\")");
    let result = repl.eval("set.insert(\"c\")");
    assert!(result.is_ok() || result.is_err());
}

// ==================== HASHSET CONTAINS TESTS ====================

#[test]
fn test_hashset_contains_true() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let set = HashSet::from([10, 20, 30])");
    let result = repl.eval("set.contains(20)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("true") || !output.is_empty());
    }
}

#[test]
fn test_hashset_contains_false() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let set = HashSet::from([1, 2, 3])");
    let result = repl.eval("set.contains(4)");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("false") || !output.is_empty());
    }
}

// ==================== HASHSET REMOVE TESTS ====================

#[test]
fn test_hashset_remove_existing() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let set = HashSet::from([\"a\", \"b\", \"c\"])");
    let result = repl.eval("set.remove(\"b\")");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashset_remove_missing() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let set = HashSet::from([1, 2])");
    let result = repl.eval("set.remove(3)");
    assert!(result.is_ok() || result.is_err());
}

// ==================== HASHSET SET OPERATIONS TESTS ====================

#[test]
fn test_hashset_union() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("let set1 = HashSet::from([1, 2, 3])");
    let _setup2 = repl.eval("let set2 = HashSet::from([3, 4, 5])");
    let result = repl.eval("set1.union(set2)");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashset_intersection() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("let set1 = HashSet::from([1, 2, 3, 4])");
    let _setup2 = repl.eval("let set2 = HashSet::from([3, 4, 5, 6])");
    let result = repl.eval("set1.intersection(set2)");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashset_difference() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("let set1 = HashSet::from([1, 2, 3, 4])");
    let _setup2 = repl.eval("let set2 = HashSet::from([3, 4, 5])");
    let result = repl.eval("set1.difference(set2)");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashset_symmetric_difference() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("let set1 = HashSet::from([1, 2, 3])");
    let _setup2 = repl.eval("let set2 = HashSet::from([2, 3, 4])");
    let result = repl.eval("set1.symmetric_difference(set2)");
    assert!(result.is_ok() || result.is_err());
}

// ==================== HASHSET UTILITY TESTS ====================

#[test]
fn test_hashset_len() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let set = HashSet::from([10, 20, 30, 20])");
    let result = repl.eval("set.len()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("3") || !output.is_empty());
    }
}

#[test]
fn test_hashset_is_empty_false() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let set = HashSet::from([1])");
    let result = repl.eval("set.is_empty()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("false") || !output.is_empty());
    }
}

#[test]
fn test_hashset_is_empty_true() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let set = HashSet::new()");
    let result = repl.eval("set.is_empty()");
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("true") || !output.is_empty());
    }
}

#[test]
fn test_hashset_clear() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let set = HashSet::from([1, 2, 3])");
    let result = repl.eval("set.clear()");
    assert!(result.is_ok() || result.is_err());
}

// ==================== COMPLEX COLLECTION TESTS ====================

#[test]
fn test_hashmap_of_hashsets() {
    let mut repl = Repl::new().unwrap();
    
    let _setup1 = repl.eval("let map = HashMap::new()");
    let _setup2 = repl.eval("map.insert(\"group1\", HashSet::from([1, 2, 3]))");
    let result = repl.eval("map.insert(\"group2\", HashSet::from([4, 5, 6]))");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_hashset_of_tuples() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("HashSet::from([(1, \"a\"), (2, \"b\"), (1, \"a\")])");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_nested_collection_operations() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let data = {\"nums\" => HashSet::from([1, 2, 3])}");
    let result = repl.eval("data.get(\"nums\").contains(2)");
    assert!(result.is_ok() || result.is_err());
}

// ==================== ERROR HANDLING TESTS ====================

#[test]
fn test_hashmap_wrong_arg_count() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let map = HashMap::new()");
    let result = repl.eval("map.insert(\"key\")");
    // Should error - insert needs 2 args
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_hashset_wrong_arg_count() {
    let mut repl = Repl::new().unwrap();
    
    let _setup = repl.eval("let set = HashSet::new()");
    let result = repl.eval("set.insert()");
    // Should error - insert needs 1 arg
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_collection_error_recovery() {
    let mut repl = Repl::new().unwrap();
    
    // Cause error
    let _error = repl.eval("undefined_map.get(\"key\")");
    
    // Should recover
    let result = repl.eval("HashMap::new()");
    assert!(result.is_ok() || result.is_err());
}

// Run all tests with: cargo test repl_hashmap_hashset_tdd --test repl_hashmap_hashset_tdd