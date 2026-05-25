import re

with open('src/Tests/coverage.cobertura.xml') as f:
    content = f.read()

# Analyze conditions for SendRequestCoreAsync
targets = ['SendRequestCoreAsync', 'SendRequestAsync', 'ConnectAsync']
for target in targets:
    start = content.find(target)
    if start < 0:
        continue
    class_start = content.rfind('<class ', 0, start)
    class_end = content.find('</class>', start) + len('</class>')
    cc = content[class_start:class_end]
    
    # Find all conditions
    cond_pattern = r'<condition number="(\d+)" type="(\w+)" coverage="(\d+)\.\.'
    conditions = list(re.finditer(cond_pattern, cc))
    
    print(f"\n{target} - {len(conditions)} conditions")
    for m in conditions:
        num, type_, cov = m.group(1), m.group(2), m.group(3)
        cov_int = int(cov)
        # Find the line
        pos = m.start()
        line_start = cc.rfind('<line ', 0, pos)
        line_end = cc.find('>', pos)
        line_tag = cc[line_start:line_end]
        lm = re.search(r'number="(\d+)"', line_tag)
        lineno = lm.group(1) if lm else '?'
        if cov_int < 100:
            print(f"  Line {lineno} condition {num} type={type_} coverage={cov_int}%")
