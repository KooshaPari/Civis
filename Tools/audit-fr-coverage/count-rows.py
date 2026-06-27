"""Count FR-CIV-* rows in fr-emergence-matrix.md + verify batch rows landed."""
import re, sys
path = r"C:\Users\koosh\_civwork\docs\traceability\fr-emergence-matrix.md"
with open(path, encoding="utf-8") as f:
    content = f.read()
rows = re.findall(r"^\| FR-CIV-[\w-]+", content, re.MULTILINE)
print(f"total_rows: {len(rows)}")
print(f"first_5: {rows[:5]}")
print(f"last_5: {rows[-5:]}")
emergence_100 = [r for r in rows if re.match(r"\| FR-CIV-EMERGENCE-1\d\d\|", r) or re.match(r"\| FR-CIV-EMERGENCE-2\d\d\|", r)]
print(f"emergence_100_to_249_count: {len(emergence_100)}")
int_rows = [r for r in rows if "0100-int" in r]
print(f"int_rows: {int_rows}")
print(f"file_size_kb: {len(content)//1024}")