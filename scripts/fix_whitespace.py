import re
import sys

for filepath in sys.argv[1:]:
    with open(filepath, 'rb') as f:
        data = f.read()
    
    # Convert CRLF to LF
    data = data.replace(b'\r\n', b'\n')
    text = data.decode('utf-8', errors='replace')
    
    lines = text.splitlines(keepends=True)
    
    fixed = []
    for i, line in enumerate(lines):
        # Strip trailing whitespace from ALL lines (including blank lines)
        stripped = line.rstrip('\n')
        if stripped == '' and i > 0:
            # Blank lines get NO indentation (matching SDKCoverageTests.cs pattern)
            fixed.append('\n')
        else:
            # Remove trailing whitespace from content lines
            normalized = stripped.rstrip() + '\n'
            fixed.append(normalized)
    
    content = ''.join(fixed)
    # Ensure file ends with newline
    if content and not content.endswith('\n'):
        content += '\n'
    with open(filepath, 'w', newline='', encoding='utf-8') as f:
        f.write(content)
    print(f'Fixed {filepath}')
