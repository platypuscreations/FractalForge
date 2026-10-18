# SPDX-License-Identifier: GPL-3.0-or-later
"""Arithmetic-order search for a 3D orbit type (own code).

Usage (in an empty work directory):
  python3 -I .../ordersearch/mk_orbit3d.py IMAGE.gif SPECFILE
  cc -O2 -ffp-contract=off -o run main.c -lm && ./run N PREFIX X0 Y0 Z0 [REMAP] | sort -n | head
N = 1024*maxiter, PREFIX = hidden bounding-box iterations (100 lorenz3d,
500 rossler3d, 750 lorenz3d1/3/4), (X0,Y0,Z0) the start point. REMAP=1 treats
stored colours 0 and 15 as 16 and 31 (duplicate-palette artefact, needed when
maxiter >= 15). Output lines: "violations first-failing-iterate X# Y# Z#";
the true ordering gives "0 -1".
"""
import os, sys
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(here))
import sim
exec(open(os.path.join(here, 'gen.py')).read())
exec(open(sys.argv[2]).read())   # defines X, Y, Z (C expressions in x, y, z, dt, a, b, c)
g = sim.load(sys.argv[1])
open('img.raw', 'wb').write(bytes(g['pixels']))
M = sim.rot_matrix(*g['rot'])
def sw(name, L):
    s = 'static double %s(int i,double x,double y,double z){switch(i){\n' % name
    for k, e in enumerate(L):
        s += 'case %d: return %s;\n' % (k, e)
    return s + '}return 0;}\n'
src = open(os.path.join(here, 'orbit3d.c.in')).read()
src = src.replace('@FUNCS@', sw('fx', X) + sw('fy', Y) + sw('fz', Z)).replace('@NX@', str(len(X)))
src = src.replace('@NY@', str(len(Y))).replace('@NZ@', str(len(Z)))
src = src.replace('@W@', str(g['w'])).replace('@H@', str(g['h']))
src = src.replace('@P@', '%r,%r,%r,%r' % tuple(g['params'][:4])).replace('@C@', '%r,%r,%r,%r' % g['corners'])
src = src.replace('@M@', ','.join(repr(M[i][j]) for i in range(3) for j in range(3)))
open('main.c', 'w').write(src)
open('names.txt', 'w').write('\n'.join(['X%d %s' % (i, e) for i, e in enumerate(X)] +
                                       ['Y%d %s' % (i, e) for i, e in enumerate(Y)] +
                                       ['Z%d %s' % (i, e) for i, e in enumerate(Z)]))
print(len(X), len(Y), len(Z))
