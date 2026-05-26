import re

with open('src/Tests/coverage.cobertura.xml') as f:
    content = f.read()

# Find all GameClient related classes
pattern = r'class name="([^"]*(?:GameClient|Bridge\.Client)[^"]*)"[^>]*line-rate="([^"]+)"[^>]*branch-rate="([^"]+)"'
for m in re.finditer(pattern, content):
    name = m.group(1)
    lr = float(m.group(2)) * 100
    br = float(m.group(3)) * 100
    print(f'{name}: line={lr:.1f}% branch={br:.1f}%')
