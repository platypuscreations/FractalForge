# SPDX-License-Identifier: GPL-3.0-or-later
# Scores the light-colour formula of docs/specs/3d-lighting.md against every
# flat-surface case (flat100.gif / flat0.gif, filltype=5, rotation=0/0/0) in
# corpus/xfractint/3d-lighting/cases.txt.  Own code.
# Usage: python3 -I score_flat.py CORPUSDIR
import sys, os, math, collections
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gif import read_gif
from sim import light_colour
d = sys.argv[1]
tot = ok = 0; off = collections.Counter(); worst = []
for line in open(os.path.join(d, 'cases.txt')):
    p = line.split()
    if not p or p[0].startswith('#') or p[1] not in ('flat100.gif', 'flat0.gif'): continue
    kv = dict(a.split('=', 1) for a in p[2:])
    if kv.get('filltype') != '5' or kv.get('rotation') != '0/0/0' or {'stereo', 'ray', 'longitude', 'latitude', 'radius'} & set(kv): continue
    if not os.path.exists(os.path.join(d, p[0] + '.gif')): continue
    px = read_gif(os.path.join(d, p[0] + '.gif'))['pixels']
    c = collections.Counter(px); c.pop(0, None)
    got = c.most_common(1)[0][0]
    L = [int(v) for v in kv.get('lightsource', '1/-1/1').split('/')]
    m = math.sqrt(sum(v * v for v in L)) or 1.0
    pred = light_colour((0.0, 0.0, 1.0), [v / m for v in L], int(kv.get('ambient', 20)))
    tot += 1; ok += pred == got; off[got - pred] += 1
    if abs(got - pred) > 1: print("  outlier", p[0], got, pred)
print('flat cases %d, exact %d (%.1f%%), got-predicted: %s' % (tot, ok, 100.0 * ok / tot, dict(off)))
