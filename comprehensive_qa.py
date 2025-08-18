#!/usr/bin/env python3
"""
Comprehensive Grammar Coverage Test
Tests all implemented features in the REPL
"""
import subprocess

def test_expr(expr, expected=None, should_fail=False):
    input_text = f"{expr}\n:quit\n"
    
    result = subprocess.run(
        ["cargo", "run", "-p", "ruchy-cli", "--", "repl"],
        input=input_text,
        text=True,
        capture_output=True,
        timeout=10
    )
    
    output = result.stdout + result.stderr
    success = result.returncode == 0 and "Error" not in output
    
    if should_fail:
        return not success
    
    if success and expected:
        # Extract actual result
        lines = output.split('\n')
        for line in lines:
            line = line.strip()
            if line and not line.startswith('Welcome') and not line.startswith('Type') and not line.startswith('Finished') and not line.startswith('Running') and not line.startswith('Compiling') and line != 'Goodbye!':
                return line == expected
        return False
    
    return success

def run_comprehensive_test():
    print("🎯 Comprehensive Ruchy REPL Grammar Coverage Test")
    print("=" * 60)
    
    test_categories = [
        ("Core Literals", [
            ("42", "42"),
            ("3.14", "3.14"), 
            ('"hello"', '"hello"'),
            ("true", "true"),
            ("false", "false"),
            ("()", "()"),
        ]),
        
        ("Arithmetic Operations", [
            ("1 + 2", "3"),
            ("10 - 3", "7"),
            ("4 * 5", "20"),
            ("15 / 3", "5"),
            ("7 % 3", "1"),
            ("2 ** 3", "8"),
            ("3 ** 2", "9"),
            ("2.5 ** 2.0", "6.25"),
        ]),
        
        ("String Operations", [
            ('"a" + "b"', '"ab"'),
            ('"hello" + " " + "world"', '"hello world"'),
        ]),
        
        ("Comparison Operations", [
            ("5 == 5", "true"),
            ("3 != 4", "true"),
            ("2 < 5", "true"),
            ("8 > 3", "true"),
            ("5 <= 5", "true"),
            ("10 >= 8", "true"),
            ('"abc" == "abc"', "true"),
            ('"x" != "y"', "true"),
        ]),
        
        ("Boolean Logic", [
            ("true && false", "false"),
            ("true || false", "true"),
            ("!true", "false"),
            ("!false", "true"),
        ]),
        
        ("Unary Operations", [
            ("-42", "-42"),
            ("-(-5)", "5"),
        ]),
        
        ("Operator Precedence", [
            ("2 + 3 * 4", "14"),
            ("(2 + 3) * 4", "20"),
            ("1 + 2 < 5", "true"),
            ("2 ** 3 + 1", "9"),
            ("(1 + 2) ** 2", "9"),
        ]),
        
        ("Control Flow", [
            ("if true { 1 } else { 2 }", "1"),
            ("if false { 10 } else { 20 }", "20"),
            ('if 2 > 1 { "yes" } else { "no" }', '"yes"'),
            ('if "a" + "b" == "ab" { 100 } else { 200 }', "100"),
        ]),
        
        ("Block Expressions", [
            ("{ 1; 2; 3 }", "3"),
            ("{ 5 + 5 }", "10"),
            ("{ if true { 1 + 2 } else { 3 + 4 } }", "3"),
        ]),
        
        ("Variable Assignment", [
            ("x = 42", "42"),
            ('name = "Alice"', '"Alice"'),
            ("result = 2 ** 3 + 1", "9"),
        ]),
        
        ("Range Expressions", [
            ("1..10", '"1..10"'),
            ("0..100", '"0..100"'),
        ]),
        
        ("List Expressions", [
            ("[1, 2, 3]", "1"),  # Returns first element for demo
            ("[true, false]", "true"),
        ]),
        
        ("Function Definitions", [
            ("fun add(x, y) { x + y }", '"fn add(x, y)"'),
            ('fun greet(name) { "Hello " + name }', '"fn greet(name)"'),
        ]),
        
        ("Lambda Expressions", [
            ("|x| x * 2", '"|x| <body>"'),
            ("|a, b| a + b", '"|a, b| <body>"'),
        ]),
        
        ("Error Handling", [
            ("10 / 0", None, True),  # Should fail
            ("undefined_variable", None, True),  # Should fail
            ("1 + true", None, True),  # Should fail
        ]),
    ]
    
    total_passed = 0
    total_tests = 0
    category_results = []
    
    for category_name, tests in test_categories:
        print(f"\n📋 {category_name}:")
        passed = 0
        
        for test_case in tests:
            if len(test_case) == 3:
                expr, expected, should_fail = test_case
            else:
                expr, expected = test_case
                should_fail = False
                
            success = test_expr(expr, expected, should_fail)
            
            if success:
                print(f"  ✅ {expr}")
                passed += 1
            else:
                print(f"  ❌ {expr}")
            
            total_tests += 1
        
        total_passed += passed
        category_results.append((category_name, passed, len(tests)))
        print(f"     {passed}/{len(tests)} passed ({(passed/len(tests)*100):.0f}%)")
    
    # Final summary
    print("\n" + "=" * 60)
    print("📊 COMPREHENSIVE RESULTS")
    print("=" * 60)
    
    for name, passed, total in category_results:
        percentage = (passed/total*100) if total > 0 else 0
        status = "🟢" if percentage == 100 else "🟡" if percentage >= 80 else "🔴"
        print(f"{status} {name:25s}: {passed:2d}/{total:2d} ({percentage:3.0f}%)")
    
    overall_percentage = (total_passed/total_tests*100) if total_tests > 0 else 0
    
    print(f"\n🎯 OVERALL COVERAGE: {total_passed}/{total_tests} ({overall_percentage:.1f}%)")
    
    if overall_percentage >= 95:
        print("🏆 OUTSTANDING! Near-complete grammar coverage!")
    elif overall_percentage >= 85:
        print("🚀 EXCELLENT! Comprehensive language support!")
    elif overall_percentage >= 75:
        print("✅ VERY GOOD! Strong core language features!")
    elif overall_percentage >= 60:
        print("👍 GOOD! Solid foundation implemented!")
    else:
        print("🔧 More work needed on core features.")
    
    return total_passed, total_tests

if __name__ == "__main__":
    run_comprehensive_test()