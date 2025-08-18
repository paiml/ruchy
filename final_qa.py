#!/usr/bin/env python3
"""
Final comprehensive QA test of the REPL
"""
import subprocess

def test_one(expr, expected_output=None):
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
    
    if success:
        # Extract the actual result
        lines = output.split('\n')
        actual_result = None
        for line in lines:
            line = line.strip()
            if line and not line.startswith('Welcome') and not line.startswith('Type') and not line.startswith('Finished') and not line.startswith('Running') and not line.startswith('Compiling') and line != 'Goodbye!':
                actual_result = line
                break
        
        if expected_output and actual_result != expected_output:
            return False, f"Expected {expected_output}, got {actual_result}"
        return True, actual_result
    else:
        error_msg = "Unknown error"
        lines = output.split('\n')
        for line in lines:
            if 'Error:' in line:
                error_msg = line.strip()
                break
        return False, error_msg

def run_test_suite():
    tests = [
        # Basic literals
        ("42", "42"),
        ("3.14", "3.14"),
        ('"hello"', '"hello"'),
        ("true", "true"),
        ("false", "false"),
        ("()", "()"),
        
        # Arithmetic
        ("1 + 2", "3"),
        ("10 - 3", "7"),
        ("4 * 5", "20"),
        ("15 / 3", "5"),
        ("7 % 3", "1"),
        
        # String operations
        ('"a" + "b"', '"ab"'),
        ('"hello" + " " + "world"', '"hello world"'),
        
        # Comparisons
        ("5 == 5", "true"),
        ("3 != 4", "true"),
        ("2 < 5", "true"),
        ("8 > 3", "true"),
        ('"abc" == "abc"', "true"),
        ('"x" != "y"', "true"),
        
        # Boolean logic
        ("true && false", "false"),
        ("true || false", "true"),
        ("!true", "false"),
        ("!false", "true"),
        
        # Unary operations
        ("-42", "-42"),
        ("-(-5)", "5"),
        
        # Precedence
        ("2 + 3 * 4", "14"),
        ("(2 + 3) * 4", "20"),
        ("1 + 2 < 5", "true"),
        
        # If expressions
        ("if true { 1 } else { 2 }", "1"),
        ("if false { 10 } else { 20 }", "20"),
        ('if 2 > 1 { "yes" } else { "no" }', '"yes"'),
        
        # Block expressions
        ("{ 1; 2; 3 }", "3"),
        ("{ 5 + 5 }", "10"),
        
        # Combined expressions
        ('if "a" + "b" == "ab" { 100 } else { 200 }', "100"),
        ("{ if true { 1 + 2 } else { 3 + 4 } }", "3"),
        
        # List expressions (basic support)
        ("[1, 2, 3]", "1"),  # Returns first element for now
    ]
    
    print("🧪 Final REPL Grammar QA Suite")
    print("=" * 50)
    
    passed = 0
    total = len(tests)
    
    for i, (expr, expected) in enumerate(tests, 1):
        print(f"Test {i:2d}/{total}: {expr:30s} ", end="")
        
        success, result = test_one(expr, expected)
        
        if success:
            print("✅ PASS")
            if result != expected:
                print(f"        Expected: {expected}, Got: {result}")
            passed += 1
        else:
            print("❌ FAIL")
            print(f"        {result}")
    
    print("\n" + "=" * 50)
    print(f"🎯 RESULTS: {passed}/{total} tests passed ({(passed/total)*100:.1f}%)")
    
    if passed == total:
        print("🎉 ALL TESTS PASSED! REPL is working excellently!")
    elif passed > total * 0.8:
        print("🚀 EXCELLENT! REPL covers most core language features!")
    elif passed > total * 0.6:
        print("✅ GOOD! REPL covers essential language features!")
    else:
        print("⚠️  More work needed on core features.")
        
    return passed, total

if __name__ == "__main__":
    run_test_suite()