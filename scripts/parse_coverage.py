import re

with open('src/Tests/coverage.cobertura.xml') as f:
    content = f.read()

# Find all class elements with line-rate and branch-rate
pattern = r'class name="([^"]*(?:Client|GameClient)[^"]*)"[^>]*line-rate="([^"]+)"[^>]*branch-rate="([^"]+)"'
for m in re.finditer(pattern, content):
    name = m.group(1)
    lr = float(m.group(2)) * 100
    br = float(m.group(3)) * 100
    if lr < 90 or br < 80:
        print(f'{name}: line={lr:.1f}% branch={br:.1f}%')
