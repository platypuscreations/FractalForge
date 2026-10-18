# SPDX-License-Identifier: GPL-3.0-or-later
# Prints the non-zero pixels of a saved GIF as text (own analysis helper).
#   python3 -I ascii.py FILE.gif [x0 y0 x1 y1]
# '.' = colour 0; other colours print as their index in base 36 (mod 36).
import sys, os
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'resume'))
from gif import read_gif
G = read_gif(sys.argv[1]); W, H = G['w'], G['h']; px = G['pixels']
nz = [i for i, v in enumerate(px) if v]
if len(sys.argv) > 2:
    x0, y0, x1, y1 = map(int, sys.argv[2:6])
else:
    xs = [i % W for i in nz]; ys = [i // W for i in nz]
    x0, y0, x1, y1 = min(xs), min(ys), max(xs), max(ys)
D = '0123456789abcdefghijklmnopqrstuvwxyz'
print(f'# x {x0}..{x1}, y {y0}..{y1}')
for y in range(y0, y1 + 1):
    print(f'{y:4d} ' + ''.join('.' if px[y*W+x] == 0 else D[px[y*W+x] % 36] for x in range(x0, x1 + 1)))
