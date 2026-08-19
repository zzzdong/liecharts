import re
c = open('../axis_overflow.svg', encoding='utf-8').read()
# 水平网格线（split line）通常是 <line x1=.. y1=.. x2=.. y2=..>，y 相同为水平
lines = re.findall(r'<line x1="([\d.]+)" y1="([\d.]+)" x2="([\d.]+)" y2="([\d.]+)"[^>]*>', c)
print("=== horizontal grid lines (x1->x2, same y) ===")
for x1, y1, x2, y2 in lines:
    if abs(float(y1)-float(y2)) < 0.5:
        print(f"  x1={x1} y={y1} x2={x2}")
# Y轴轴线（垂直）
print("=== vertical lines (axis) ===")
for x1, y1, x2, y2 in lines:
    if abs(float(x1)-float(x2)) < 0.5:
        print(f"  x={x1} y1={y1} y2={y2}")
