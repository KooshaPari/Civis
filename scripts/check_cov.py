import re
with open('src/Tests/coverage.cobertura.xml', 'r', encoding='utf-8', errors='replace') as f:
    content = f.read()

# Find all class elements with any attributes
classes = re.findall(r'<class\s+([^>]+)>', content)
for cls in classes:
    name_match = re.search(r'name="([^"]+)"', cls)
    fname_match = re.search(r'filename="([^"]+)"', cls)
    lr_match = re.search(r'line-rate="([^"]+)"', cls)
    lc_match = re.search(r'lines-covered="([^"]+)"', cls)
    lv_match = re.search(r'lines-valid="([^"]+)"', cls)
    if name_match and fname_match:
        name = name_match.group(1)
        fname = fname_match.group(1)
        lr = float(lr_match.group(1)) * 100 if lr_match else 0
        lc = lc_match.group(1) if lc_match else '0'
        lv = lv_match.group(1) if lv_match else '0'
        if 'Bridge/Client' in fname or 'SDK' in fname.split('/')[-1].split('.')[0]:
            print(f'{name} ({fname}): {lr:.1f}% ({lc}/{lv})')
