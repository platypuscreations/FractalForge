# SPDX-License-Identifier: GPL-3.0-or-later
# Round 3: z in the init section, backslash, jump limit, PI_SYM views,
# storage flags in fractint003, long entries, unary minus, .par embedding.
# Writes inputs/r3*.frm, inputs/p1.par and cases-r3.txt.
import os
here = os.path.dirname(os.path.abspath(__file__))
def tail():
    return '  z = z + 1\n  real(z) <= real(k)\n}\n'
def ntail():  # counter n instead of z
    return '  n = n + 1\n  real(n) <= real(k)\n}\n'
files, cases = {}, []
def case(cname, fname, entry, extra=''):
    fe = (f'formulafile={fname} ' if fname else '') + (f'formulaname={entry} ' if entry else '')
    cases.append(f'r3-{cname} {fe}maxiter=1000 {extra}'.rstrip())
# A: z in the init section (counter is n)
zs = [('za', 'z = 3, k = 100 + 10 * real(z)'), ('zb', 'z = 0, z = 3, k = 100 + 10 * real(z)'),
      ('zc', 'w = 0, w = 3, k = 100 + 10 * real(w)'), ('zd', 'z = 5, k = 100 + 10 * real(z)'),
      ('ze', 'z = 3\n  k = 100 + 10 * real(z)'), ('zf', 'z = (1,2), k = 100 + 10 * real(z) + 30 * imag(z)'),
      ('zg', 'z = pixel, k = 100 + 10 * real(z)'), ('zh', 'z = 3, w = z, k = 100 + 10 * real(w)'),
      ('zi', 'z = 2 + 1, k = 100 + 10 * real(z)'), ('zj', 'a = 3, z = a, k = 100 + 10 * real(z)'),
      ('zk', 'k = 100 + 10 * real(z)'), ('zl', 'z = 3, k = 100 + 10 * real(z) + 0 * z')]
s = ''
for n, init in zs:
    s += f'{n} {{\n  n = 0, {init}:\n' + ntail()
    case('zinit-' + n, 'r3a.frm', n)
s += 'zm {\n  n = 0, z = 3, k = 100:\n  n = n + 1\n  real(n) <= real(k) + 10 * real(z)\n}\n'
case('zinit-zm', 'r3a.frm', 'zm')
s += 'zn {\n  n = 0, k = 100:\n  n = n + 1\n  z = 3\n  real(n) <= real(k) + 10 * real(z)\n}\n'
case('zinit-zn', 'r3a.frm', 'zn')
files['r3a.frm'] = s
# B: backslash
files['r3b.frm'] = ('bs1 {\n  z = 0, k = 100 \\ + 2\n  + 3:\n' + tail()
  + 'bs2 {\n  z = 0, k = 100 \\ ; + 2\n  + 3:\n' + tail()
  + 'bs3 {\n  z = 0, k = 10\\0 + 4:\n' + tail())
for n in ['bs1', 'bs2', 'bs3']: case('bslash-' + n, 'r3b.frm', n)
# C: jump-command limit with sequential if/endif pairs
s = ''
for n in [49, 50, 51, 99, 100, 101]:
    s += f'jp{n} {{\n  z = 0, k = 100\n' + '  if (1)\n  k = k + 1\n  endif\n' * n + ':\n' + tail()
    case(f'jumps-if{n}', 'r3c.frm', f'jp{n}')
for n in [33, 34, 66, 67]:
    s += f'je{n} {{\n  z = 0, k = 100\n' + '  if (0)\n  k = k + 7\n  else\n  k = k + 1\n  endif\n' * n + ':\n' + tail()
    case(f'jumps-ife{n}', 'r3c.frm', f'je{n}')
files['r3c.frm'] = s
# E: storage flags (bytes after the name in fractint003)
st = [('sn', 't = 1'), ('s1', 't = p1'), ('s2', 't = p2'), ('s3', 't = p3'), ('s4', 't = p4'), ('s5', 't = p5'),
      ('sf1', 't = fn1(1)'), ('sf2', 't = fn2(1)'), ('sf3', 't = fn3(1)'), ('sf4', 't = fn4(1)'),
      ('s13', 't = p1 + p3'), ('spx', 't = pixel'), ('srs', 't = rand'), ('sws', 't = whitesq'),
      ('ssn', 't = scrnpix'), ('sab', 'p1 = 3'), ('sp6', 't = p6'), ('ssin', 't = sin(1)'), ('ssm', 't = scrnmax'),
      ('smi', 't = maxit'), ('sis', 't = ismand'), ('sce', 't = center'), ('smx', 't = magxmag'),
      ('srk', 't = rotskew'), ('sls', 't = lastsqr'), ('spi', 't = pi'), ('se', 't = e')]
s = ''
for n, init in st:
    s += f'{n} {{\n  z = 0, k = 5, {init}:\n' + tail()
    case('store-' + n, 'r3e.frm', n, 'maxiter=20' if n == 'srs' else '')
s += 'n18abcdefghijklmno {\n  z = 0, k = 5, t = p2:\n' + tail()
case('store-len18', 'r3e.frm', 'n18abcdefghijklmno')
s += 'sit2 {\n  z = 0, k = 5:\n  z = z + 1 + 0 * p2\n  real(z) <= real(k)\n}\n'
case('store-p2iter', 'r3e.frm', 'sit2')
files['r3e.frm'] = s
case('store-fnset', 'r3e.frm', 'sf1', 'function=cos/tan/exp/log')
case('store-params', 'r3e.frm', 's2', 'params=1/2/3/4/5/6/7/8/9/10')
case('store-paramsnone', 'r3e.frm', 'sn', 'params=1/2/3/4/5/6/7/8/9/10')
# D: PI_SYM in other views (same asymmetric body as gen-sym.py, plus a periodic one)
body = '  z = pixel:\n  z = z*z + (0.3,0.4)*pixel + (0.1,0.1)\n  |z| <= 4\n'
pbody = '  z = sin(pixel) + (0.2,0.1):\n  z = z*z + (0.3,0.4)\n  |z| <= 4\n'
files['r3d.frm'] = ('ps(PI_SYM) {\n' + body + '}\n' + 'pn {\n' + body + '}\n'
  + 'pxy(XYAXIS) {\n' + body + '}\n' + 'qs(PI_SYM) {\n' + pbody + '}\n' + 'qn {\n' + pbody + '}\n')
for v, c in [('wide', 'corners=-10/10/-7.5/7.5'), ('narrow', 'corners=-1/1/-0.75/0.75'),
             ('off', 'corners=-1/7/-3/3')]:
    for n in ['ps', 'pn', 'pxy', 'qs', 'qn']:
        cases.append(f'r3-pisym-{n}-{v} formulafile=r3d.frm formulaname={n} {c}')
# G: long entries made of comments / blank lines
files['r3g.frm'] = ('lg1 {\n' + '  ; comment line\n' * 2000 + '  z = 0, k = 71:\n' + tail()
  + 'lg2 {\n' + '\n' * 3000 + '  z = 0, k = 72:\n' + tail()
  + 'lg3 {\n  z = 0, k = 73' + ' ' * 5000 + ':\n' + tail())
for n in ['lg1', 'lg2', 'lg3']: case('longent-' + n, 'r3g.frm', n)
# H/I: unary minus before '(' with variables, and 0^2
um = [('um1', 'a = 3, j = -(2 + a)'), ('um2', 'a = 3, j = -(a + 2)'), ('um3', 'j = -(2)'),
      ('um4', 'j = 0 ^ 2'), ('um5', 'j = (0) ^ 2'), ('um6', 'a = 0, j = a ^ 2'), ('um7', 'j = -(.5 + 1)'),
      ('um8', 'j = -(1e1 + 1)'), ('um9', 'j = -((1, 2)) + (0, 0)'), ('um10', 'a = 2, j = -(a) * 3')]
s = ''
for n, init in um:
    s += f'{n} {{\n  z = 0, {init}, k = 100 + 10 * real(j):\n' + tail()
    case('uminus-' + n, 'r3h.frm', n)
s += 'um11 {\n  z = 0, k = 100:\n  z = z + 1\n  j = -(2)\n  real(z) <= real(k) + 10 * real(j)\n}\n'
case('uminus-um11', 'r3h.frm', 'um11')
files['r3h.frm'] = s
# J: formulas inside .par files
files['p1.par'] = ('frm:pq {\n  z = 0, k = 30:\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'img1 { reset type=formula formulaname=pq maxiter=1000 viewwindows=2 passes=1 periodicity=0 map=uniq.map }\n'
  'img2 { reset type=formula formulafile=p1.par formulaname=pq maxiter=1000 viewwindows=2 passes=1 periodicity=0 map=uniq.map }\n'
  'img3 { reset type=formula formulaname=frm:pq maxiter=1000 viewwindows=2 passes=1 periodicity=0 map=uniq.map }\n'
  'pr {\n  z = 0, k = 31:\n  z = z + 1\n  real(z) <= real(k)\n}\n')
for n in ['img1', 'img2', 'img3']:
    cases.append(f'r3-par-{n} @p1.par/{n}')
case('par-frmfile', 'p1.par', 'pq'); case('par-frmfile2', 'p1.par', 'frm:pq'); case('par-frmfile3', 'p1.par', 'pr')
cases.append('r3-par-cmdfile formulafile=p1.par @p1.par/img1')

for fn, txt in files.items():
    with open(os.path.join(here, 'inputs', fn), 'w', newline='') as f:
        f.write(txt)
with open(os.path.join(here, 'cases-r3.txt'), 'w') as f:
    f.write('# SPDX-License-Identifier: GPL-3.0-or-later\n# Generated by gen-r3.py: round 3.\n')
    f.write('\n'.join(cases) + '\n')
print(len(cases), 'cases')
