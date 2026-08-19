import re, sys

f = '../axis_overflow.svg'
c = open(f, encoding='utf-8').read()
# 提取所有 <text x=".." y=".." ...>label</text>，并识别是否是数值标签
texts = re.findall(r'<text x="([-0-9.]+)" y="([-0-9.]+)"[^>]*>([^<]*)</text>', c)
num_labels = [(float(x), float(y), t) for x, y, t in texts if re.fullmatch(r'[\d.\-]+', t.strip())]
print("=== numeric (axis tick) labels ===")
for x, y, t in sorted(num_labels, key=lambda a: a[1]):
    mark = "  <-- OVERFLOW" if x < 0 else ""
    print(f"  x={x:8.1f} y={y:8.1f} text='{t}'{mark}")
print("\n=== all text min x ===")
for x, y, t in texts:
    if float(x) < 0:
        print(f"  NEGATIVE x: text='{t}' x={x}")
