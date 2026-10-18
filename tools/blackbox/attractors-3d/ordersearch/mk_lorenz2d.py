# SPDX-License-Identifier: GPL-3.0-or-later
"""Arithmetic-order search for the lorenz (type 63) Euler step (own code).

Usage (in an empty work directory):
  python3 -I .../ordersearch/mk_lorenz2d.py IMAGE.gif
  cc -O2 -ffp-contract=off -o run main.c && ./run N | sort -n | tail
N = 1024*maxiter. Each output line is "first failing step X# Y# Z#"; names.txt
lists the expressions. A variant survives (first failing step == N) only if
every drawn segment is consistent with the image (colours only increase).
-ffp-contract=off matters: fused multiply-adds change the rounding.
"""
import os, sys, struct
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.dirname(here))
from gif import read_gif
exec(open(os.path.join(here, 'gen.py')).read())
X = variants('x', [(1, ['s', 'y']), (-1, ['s', 'x'])])
X += ['(x+dt*(s*(y-x)))', '(x+(dt*s)*(y-x))', '(x+s*(dt*(y-x)))', '(x+s*((dt*y)-(dt*x)))',
      '((1.0-dt*s)*x+(dt*s)*y)', '((x-(dt*s)*x)+(dt*s)*y)', '(x+(y-x)*(dt*s))']
Y = variants('y', [(1, ['r', 'x']), (-1, ['y']), (-1, ['x', 'z'])])
Y += ['(y+dt*(x*(r-z)-y))', '((y+(dt*x)*(r-z))-dt*y)', '((y-dt*y)+(dt*x)*(r-z))',
      '((1.0-dt)*y+(dt*x)*(r-z))', '((1.0-dt)*y+dt*(x*(r-z)))', '(y+(dt*(x*(r-z))-dt*y))',
      '((y+dt*(x*(r-z)))-dt*y)', '(y+((r-z)*x-y)*dt)', '((y-dt*y)+dt*(x*(r-z)))',
      '((y-dt*y)+x*(dt*(r-z)))', '((y+x*(dt*(r-z)))-dt*y)', '((y+x*((r-z)*dt))-y*dt)']
Z = variants('z', [(1, ['x', 'y']), (-1, ['b', 'z'])])
Z += ['(z+dt*(x*y-b*z))', '((1.0-dt*b)*z+(dt*x)*y)', '((1.0-dt*b)*z+dt*(x*y))',
      '((1.0-dt*b)*z+x*(dt*y))', '((1.0-b*dt)*z+(x*y)*dt)']
g = read_gif(sys.argv[1]); b = g['exts']['fractint001']
f = lambda o: struct.unpack_from('<d', b, o)[0]
open('img.raw', 'wb').write(bytes(g['pixels']))
def sw(name, L):
    s = 'static double %s(int i,double x,double y,double z){switch(i){\n' % name
    for k, e in enumerate(L):
        s += 'case %d: return %s;\n' % (k, e)
    return s + '}return 0;}\n'
src = open(os.path.join(here, 'lorenz2d.c.in')).read()
src = src.replace('@FUNCS@', sw('fx', X) + sw('fy', Y) + sw('fz', Z)).replace('@NX@', str(len(X)))
src = src.replace('@NY@', str(len(Y))).replace('@NZ@', str(len(Z)))
src = src.replace('@W@', str(g['w'])).replace('@H@', str(g['h']))
src = src.replace('@P@', '%r,%r,%r,%r' % (f(44), f(52), f(270), f(278)))
src = src.replace('@C@', '%r,%r,%r,%r' % (f(12), f(20), f(28), f(36)))
open('main.c', 'w').write(src)
open('names.txt', 'w').write('\n'.join(['X%d %s' % (i, e) for i, e in enumerate(X)] +
                                       ['Y%d %s' % (i, e) for i, e in enumerate(Y)] +
                                       ['Z%d %s' % (i, e) for i, e in enumerate(Z)]))
