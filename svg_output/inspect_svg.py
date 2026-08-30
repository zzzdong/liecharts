#!/usr/bin/env python3
"""解析 liecharts 生成的 SVG，输出文本/线条/矩形元素摘要，用于布局检查。"""
import re
import sys

TEXT_RE = re.compile(r"<text[^>]*>.*?</text>", re.S)
TAG_RE = re.compile(r"<(\w+)((?:[^>\"']|\"[^\"]*\"|'[^']*')*)>")
ATTR_RE = re.compile(r'([\w:-]+)="([^"]*)"')


def attrs(s):
    return dict(ATTR_RE.findall(s))


def content_str(s):
    return re.sub(r"<[^>]+>", "", s).strip()


def main(path, show_rect=False):
    svg = open(path, encoding="utf-8").read()
    print(f"== {path} ==")
    for m in TEXT_RE.finditer(svg):
        a = attrs(m.group(0))
        txt = content_str(m.group(0))
        if not txt:
            continue
        print(
            f"  text ({float(a.get('x', 0)):7.1f},{float(a.get('y', 0)):7.1f})"
            f" size={a.get('font-size','?'):>4} anchor={a.get('text-anchor','-'):>6}"
            f" rotate={a.get('transform','-')[:28]:<28} |{txt}|"
        )
    if show_rect:
        for m in TAG_RE.finditer(svg):
            tag, a = m.group(1), attrs(m.group(2))
            if tag == "line":
                print(
                    f"  line ({float(a['x1']):7.1f},{float(a['y1']):7.1f})"
                    f" -> ({float(a['x2']):7.1f},{float(a['y2']):7.1f})"
                    f" stroke={a.get('stroke','-')} w={a.get('stroke-width','?')}"
                    f" dash={a.get('stroke-dasharray','-')}"
                )


if __name__ == "__main__":
    main(sys.argv[1], show_rect="--rect" in sys.argv)
