#!/usr/bin/env python3
import re, json
from pathlib import Path
from datetime import datetime

def load_test_file(path):
    try:
        with open(path, 'r', encoding='utf-8') as f:
            return f.read()
    except:
        return ""

def strip_comments_and_blanks(lines):
    result = []
    for line in lines:
        stripped = line.strip()
        if not stripped or stripped.startswith('//'):
            continue
        result.append(line)
    return result

def extract_method_bodies(content, filename):
    methods = []
    fact_pattern = r'\[(?:Fact|Theory).*?\]\s*(?:public\s+)?(?:async\s+)?(?:void|Task|int|bool).*?(\w+)\s*\((.*?)\)\s*\{(.*?)(?=\n\s*(?:\[|public|private|protected|internal|})\s*$)'
    
    for match in re.finditer(fact_pattern, content, re.DOTALL | re.MULTILINE):
        method_name = match.group(1)
        body = match.group(3)
        start_line = content[:match.start()].count('\n') + 1
        methods.append({
            'name': method_name,
            'body': body,
            'line': start_line,
            'filename': filename
        })
    return methods

def check_p1_pattern(body):
    return bool(re.search(r'Assert\.(True|False)\s*\(\s*(true|false)\s*(?:,|;|\))', body, re.IGNORECASE))

def check_p2_pattern(body):
    return bool(re.search(r'Assert\.NotNull\s*\(\s*new\s+', body) and re.search(r'\.Should\(\)\.NotBeNull\(\)', body))

def check_p3_pattern(body):
    return bool(re.search(r'if\s*\(\s*!\s*(?:_initialized|fixture\.IsInitialized|_fixture\.GameAvailable|initialized)\s*\)\s*\{', body, re.IGNORECASE))

def is_mock_theater(body):
    lines = body.split('\n')
    lines = strip_comments_and_blanks(lines)
    code_lines = [l for l in lines if l.strip() not in ('{', '}', '};')]
    
    if len(code_lines) > 5:
        return False, None
    
    assert_count = len(re.findall(r'(?:Assert|Should)\s*\.', body))
    if assert_count != 1:
        return False, None
    
    if check_p1_pattern(body):
        return True, "P1"
    if check_p2_pattern(body):
        return True, "P2"
    if check_p3_pattern(body):
        return True, "P3"
    return False, None

def main():
    repo_root = Path('C:/Users/koosh/Dino')
    test_dir = repo_root / 'src' / 'Tests'
    output_dir = repo_root / 'docs' / 'test-results'
    output_dir.mkdir(parents=True, exist_ok=True)
    
    mock_theater_items = []
    total_methods = 0
    
    for cs_file in sorted(test_dir.rglob('*.cs')):
        content = load_test_file(cs_file)
        if not content:
            continue
        
        methods = extract_method_bodies(content, str(cs_file.relative_to(repo_root)))
        
        for method in methods:
            total_methods += 1
            is_mock, pattern = is_mock_theater(method['body'])
            
            if is_mock:
                snippet = method['body'].strip()[:120].replace('\n', ' ')
                mock_theater_items.append({
                    'file': method['filename'],
                    'method': method['name'],
                    'line': method['line'],
                    'pattern': pattern,
                    'snippet': snippet
                })
    
    result = {
        'audit_utc': datetime.utcnow().isoformat() + 'Z',
        'total_test_methods_scanned': total_methods,
        'mock_theater_count': len(mock_theater_items),
        'items': mock_theater_items
    }
    
    output_file = output_dir / 'mock-theater-strict-enumeration.json'
    with open(output_file, 'w') as f:
        json.dump(result, f, indent=2)
    
    print(f"Scanned {total_methods} test methods")
    print(f"Found {len(mock_theater_items)} mock-theater tests")
    print(f"Written to {output_file}")
    
    if mock_theater_items:
        ex = mock_theater_items[0]
        print(f"Example: {ex['file']}:{ex['line']} {ex['method']} ({ex['pattern']})")

if __name__ == '__main__':
    main()
