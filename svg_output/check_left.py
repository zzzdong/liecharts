import re, glob
for f in sorted(glob.glob('*.svg')):
    c = open(f, encoding='utf-8').read()
    texts = re.findall(r'<text x="(-?[\d.]+)" y="([\d.]+)"[^>]*>([^<]*)</text>', c)
    for x, y, t in texts:
        xf = float(x)
        # 文字左边缘 = x（左对齐/text-anchor未标）或 x - 宽度（右对齐）
        # 简单检查：x 为负 或 x 很小（<10）且不是轴名称旋转
        if xf < 0:
            print(f"{f}: text '{t}' x={x} NEGATIVE")
        elif xf < 12 and 'rotate' not in c[c.rfind(t)-200:c.rfind(t)]:
            print(f"{f}: text '{t}' x={x} very-left")
