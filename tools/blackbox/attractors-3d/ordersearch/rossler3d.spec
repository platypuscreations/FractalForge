# SPDX-License-Identifier: GPL-3.0-or-later
# Candidate C expressions (X, Y, Z lists) for mk_orbit3d.py; evaluated as Python.
X=variants('x',[(-1,['y']),(-1,['z'])])
Y=variants('y',[(1,['x']),(1,['a','y'])])
Z=variants('z',[(1,['b']),(1,['x','z']),(-1,['c','z'])])
Z+=['(z+dt*(b+z*(x-c)))','(z+(dt*b+(dt*z)*(x-c)))','((z+dt*b)+(dt*z)*(x-c))','(z+(dt*(b+z*(x-c))))','((z+dt*b)+dt*(z*(x-c)))','(z+(dt*b+dt*(z*(x-c))))']
X+=['(x-dt*(y+z))','(x-(dt*(y+z)))']
Y+=['(y+dt*(x+a*y))']
