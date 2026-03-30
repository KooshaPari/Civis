import xml.etree.ElementTree as ET
import glob

files = glob.glob(r'C:\Users\koosh\Dino\src\Tests\TestResults\*\coverage.cobertura.xml')
latest = max(files, key=lambda f: __import__('os').path.getmtime(f))
print(f'Coverage file: {latest}\n')

tree = ET.parse(latest)
root = tree.getroot()

print(f'Overall: {float(root.get("line-rate"))*100:.1f}% line, {float(root.get("branch-rate"))*100:.1f}% branch')
print(f'Lines: {root.get("lines-covered")}/{root.get("lines-valid")}\n')

packages = root.findall('packages/package')
print(f'Packages ({len(packages)}):')
for p in packages:
    name = p.get('name', '')
    lr = float(p.get('line-rate') or 0)
    br = float(p.get('branch-rate') or 0)
    lc = p.get('lines-covered') or '0'
    lv = p.get('lines-valid') or '0'
    pct = lr*100
    flag = ' <<<' if pct < 85 else ''
    print(f'  {pct:.1f}% LR / {br*100:.1f}% BR [{lc}/{lv}]: {name}{flag}')
