# SPDX-License-Identifier: GPL-3.0-or-later
# Renders every image case of the L-system corpus with lsysmodel.py and compares
# the result with XFractint's saved pixels.
#   python3 -I verify.py CORPUSDIR [round ...]
# CORPUSDIR holds one folder per round (NAME.gif, NAME.stopmsg.txt); the
# arguments of each case are read from cases-<round>.txt next to this script.
# Skipped: cases that wrote a stop message or no image, cases whose .l file
# is not the one XFractint read (file look-up tests "f-lk-*"), and orders
# above MAXORDER (default 8; the model is slow) or matching the regular
# expression in SKIP.  Prints one line per
# mismatching case and a total.
import os, sys, re
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
sys.path.insert(0, os.path.join(here, '..', 'resume'))
sys.dont_write_bytecode = True
from gif import read_gif
import lsysmodel as model

root = sys.argv[1]
rounds = sys.argv[2:] or ['seg', 'rich', 'prec', 'sem', 'ext', 'mv', 'deg', 'deg2', 'arr', 'misc',
                          'chars', 'args', 'fmt', 'fmt2', 'fmt3', 'lhs']
maxorder = int(os.environ.get('MAXORDER', '8'))
skipre = re.compile(os.environ['SKIP']) if os.environ.get('SKIP') else None
inputs = os.path.join(here, 'inputs')
files = {f.lower(): f for f in os.listdir(inputs)}
ok = bad = skipped = 0
for r in rounds:
    for line in open(os.path.join(here, f'cases-{r}.txt')):
        if line.startswith('#') or not line.strip():
            continue
        t = line.split(); name = t[0]
        a = dict(x.split('=', 1) for x in t[1:] if '=' in x and not x.startswith('only='))
        g = os.path.join(root, r, name + '.gif')
        if (not os.path.exists(g) or os.path.exists(os.path.join(root, r, name + '.stopmsg.txt'))
                or name.startswith('f-lk-') or 'lfile' not in a or 'lname' not in a):
            skipped += 1; continue
        lf = a['lfile'].lower()
        if not lf.endswith('.l'): lf += '.l'
        if lf not in files:
            skipped += 1; continue
        order = float(a.get('params', '2').split('/')[0])
        if order > maxorder or (skipre and skipre.search(name)):
            skipped += 1; continue
        G = read_gif(g)
        text = open(os.path.join(inputs, files[lf]), newline='').read()
        try:
            img = model.render(text, a['lname'], order, G['w'], G['h'])
        except KeyError:
            skipped += 1; continue
        n = sum(1 for p, q in zip(img, G['pixels']) if p != q)
        if n:
            bad += 1; print(f'{r}/{name}: {n} pixels differ', flush=True)
        else:
            ok += 1
print(f'identical {ok}, different {bad}, skipped {skipped}')
