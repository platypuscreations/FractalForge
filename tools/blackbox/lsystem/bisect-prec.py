# SPDX-License-Identifier: GPL-3.0-or-later
# Bisection version of gen-prec.py: for each target column M (400x300 image,
# same frame as gen-prec.py) finds the offset delta at which XFractint's
# rounding of the segment end flips from M to M+1, i.e. the error of its
# arithmetic at that point.  Runs XFractint (amd64 image) through
# run-case.sh; writes one line per M to stdout:
#   M  delta_lo  delta_hi  (the flip lies between them)
#   python3 -I bisect-prec.py OUTDIR M1 M2 ...
import os, sys, subprocess, concurrent.futures as cf
from fractions import Fraction as Fr
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(here, '..', 'resume'))
from gif import read_gif
out = sys.argv[1]; Ms = [int(x) for x in sys.argv[2:]]
os.makedirs(out, exist_ok=True)
def a_for(m, d):
    return (Fr(11, 10) * (m + Fr(1, 2) + d) - Fr(4395, 100)) / 390
def run(m, d, tag):
    txt = f'{float(a_for(m, d)):.17f}'
    fn = f'b{m}.l'
    w = os.path.join(out, f'{tag}')
    os.makedirs(w, exist_ok=True)
    open(os.path.join(w, fn), 'w').write(f'e {{\n  angle 4\n  axiom [@{txt}F][|@0.1G]G\n}}\n')
    subprocess.run(['docker', 'run', '--rm', '-v', f'{w}:/work', 'fractalforge-blackbox', 'type=lsystem',
                    f'lfile={fn}', 'lname=e', 'params=0', 'viewwindows=2', 'savename=ff.gif'],
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    G = read_gif(os.path.join(w, 'ff.gif')); W = G['w']; px = G['pixels']
    xs = [i % W for i, v in enumerate(px) if v]
    for f in os.listdir(w): os.remove(os.path.join(w, f))
    os.rmdir(w)
    return max(xs), txt
def bisect(m):
    lo, hi = Fr(-2, 100000), Fr(2, 100000)
    log = []
    for i in range(16):
        mid = (lo + hi) / 2
        r, txt = run(m, mid, f'w{m}-{i}')
        log.append((float(mid), txt, r))
        if r == m + 1: hi = mid
        else: lo = mid
    return m, float(lo), float(hi), log
with cf.ThreadPoolExecutor(max_workers=10) as ex:
    for m, lo, hi, log in ex.map(bisect, Ms):
        print(m, f'{lo:.3e}', f'{hi:.3e}', flush=True)
