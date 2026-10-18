# SPDX-License-Identifier: GPL-3.0-or-later
# Make depth-map GIFs for stereogram probes: keeps the palette and fractint001
# block of a 400x300 XFractint save (TEMPLATE) and replaces the pixels.
#   python3 -I craft-depth.py TEMPLATE.gif OUTDIR
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gifio import read_gif, write_gif
t = read_gif(sys.argv[1]); out = sys.argv[2]
w, h = t['w'], t['h']; assert (w, h) == (400, 300)
ext = [('fractint001', t['exts']['fractint001'])]
def save(name, px): write_gif(os.path.join(out, name), w, h, t['gct'], bytes(px), ext)
# rows: row y has colour y mod 256
save('rows.gif', [y % 256 for y in range(h) for x in range(w)])
# box: background 1, centre rectangle 255
save('box.gif', [255 if 120 <= x < 280 and 90 <= y < 210 else 1 for y in range(h) for x in range(w)])
# mix: runs of random length 1..40 and random colour 0..255 (own 32-bit LCG, seed 12345)
s = 12345
def nxt():
    global s
    s = (s * 1103515245 + 12345) & 0x7fffffff
    return s >> 8
px = []
for y in range(h):
    x = 0
    while x < w:
        n = 1 + nxt() % 40; c = nxt() % 256
        px += [c] * min(n, w - x); x += n
save('mix.gif', px)
# flat images
for c in (0, 1, 128, 255):
    save('flat%d.gif' % c, [c] * (w * h))
# partial colour ranges
save('r64.gif', [64 + y % 128 for y in range(h) for x in range(w)])
save('r10.gif', [10 + y % 100 for y in range(h) for x in range(w)])
# palettes: (a) random unique 6-bit colours (own LCG), (b) bar-colour ties
def pal_bytes(entries):
    return bytes(v for e in entries for v in e)
s = 999
seen = set(); ents = []
while len(ents) < 256:
    s = (s * 1103515245 + 12345) & 0x7fffffff
    e = tuple(((s >> sh) & 63) * 4 + (((s >> sh) & 63) >> 4) for sh in (4, 10, 16))
    if e not in seen: seen.add(e); ents.append(e)
rows_px = [y % 256 for y in range(h) for x in range(w)]
write_gif(os.path.join(out, 'rows-randpal.gif'), w, h, pal_bytes(ents), bytes(rows_px), ext)
ties = [(4 * (i % 32), 4 * (i // 32), 8) for i in range(256)]   # all dark-ish, unique
ties[10] = (0, 255, 255); ties[20] = (255, 255, 0); ties[30] = (170, 170, 170)
write_gif(os.path.join(out, 'rows-tiepal.gif'), w, h, pal_bytes(ties), bytes(rows_px), ext)
# two-level images (0 and 255) used to locate the calibration-bar sample window
def two(name, f): save(name, [255 if f(x, y) else 0 for y in range(h) for x in range(w)])
two('lefthalf.gif', lambda x, y: x < 200)
two('tophalf.gif', lambda x, y: y < 150)
two('row150.gif', lambda x, y: y == 150)
for X0 in (180, 220): two('xge%d.gif' % X0, lambda x, y, X0=X0: x >= X0)
for Y0 in (130, 170): two('yge%d.gif' % Y0, lambda x, y, Y0=Y0: y >= Y0)
two('tyge9.gif', lambda x, y: y >= 9)
# des0..des6 (random rectangles chosen by a search script) are kept only as files.
