# SPDX-License-Identifier: GPL-3.0-or-later
# Copies a saved GIF and paints marker pixels (colour 7) so a reload test can
# show whether XFractint keeps or clears the loaded pixels.  Every even column
# of row R1 and columns 0-59 of row R2 are painted.  The extension blocks are
# copied unchanged.
#   python3 -I mark.py IN.gif OUT.gif [R1 R2]
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'resume'))
import gif
src, dst = sys.argv[1], sys.argv[2]
r1, r2 = (int(sys.argv[3]), int(sys.argv[4])) if len(sys.argv) > 4 else (290, 100)
G = gif.read_gif(src); W, H = G['w'], G['h']; px = bytearray(G['pixels'])
for x in range(0, W, 2): px[r1 * W + x] = 7
for x in range(0, 60): px[r2 * W + x] = 7
order = [k for k in G['order'] if k.startswith('fractint')]
gif.write_gif(dst, W, H, G['gct'], bytes(px), [(k, G['exts'][k]) for k in order])
