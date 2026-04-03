import re

with open('src/Tests/coverage.cobertura.xml') as f:
    content = f.read()

# Find uncovered lines in specific classes
targets = [
    'DINOForge.Bridge.Client.GameClient/&lt;ConnectAsync&gt;d__15',
    'DINOForge.Bridge.Client.GameProcessManager'
]

for target in targets:
    # Find the class element
    class_pattern = rf'<class name="{re.escape(target)}"[^>]*>(.*?)</class>'
    m = re.search(class_pattern, content, re.DOTALL)
    if not m:
        print(f"Class {target} not found")
        continue
    
    class_content = m.group(1)
    
    # Find all lines
    line_pattern = r'<line number="(\d+)" hits="(\d+)"'
    uncovered = []
    for lm in re.finditer(line_pattern, class_content):
        lineno, hits = int(lm.group(1)), int(lm.group(2))
        if hits == 0:
            uncovered.append(lineno)
    
    print(f"\n{target.split('/')[-1]}: {len(uncovered)} uncovered lines: {uncovered}")
