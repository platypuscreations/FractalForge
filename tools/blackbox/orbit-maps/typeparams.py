# SPDX-License-Identifier: GPL-3.0-or-later
# Prints "TYPE MAXITER P1 P2 P3" for a saved GIF (TYPE '-' if libm-orbits.sh
# does not handle it). Own code; used by libm-orbits.sh.
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from orbitsim import load
g = load(sys.argv[1]); f = g['f']; p = f['params']
names = {122: 'martin', 166: 'chip', 167: 'quadruptwo', 168: 'threeply'}
print(names.get(f['type'], '-'), f['maxiter'], repr(p[0]), repr(p[1]), repr(p[2]))
