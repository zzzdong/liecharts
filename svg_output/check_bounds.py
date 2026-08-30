#!/usr/bin/env python3
"""检查 SVG 中所有元素是否越出画布边界（忽略 stroke 宽度容差）。"""
import math
import re
import sys
from pathlib import Path

NUM = r"[-+0-9.eE]+"


def fmt_attr(a):
    return dict(re.findall(r'([\w:-]+)="([^"]*)"', a))


def xform_translate(t):
    m = re.search(r"translate\((%s)[ ,]+(%s)\)" % (NUM, NUM), t)
    if m:
        return float(m.group(1)), float(m.group(2))
    return 0.0, 0.0


def collect(svg):
    """返回 [(tag, pts, transform)]。"""
    out = []
    # 逐个标签扫描（简单 XML，无嵌套同名捕获问题）
    for m in re.finditer(r"<(rect|line|circle|text|path)\b([^>]*)>", svg):
        tag, a = m.group(1), fmt_attr(m.group(2))
        tf = a.get("transform", "")
        tx, ty = xform_translate(tf)
        rot = "rotate" in tf
        if tag == "rect":
            x, y = float(a.get("x", 0)), float(a.get("y", 0))
            w, h = float(a.get("width", 0)), float(a.get("height", 0))
            pts = [(x, y), (x + w, y + h)]
        elif tag == "line":
            pts = [(float(a["x1"]), float(a["y1"])), (float(a["x2"]), float(a["y2"]))]
        elif tag == "circle":
            cx, cy, r = float(a["cx"]), float(a["cy"]), float(a.get("r", 0))
            pts = [(cx - r, cy - r), (cx + r, cy + r)]
        elif tag == "text":
            x, y = float(a.get("x", 0)), float(a.get("y", 0))
            fs = float(a.get("font-size", 12) or 12)
            inner = re.sub(r"<[^>]+>", "", m.group(0)[m.group(0).index(">") + 1 :])
            n = len(inner.strip())
            w = n * fs  # 粗略估宽
            pts = [(x - 2, y - fs), (x + w + 2, y + fs * 0.4)]
        else:
            continue
        if tx or ty:
            pts = [(x + tx, y + ty) for x, y in pts]
        out.append((tag, pts, rot))
    return out


def check(path, tol=2.0):
    svg = Path(path).read_text(encoding="utf-8")
    m = re.search(r'viewBox="0 0 ([\d.]+) ([\d.]+)"', svg)
    if not m:
        return
    W, H = float(m.group(1)), float(m.group(2))
    issues = []
    for tag, pts, rot in collect(svg):
        if rot:
            continue  # 旋转文本包围盒估算不可靠，跳过
        for x, y in pts:
            if x < -tol or y < -tol or x > W + tol or y > H + tol:
                issues.append((tag, x, y))
    if issues:
        uniq = sorted(set((t, round(x, 1), round(y, 1)) for t, x, y in issues))
        print(f"[OVERFLOW] {path} ({W:.0f}x{H:.0f}): {len(issues)} 处越界")
        for t, x, y in uniq[:8]:
            print(f"    {t} at ({x},{y})")


if __name__ == "__main__":
    paths = sys.argv[1:] or [str(p) for p in Path(".").glob("*.svg")]
    for p in sorted(paths):
        check(p)
