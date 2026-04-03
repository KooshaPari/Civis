import re

with open('src/Tests/coverage.cobertura.xml') as f:
    content = f.read()

# Look for SendRequestCoreAsync class and its branch elements
target = 'SendRequestCoreAsync'
start = content.find('SendRequestCoreAsync')
if start < 0:
    print("Not found")
    exit()

# Find the class element start
class_start = content.rfind('<class ', 0, start)
class_end = content.find('</class>', start) + len('</class>')
class_content = content[class_start:class_end]

# Find branch elements
branches = re.findall(r'<branch[^>]*>', class_content)
print(f"Total branch tags: {len(branches)}")
for b in branches:
    print(f"  {b[:200]}")
