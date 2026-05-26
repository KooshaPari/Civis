import os

snapshots_dir = 'src/Tests/Snapshots'
for fname in os.listdir(snapshots_dir):
    fpath = os.path.join(snapshots_dir, fname)
    with open(fpath, 'rb') as f:
        data = f.read()
    # Check for trailing whitespace
    lines = data.split(b'\n')
    has_trailing = any(line.rstrip() != line.rstrip(b' \t') for line in lines[:-1])
    has_crlf = b'\r\n' in data
    print(f'{fname}: len={len(data)}, trailing_ws={has_trailing}, crlf={has_crlf}')
