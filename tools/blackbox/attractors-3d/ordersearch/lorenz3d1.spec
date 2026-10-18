# SPDX-License-Identifier: GPL-3.0-or-later
# Candidate C expressions (X, Y, Z lists) for mk_orbit3d.py; evaluated as Python.
import itertools
R='sqrt(x*x+y*y)'
def combos(v, terms_forms, orders):
    out=[]
    for forms in itertools.product(*terms_forms):
        for order in orders:
            t=[forms[i] for i in order]
            out.append('(%s+((%s+%s)+%s))'%(v,t[0],t[1],t[2]))
            out.append('(((%s+%s)+%s)+%s)'%(v,t[0],t[1],t[2]))
            out.append('(%s+(%s+(%s+%s)))'%(v,t[0],t[1],t[2]))
    return out
A1=['(-(dt*a+dt)*x)','(-((a+1.0)*dt)*x)','(-(a+1.0)*(dt*x))','(-(dt*a)*x-dt*x)','(-dt*((a+1.0)*x))']
B1=['(((dt*a-dt*b)+dt*z)*y)','((((a-b)+z)*dt)*y)','(((a-b)+z)*(dt*y))','(((a-b)*dt+dt*z)*y)','(dt*(((a-b)+z)*y))','((dt*a+(dt*z-dt*b))*y)']
C1=['((dt-dt*a)*R)','(((1.0-a)*dt)*R)','((1.0-a)*(dt*R))','(dt*((1.0-a)*R))']
X=[e.replace('R',R) for e in combos('x',[A1,B1,C1],[(0,1,2)])]
A2=['(((dt*b-dt*a)-dt*z)*x)','((((b-a)-z)*dt)*x)','(((b-a)-z)*(dt*x))','(((b-a)*dt-dt*z)*x)','(dt*(((b-a)-z)*x))','((dt*b-(dt*a+dt*z))*x)']
B2=['(-(dt*a+dt)*y)','(-((a+1.0)*dt)*y)','(-(a+1.0)*(dt*y))','(-dt*((a+1.0)*y))']
C2=['(((dt*b+dt*a)-dt*z)*R)','((((b+a)-z)*dt)*R)','(((b+a)-z)*(dt*R))','(dt*(((b+a)-z)*R))','(((b+a)*dt-dt*z)*R)']
Y=[e.replace('R',R) for e in combos('y',[A2,B2,C2],[(0,1,2)])]
Z=['(z+((dt*y)/2.0-(dt*c)*z))','(z+(dt*y/2.0-dt*c*z))','(z+dt*(y/2.0-c*z))','((z+(dt*y)/2.0)-(dt*c)*z)','(z+((dt*y)*0.5-(dt*c)*z))','(z+(dt*(y/2.0)-(dt*c)*z))','(z+((y/2.0)*dt-(c*dt)*z))','(z+(dt*(0.5*y)-(dt*c)*z))','(z+(((dt*y)/2.0)-(dt*z)*c))']
