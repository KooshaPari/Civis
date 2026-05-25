import xml.etree.ElementTree as ET
import sys

tree = ET.parse('src/Tests/coverage.cobertura.xml')
root = tree.getroot()

def get_uncovered_lines(cls):
    lines_el = cls.find('lines')
    if lines_el is None:
        return []
    uncovered = []
    for line in lines_el.findall('line'):
        hits = int(line.get('hits', 0))
        if hits == 0:
            uncovered.append(int(line.get('number', 0)))
    return uncovered

# Find GameProcessManager state machines
print("=== GameProcessManager state machines ===")
for cls in root.iter('class'):
    cname = cls.get('name', '')
    fname = cls.get('filename', '')
    if 'GameProcessManager' in cname and ('d__' in cname or 'GameProcessManager' == cname.split('/')[0]):
        lines = get_uncovered_lines(cls)
        lines_el = cls.find('lines')
        total = len(lines_el.findall('line')) if lines_el is not None else 0
        print(f'{cname}: uncovered = {sorted(lines)[:30]}')
        print(f'  file: {fname}')

# Find GameClient/<SendRequestCoreAsync>
print()
print("=== GameClient SendRequestCoreAsync ===")
for cls in root.iter('class'):
    cname = cls.get('name', '')
    fname = cls.get('filename', '')
    if 'SendRequestCoreAsync' in cname or 'SendRequestAsync' in cname:
        lines = get_uncovered_lines(cls)
        lines_el = cls.find('lines')
        total = len(lines_el.findall('line')) if lines_el is not None else 0
        print(f'{cname}: uncovered = {sorted(lines)[:30]}')
        print(f'  file: {fname}')
