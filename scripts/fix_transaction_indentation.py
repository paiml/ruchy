#!/usr/bin/env python3
"""
Fix indentation issues in transaction.rs where functions are not properly indented inside impl blocks
"""

def fix_transaction_file():
    with open('src/runtime/transaction.rs', 'r') as f:
        lines = f.readlines()
    
    fixed_lines = []
    in_impl_block = False
    impl_level = 0
    
    for i, line in enumerate(lines):
        # Check if we're entering an impl block
        if line.startswith('impl '):
            in_impl_block = True
            impl_level = 0
            fixed_lines.append(line)
            continue
        
        # Check if impl block is ending
        if in_impl_block and line == '}\n' and impl_level == 0:
            in_impl_block = False
            fixed_lines.append(line)
            continue
        
        # Fix indentation for functions inside impl blocks  
        if in_impl_block:
            # Track brace depth
            impl_level += line.count('{') - line.count('}')
            
            # Fix pub fn lines that aren't indented
            if line.startswith('pub fn '):
                fixed_lines.append('    ' + line)
            # Fix function bodies that aren't indented
            elif i > 0 and lines[i-1].startswith('pub fn '):
                # This line is the opening brace and body of a function
                if not line.startswith('    '):
                    fixed_lines.append('    ' + line)
                else:
                    fixed_lines.append(line)
            else:
                fixed_lines.append(line)
        else:
            fixed_lines.append(line)
    
    # Write back
    with open('src/runtime/transaction.rs', 'w') as f:
        f.writelines(fixed_lines)
    
    print("Fixed indentation in transaction.rs")

if __name__ == '__main__':
    fix_transaction_file()