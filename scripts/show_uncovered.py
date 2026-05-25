import xml.etree.ElementTree as ET
import glob

files = glob.glob(r'C:\Users\koosh\Dino\src\Tests\TestResults\*\coverage.cobertura.xml')
latest = max(files, key=lambda f: __import__('os').path.getmtime(f))
print(f'Coverage file: {latest}\n')

tree = ET.parse(latest)
root = tree.getroot()

# Show structure
print(f'Root: {root.tag}')
print(f'Children: {[c.tag for c in root]}')

packages = root.find('packages')
if packages:
    pkg_list = packages.findall('package')
    print(f'Packages: {len(pkg_list)}')
    for p in pkg_list:
        pname = p.get('name')
        classes = p.find('classes')
        if classes:
            cls_list = classes.findall('class')
            print(f'  {pname}: {len(cls_list)} classes')
            for c in cls_list[:3]:
                cname = c.get('name')
                lines = c.find('lines')
                if lines:
                    uncovered = [l for l in lines.findall('line') if int(l.get('hits', 0)) == 0]
                    if uncovered:
                        print(f'    {cname}: {len(uncovered)} uncovered lines')
