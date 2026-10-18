# SPDX-License-Identifier: GPL-3.0-or-later
# Check every case in corpus/xfractint/stereogram/cases.txt against rdsmodel.py.
#   python3 -I tools/blackbox/stereogram/verify.py [corpus/xfractint/stereogram]
import sys, os
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from rdsmodel import *
root = sys.argv[1] if len(sys.argv) > 1 else os.path.join(here, '../../../corpus/xfractint/stereogram')
for line in open(os.path.join(root, 'cases.txt')):
    if line.startswith('#') or not line.strip():
        continue
    name, depth, wd, eff, bars, seed = line.split()[:6]
    o = read_gif(os.path.join(root, 'out', name + '.gif')); d = read_gif(os.path.join(root, 'depth', depth + '.gif'))
    w, h = o['w'], o['h']
    dep = d['pixels']; pattern = None; note = ''
    if bars == 'gray':
        g = grey_values(d['gct']); dep = [g[c] for c in dep]; bars = 'middle'; note = '(grayscale: approximate model)'
    if seed == '-':
        p = read_gif(os.path.join(root, 'depth', 'pat.gif')); pattern = (p['w'], p['h'], p['pixels']); seed = '1'
    m = stereogram(dep, w, h, int(seed), float(wd), int(eff), A_INCH, bars, d['gct'], pattern=pattern)
    rows = [y for y in range(h) if m[y * w:(y + 1) * w] != list(o['pixels'][y * w:(y + 1) * w])]
    px = sum(m[i] != o['pixels'][i] for i in range(w * h))
    print('%-15s %s  mismatched rows %3d  pixels %6d %s %s' % (name, 'OK ' if not rows else 'BAD', len(rows), px, rows[:6], note))
