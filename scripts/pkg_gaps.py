import xml.etree.ElementTree as ET
import glob

files = glob.glob(r'C:\Users\koosh\Dino\src\Tests\TestResults\*\coverage.cobertura.xml')
latest = max(files, key=lambda f: __import__('os').path.getmtime(f))
print(f'File: {latest}\n')

tree = ET.parse(latest)
root = tree.getroot()

packages = root.findall('packages/package')
print(f'All packages: {[p.get("name") for p in packages]}')

for pkg in packages:
    pname = pkg.get('name', '')
    classes = pkg.findall('.//class')
    if not classes:
        print(f'{pname}: 0 classes found')
        continue
    total = covered = 0
    classes_info = []
    for cls in classes:
        cname = cls.get('name', '')
        lines_elem = cls.find('lines')
        if lines_elem is None:
            continue
        cls_total = cls_covered = 0
        for line in lines_elem.findall('line'):
            cls_total += 1
            if int(line.get('hits', 0)) > 0:
                cls_covered += 1
        total += cls_total
        covered += cls_covered
        pct = (cls_covered/float(cls_total)*100) if cls_total > 0 else 100.0
        if pct < 85.0 and cls_total > 10:
            classes_info.append((pct, cls_covered, cls_total, cname))

    overall = covered/float(total)*100 if total > 0 else 100.0
    gap85 = int(total * 0.85) - covered
    print(f'\n=== {pname}: {overall:.1f}% ({covered}/{total}) GAP={gap85} ===')
    classes_info.sort()
    for pct, c, t, name in classes_info[:20]:
        print(f'  {pct:.0f}% [{c}/{t}]: {name}')
    if len(classes_info) > 20:
        print(f'  ... and {len(classes_info)-20} more classes below 85%')
