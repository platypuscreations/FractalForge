# SPDX-License-Identifier: GPL-3.0-or-later
# Mirror check for the symmetry-qualifier cases (docs/specs/frm-syntax.md).
#   python3 -I symcheck.py DIR REFCASE
# For every GIF in DIR prints: pixels different from REFCASE, and the number
# of pixels that differ from their mirror image about the horizontal centre
# line (xmirror), the vertical centre line (ymirror) and the centre point
# (origin), plus the fractint001 symmetry setting (i16 at 124).
import sys, os, struct
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'resume'))
from gif import read_gif
d, ref = sys.argv[1], sys.argv[2]
def path(n):
    for s in ('.gif', '.ff.gif'):
        if os.path.exists(os.path.join(d, n + s)):
            return os.path.join(d, n + s)
R = read_gif(path(ref))
w, h = R['w'], R['h']
for f in sorted(os.listdir(d)):
    if not f.endswith('.gif'):
        continue
    G = read_gif(os.path.join(d, f)); px = G['pixels']
    diff = sum(a != b for a, b in zip(px, R['pixels']))
    xm = sum(px[r*w+c] != px[(h-1-r)*w+c] for r in range(h) for c in range(w))
    ym = sum(px[r*w+c] != px[r*w+(w-1-c)] for r in range(h) for c in range(w))
    om = sum(px[r*w+c] != px[(h-1-r)*w+(w-1-c)] for r in range(h) for c in range(w))
    sym = struct.unpack('<h', G['exts']['fractint001'][124:126])[0]
    print(f'{f.split(".")[0]:28s} diff={diff:6d} xmirror={xm:6d} ymirror={ym:6d} origin={om:6d} sym124={sym}')
