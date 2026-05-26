import re

with open('src/Tests/coverage.cobertura.xml') as f:
    content = f.read()

# Find branch coverage for SendRequestCoreAsync
target = 'SendRequestCoreAsync'
for m in re.finditer(r'class name="([^"]*)"[^>]*line-rate="([^"]+)"[^>]*branch-rate="([^"]+)"', content):
    name = m.group(1)
    if 'SendRequestCoreAsync' in name:
        print(f"\n{name}:")
        print(f"  line={float(m.group(2))*100:.1f}%, branch={float(m.group(3))*100:.1f}%")
        
        # Find the class element
        class_pattern = rf'<class name="{re.escape(name)}"[^>]*>(.*?)</class>'
        cm = re.search(class_pattern, content, re.DOTALL)
        if cm:
            cc = cm.group(1)
            
            # Count branch points
            branch_points = re.findall(r'<branchpoint .*?/>', cc, re.DOTALL)
            print(f"  Total branchpoints: {len(branch_points)}")
            
            # Find uncovered branches
            uncovered = re.findall(r'<branchpoint .*?hits="0"[^>]*/>', cc, re.DOTALL)
            print(f"  Uncovered branches: {len(uncovered)}")
            for ub in uncovered:
                # Extract line and type
                lm = re.search(r'line="(\d+)"', ub)
                type_m = re.search(r'type="([^"]+)"', ub)
                offset_m = re.search(r'offset="([^"]+)"', ub)
                print(f"    line={lm.group(1) if lm else '?'} type={type_m.group(1) if type_m else '?'} offset={offset_m.group(1) if offset_m else '?'}")
