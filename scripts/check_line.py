with open('src/Tests/ModelSnapshotTests.cs', 'rb') as f:
    data = f.read()
lines = data.split(b'\n')
print(f'File ends with: {data[-50:]!r}')
print(f'Last char: {data[-1]!r}')
has_crlf = b'\r\n' in data
print(f'Has CRLF: {has_crlf}')
for i in [27, 28, 29, 30, 31]:
    if i < len(lines):
        print(f'Line {i+1}: {lines[i]!r} (len={len(lines[i])})')
