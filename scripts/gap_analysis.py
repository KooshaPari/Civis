import xml.etree.ElementTree as ET
import glob

files = glob.glob(r'C:\Users\koosh\Dino\src\Tests\TestResults\*\coverage.cobertura.xml')
latest = max(files, key=lambda f: __import__('os').path.getmtime(f))
print(f'File: {latest}\n')

tree = ET.parse(latest)
root = tree.getroot()

# Count lines per package using class-level data
packages_data = {}
for pkg in root.findall('packages/package'):
    pname = pkg.get('name', '')
    total = covered = 0
    for cls in pkg.findall('.//class'):
        for line in cls.findall('.//line'):
            total += 1
            if int(line.get('hits', 0)) > 0:
                covered += 1
    if total > 0:
        packages_data[pname] = (covered, total, covered/total*100)

print(f'{'Package':<45} {'Covered':>8} {'Total':>8} {'Pct':>8}')
print('-' * 73)
for pname, (c, t, pct) in sorted(packages_data.items(), key=lambda x: x[1][2]):
    gap = int(t * 0.85) - c
    flag = ' *** GAP:' + str(gap) if gap > 0 else ' OK'
    print(f'{pname:<45} {c:>8} {t:>8} {pct:>7.1f}%{flag}')
print()

total_cov = sum(d[0] for d in packages_data.values())
total_all = sum(d[1] for d in packages_data.values())
print(f'TOTAL: {total_cov}/{total_all} = {total_cov/total_all*100:.1f}%')
target = int(total_all * 0.85)
print(f'Target 85%: {target} lines needed, {target - total_cov} more lines required')
