# SPDX-License-Identifier: GPL-3.0-or-later
# Expression-variant generator for the arithmetic-order search (own code).
# generates C expression variants for an Euler step v + dt*sum(coef*vars)
import itertools
def trees(items):
    # all binary trees (commutative dedup approx) over list of strings
    if len(items)==1: yield items[0]; return
    seen=set()
    n=len(items)
    for mask in range(1,(1<<n)-1):
        if not (mask & 1): continue  # canonical: first item in left part
        L=[items[i] for i in range(n) if mask>>i&1]; R=[items[i] for i in range(n) if not mask>>i&1]
        for a in trees(L):
            for b in trees(R):
                s='(%s+%s)'%(a,b)
                if s not in seen: seen.add(s); yield s
def prods(factors):
    # factors: list of strings; all multiplication trees
    if len(factors)==1: yield factors[0]; return
    seen=set()
    n=len(factors)
    for mask in range(1,(1<<n)-1):
        if not (mask&1): continue
        L=[factors[i] for i in range(n) if mask>>i&1]; R=[factors[i] for i in range(n) if not mask>>i&1]
        for a in prods(L):
            for b in prods(R):
                s='(%s*%s)'%(a,b)
                if s not in seen: seen.add(s); yield s
def variants(v, monos):
    # monos: list of (sign, [factors without dt]) ; returns C expr strings
    out=[]
    # expanded: v + sum(sign*dt*factors)
    plist=[]
    for sg,f in monos:
        ps=list(prods(['dt']+f))
        plist.append([('-' if sg<0 else '')+p for p in ps])
    for choice in itertools.product(*plist):
        for t in trees([v]+list(choice)): out.append(t)
    # factored: v + dt*T  and v + T*dt (same), where T = sum tree of monomials
    plist2=[]
    for sg,f in monos:
        ps=list(prods(f)) if f else ['1.0']
        plist2.append([('-' if sg<0 else '')+p for p in ps])
    for choice in itertools.product(*plist2):
        for t in trees(list(choice)): out.append('(%s+dt*%s)'%(v,t))
    return out
