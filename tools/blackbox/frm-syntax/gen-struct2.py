# SPDX-License-Identifier: GPL-3.0-or-later
# Second round of file-structure tests: inputs/tNN.frm and cases-t.txt.
# Same probe body as gen-struct.py (colour index = floor(k) + 1).
import os
here = os.path.dirname(os.path.abspath(__file__))
def body(k, ind='  '):
    return f'{ind}z = 0, k = {k}:\n{ind}z = z + 1\n{ind}real(z) <= real(k)\n'
def ent(name, k, hdr=None):
    return (hdr if hdr is not None else f'{name} {{') + '\n' + body(k) + '}\n'
def tail():
    return '  z = z + 1\n  real(z) <= real(k)\n}\n'
files, cases = {}, []
def case(cname, fname, entry, extra=''):
    if entry:  # xargs (run-cases.sh) treats quotes and backslashes specially
        entry = entry.replace('\\', '\\\\').replace('"', '\\"')
    fe = (f'formulafile={fname} ' if fname else '') + (f'formulaname={entry} ' if entry else '')
    cases.append(f't-{cname} {fe}maxiter=1000 {extra}'.rstrip())

# t01: statement count limits (one "k = k + 1" per line)
s = ''
for n in [50, 100, 150, 200, 250]:
    s += f'st{n} {{\n  z = 0, k = 0\n' + '  k = k + 1\n' * n + ':\n' + tail()
    case(f'stmts{n}', 't01.frm', f'st{n}')
files['t01.frm'] = s
# t02: one long line "k = 0 + 1 + 1 ..." with n terms
s = ''
for n in [100, 150, 200, 250, 300, 400]:
    s += f'tm{n} {{\n  z = 0, k = 0' + ' + 1' * n + ':\n' + tail()
    case(f'terms{n}', 't02.frm', f'tm{n}')
files['t02.frm'] = s
# t03: long line without many tokens (a long name repeated is not possible; use a long comment)
s = ''
for n in [300, 1000, 3000]:
    s += f'lc{n} {{\n  z = 0, k = {n // 100} ; ' + 'x' * n + '\n:\n' + tail()
    case(f'longcomment{n}', 't03.frm', f'lc{n}')
s += 'Free text ' + 'y' * 3000 + '\n' + ent('lc', 33)
case('longtext', 't03.frm', 'lc')
files['t03.frm'] = s
# t04: more name characters
nm = [('a,b', 40), ('a;b', 41), ('a:b', 42), ('a"b', 43), ('a}b', 44), ('a)b', 45), ('a\\b', 46), ('a|b', 47)]
files['t04.frm'] = ''.join(ent(n, k) for n, k in nm) + ent('   lead', 48)
for i, (n, k) in enumerate(nm): case(f'char{i}', 't04.frm', n)
case('leadspace', 't04.frm', 'lead')
# t05: braces in the body
files['t05.frm'] = ('c5d {\n  z = 0, k = 50:\n  {\n' + tail()
  + 'c5e {\n  z = 0, k = 51:\n  }\n  k = 52\n' + tail()
  + ent('c5f', 53)
  + 'c5g (xa{xis) {\n' + body(54) + '}\n' + ent('c5h', 55))
for n in ['c5d', 'c5e', 'c5f', 'c5g', 'c5h']: case('brace-' + n, 't05.frm', n)
# t06: continuation lines
files['t06.frm'] = (
  'c6a {\n  z = 0, k = 100 + \\ ; comment\n  1:\n' + tail()
  + 'c6b {\n  z = 0, k = 100 \\ + 2:\n' + tail()
  + 'c6c {\n  z = 0, k = 100 + \\   \n  3:\n' + tail()
  + 'c6d {\n  z = 0, k = 100 \\\n + \\\n 4:\n' + tail()
  + 'c6e {\n  z = 0, kk = 100, k = k\\\nk + 5:\n' + tail()
  + 'c6f {\n  z = 0, k = 100 + 6 \\\n:\n' + tail()
  + 'c6g {\n  z = 0, k = 107 \\\n\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  + 'c6h {\n  z = 0, k = 1\\\n08:\n' + tail())
for n in 'abcdefgh': case('cont-c6' + n, 't06.frm', 'c6' + n)
# t07: file lookup
case('nofile-c1', 'zz.frm', 'c1')
case('nofileparam-c1', None, 'c1')
case('noname', 's01.frm', None)
case('upperfile', 'S01.FRM', 'c1')
case('noext', 's01', 'c1')
# t08: DOS end-of-file byte, non-ASCII bytes
files['t08.frm'] = ent('c8a', 80) + '\x1a\n' + ent('c8b', 81)
case('ctrlz-before', 't08.frm', 'c8a'); case('ctrlz-after', 't08.frm', 'c8b')
files['t09.frm'] = ('caf\xe9 text outside \xe9\n' + 'c9a {\n  ; comment \xe9\xff\n' + body(90) + '}\n'
  + ent('c9\xe9', 91))
case('latin1-comment', 't09.frm', 'c9a'); case('latin1-name', 't09.frm', 'c9\xe9')
# t10: tab before brace, comments/blank-only bodies, ':' only
files['t10.frm'] = (ent('c10a', 100, 'c10a\t{') + 'c10b {\n  ; only a comment\n\n}\n'
  + 'c10c {\n  :\n}\n' + 'c10d {\n  ,\n}\n' + 'c10e{z=0,k=104:z=z+1,real(z)<=real(k)}\n')
for n in ['c10a', 'c10b', 'c10c', 'c10d', 'c10e']: case('misc-' + n, 't10.frm', n)
# t11: deep if nesting
s = ''
for d in [5, 20, 50, 100, 200]:
    s += f'nest{d} {{\n  z = 0, k = 100\n' + '  if (1)\n' * d + f'  k = {110 + d % 100}\n' + '  endif\n' * d + ':\n' + tail()
    case(f'nest{d}', 't11.frm', f'nest{d}')
files['t11.frm'] = s
# t12: if blocks in the iteration section and as the last statement
files['t12.frm'] = (
  'it1 {\n  z = 0, k = 120:\n  z = z + 1\n  if (real(z) > real(k))\n    t = 0\n  else\n    t = 1\n  endif\n}\n'
  'it2 {\n  z = 0, k = 121:\n  z = z + 1\n  if (1)\n    real(z) <= real(k)\n  endif\n}\n'
  'it3 {\n  z = 0, k = 122:\n  z = z + 1\n  w = real(z) <= real(k)\n  if (0)\n    w = 1\n  endif\n}\n'
  'it4 {\n  z = 0, k = 123\n  if (1)\n  :\n  endif\n' + tail()
  + 'it5 {\n  z = 0, k = 124:\n  z = z + 1\n  if (real(z) <= real(k))\n    t = 5\n  endif\n}\n'
  'it6 {\n  z = 0, k = 125:\n  if (1)\n    z = z + 1\n  endif\n  real(z) <= real(k)\n}\n'
  'it7 {\n  z = 0, k = 126:\n  z = z + 1\n  if (real(z) > real(k))\n    t = 0\n  endif\n}\n')
for n in ['it1', 'it2', 'it3', 'it4', 'it5', 'it6', 'it7']: case('ifit-' + n, 't12.frm', n)

for fn, txt in files.items():
    with open(os.path.join(here, 'inputs', fn), 'w', newline='', encoding='latin-1') as f:
        f.write(txt)
with open(os.path.join(here, 'cases-t.txt'), 'w', encoding='latin-1') as f:
    f.write('# SPDX-License-Identifier: GPL-3.0-or-later\n# Generated by gen-struct2.py: file-structure cases, round 2.\n')
    f.write('\n'.join(cases) + '\n')
print(len(cases), 'cases')
