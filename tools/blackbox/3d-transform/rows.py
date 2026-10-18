# SPDX-License-Identifier: GPL-3.0-or-later
# Prints, per row, the runs of non-background pixels: row: [x0-x1 colour...]
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gif import read_gif
g = read_gif(sys.argv[1]); w, h, p = g['w'], g['h'], g['pixels']
bg = int(sys.argv[2]) if len(sys.argv) > 2 else 0
for y in range(h):
    runs = []; x = 0
    while x < w:
        c = p[y*w+x]
        if c == bg: x += 1; continue
        x0 = x
        while x < w and p[y*w+x] == c: x += 1
        runs.append('%d-%d:%d' % (x0, x-1, c) if x-1 > x0 else '%d:%d' % (x0, c))
    if runs: print(y, ' '.join(runs))
