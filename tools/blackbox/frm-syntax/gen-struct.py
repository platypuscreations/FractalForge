# SPDX-License-Identifier: GPL-3.0-or-later
# Writes the file-structure test files inputs/sNN.frm and cases-s.txt.
# Most entries use the probe body of gen-probe.py, so the saved colour index
# is floor(k) + 1 and identifies which entry / statement was used.
# Structure files carry no licence comment where a comment would change the
# test; they are tiny test inputs we wrote ourselves.
import os
here = os.path.dirname(os.path.abspath(__file__))
def body(k, ind='  '):
    return f'{ind}z = 0, k = {k}:\n{ind}z = z + 1\n{ind}real(z) <= real(k)\n'
def ent(name, k, hdr=None):
    return (hdr if hdr is not None else f'{name} {{') + '\n' + body(k) + '}\n'
files, cases = {}, []
def case(cname, fname, entry, extra=''):
    cases.append(f's-{cname} formulafile={fname} formulaname={entry} maxiter=1000 {extra}'.rstrip())

# s01 comments everywhere, blank lines, tabs
files['s01.frm'] = ('; comment at the top of the file\n\n'
  'c1 { ; comment after the brace\n'
  '  ; comment line inside\n\n'
  '\tz = 0, k = 10:   ; trailing comment after the colon\n'
  '  z = z + 1 ; comment\n'
  '\n'
  '  real(z) <= real(k) ; comment on the test\n'
  '} ; comment after the closing brace\n')
case('comments', 's01.frm', 'c1')
# s02 free text outside entries
files['s02.frm'] = ('This is free text before any entry.\nMore text, with commas: and colons.\n\n'
  + ent('c2', 20) + 'Text between entries.\n' + ent('c2b', 21) + 'Trailing text.\n')
case('text-before', 's02.frm', 'c2'); case('text-between', 's02.frm', 'c2b')
# s03 brace on the next line; no space before brace; brace after blank line
files['s03.frm'] = ('c3\n{\n' + body(30) + '}\n' + 'c3b{\n' + body(31) + '}\n'
  + 'c3c\n\n{\n' + body(32) + '}\n' + 'c3d   \t {\n' + body(33) + '}\n')
for n in ['c3', 'c3b', 'c3c', 'c3d']: case('brace-' + n, 's03.frm', n)
# s04 header qualifiers
q = [('c4a', '(XAXIS)'), ('c4b', ' (xaxis)'), ('c4c', '(BOGUS)'), ('c4d', '(ORIGIN)'),
     ('c4e', '(NOSYM)'), ('c4f', ' junk words'), ('c4g', '[x=1]'), ('c4h', '(XAXIS)[x]'),
     ('c4i', '(YAXIS)'), ('c4j', '(XYAXIS)'), ('c4k', '(PI_SYM)'), ('c4l', '(XAXIS_NOPARM)'),
     ('c4m', '()'), ('c4n', '(xaxis'), ('c4o', '(NONE)'), ('c4p', '(XAXIS_NOREAL)'),
     ('c4q', '(XAXIS_NOIMAG)'), ('c4r', '(ORIGIN_NOPARM)'), ('c4s', '(PI_SYM_NOPARM)'),
     ('c4t', '(YAXIS_NOPARM)'), ('c4u', '(XYAXIS_NOPARM)'), ('c4v', '(x axis)')]
s = ''
for i, (n, h) in enumerate(q):
    s += ent(n, 40 + i, f'{n}{h} {{')
files['s04.frm'] = s
for n, h in q: case('hdr-' + n, 's04.frm', n)
# s05 one-line entry, closing brace after last statement, text after '{'
files['s05.frm'] = ('c5 { z = 0, k = 50: z = z + 1, real(z) <= real(k) }\n'
  'c5b {\n  z = 0, k = 51:\n  z = z + 1\n  real(z) <= real(k) }\n'
  'c5c { z = 0, k = 52:\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'c5d {\n  z = 0, k = 53:\n  z = z + 1\n  real(z) <= real(k)\n} c5e {\n' + body(54) + '}\n')
for n in ['c5', 'c5b', 'c5c', 'c5d', 'c5e']: case('line-' + n, 's05.frm', n)
# s06 CRLF line ends
files['s06.frm'] = (ent('c6', 60) + ent('c6b', 61)).replace('\n', '\r\n')
case('crlf', 's06.frm', 'c6'); case('crlf2', 's06.frm', 'c6b')
# s07 duplicate names and case
files['s07.frm'] = ent('dup', 70) + ent('dup', 71) + ent('Mixed', 72) + ent('UPPER', 73)
case('dup', 's07.frm', 'dup'); case('case-dupU', 's07.frm', 'DUP')
case('case-mixed', 's07.frm', 'mixed'); case('case-upper', 's07.frm', 'upper')
case('case-upper2', 's07.frm', 'UPPER')
# s08 long names
lens = [15, 16, 17, 18, 20, 32, 40, 47, 48, 49, 60]
s = ''
for i, L in enumerate(lens):
    nm = (f'n{L}' + 'abcdefghijklmnopqrstuvwxyz' * 3)[:L]
    s += ent(nm, 80 + i)
    case(f'len{L}', 's08.frm', nm)
    case(f'len{L}-cut15', 's08.frm', nm[:15]) if L > 15 else None
files['s08.frm'] = s
# s09 name characters
nm = [('a-b', 90), ('a_b', 91), ('a.b', 92), ('1ab', 93), ('a#b', 94), ('a!b', 95), ('a+b', 96), ('a@b', 97), ('a/b', 98), ('a$b', 99)]
files['s09.frm'] = ''.join(ent(n, k) for n, k in nm)
for i, (n, k) in enumerate(nm): case(f'char{i}', 's09.frm', n)
# s10 statements split across lines
files['s10.frm'] = ('c10 {\n  z = 0, k = 100 + \\\n  1:\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'c10b {\n  z = 0, k = 100 +\n  2:\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'c10c {\n  z = 0, k = 100 \\\n + 3:\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'c10d {\n  z = 0, k = (100\n + 4):\n  z = z + 1\n  real(z) <= real(k)\n}\n')
for n in ['c10', 'c10b', 'c10c', 'c10d']: case('split-' + n, 's10.frm', n)
# s12 missing closing brace at end of file
files['s12.frm'] = ent('c12a', 120) + 'c12 {\n' + body(121)
case('noclose-eof', 's12.frm', 'c12'); case('noclose-eof-first', 's12.frm', 'c12a')
# s13 missing closing brace before the next entry
files['s13.frm'] = 'c13a {\n' + body(130) + '\n' + ent('c13b', 131) + ent('c13c', 132)
case('noclose-a', 's13.frm', 'c13a'); case('noclose-b', 's13.frm', 'c13b'); case('noclose-c', 's13.frm', 'c13c')
# s15 braces inside comments and the body
files['s15.frm'] = ('c15 {\n  ; } closing brace in a comment\n' + body(150) + '}\n'
  'c15b {\n  z = 0, k = 151: ; {\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'c15c {\n  z = 0, k = 152:\n  { z = z + 1 }\n  real(z) <= real(k)\n}\n')
for n in ['c15', 'c15b', 'c15c']: case('brace-in-' + n, 's15.frm', n)
# s17 empty statements
files['s17.frm'] = ('c17 {\n  z = 0,, k = 170,:\n  ,z = z + 1,,\n  real(z) <= real(k),\n}\n'
  'c17b {\n  ,z = 0, k = 171:\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'c17c {\n  z = 0, k = 172:\n  z = z + 1\n  real(z) <= real(k)\n  ,\n}\n')
for n in ['c17', 'c17b', 'c17c']: case('empty-' + n, 's17.frm', n)
# s18-s23 init/iteration separator and the bailout test
files['s18.frm'] = (
  'two {\n  z = 0, k = 180:\n  k = 181:\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'none {\n  z = z + 1\n  real(z) <= 9\n}\n'
  'none2 {\n  k = 7, z = z + 1\n  real(z) <= real(k)\n}\n'
  'mid {\n  z = 0, k = 183:\n  z = z + 1\n  real(z) <= real(k)\n  z = z\n}\n'
  'mid2 {\n  z = 0, k = 184:\n  real(z) <= 3\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'asg {\n  z = 0, k = 185:\n  z = z + 1\n}\n'
  'asgk {\n  z = 0, k = 186:\n  z = z + 1, k = real(z) <= 9\n}\n'
  'asg0 {\n  z = 0:\n  z = z + 1, w = 0\n}\n'
  'emptyit {\n  z = 0, k = 187:\n}\n'
  'emptyin {\n  :\n  z = z + 1\n  real(z) <= 8\n}\n'
  'sameln {\n  z = 0, k = 188: z = z + 1\n  real(z) <= real(k)\n}\n'
  'space {\n  z = 0 k = 189:\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'testinit {\n  z = 0, k = 190, real(z) < 0:\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'colon2 {\n  z = 0, k = 191:\n  z = z + 1:\n  real(z) <= real(k)\n}\n'
  'empty {\n}\n'
  'onlytest {\n  real(z) <= 5\n}\n')
for n in ['two', 'none', 'none2', 'mid', 'mid2', 'asg', 'asgk', 'asg0', 'emptyit', 'emptyin', 'sameln', 'space', 'testinit', 'colon2', 'empty', 'onlytest']:
    case('sep-' + n, 's18.frm', n)
# s26 long line, long entry
files['s26.frm'] = (
  'l200 {\n  z = 0, k = 20' + ' + 0' * 45 + ':\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'l600 {\n  z = 0, k = 21' + ' + 0' * 145 + ':\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'l2000 {\n  z = 0, k = 22' + ' + 0' * 495 + ':\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'm300 {\n  z = 0, k = 0\n' + '  k = k + 0.1\n' * 300 + ':\n  z = z + 1\n  real(z) <= real(k)\n}\n'
  'm2000 {\n  z = 0, k = 0\n' + '  k = k + 0.025\n' * 2000 + ':\n  z = z + 1\n  real(z) <= real(k)\n}\n')
for n in ['l200', 'l600', 'l2000', 'm300', 'm2000']: case('long-' + n, 's26.frm', n)
# s28 odd headers
files['s28.frm'] = ('my name {\n' + body(200) + '}\n' + '{\n' + body(201) + '}\n' + ent('c28', 202)
  + 'c28b {}\n' + 'c28c\n' + ent('c28d', 203))
for n in ['my', 'c28', 'c28b', 'c28c', 'c28d']: case('hdr2-' + n, 's28.frm', n)
case('hdr2-myname', 's28.frm', '"my name"')
# s34 other comment characters
cc = [('h', '# hash'), ('sl', '// slashes'), ('ap', "' apostrophe"), ('rem', 'rem remark'), ('bang', '! bang')]
files['s34.frm'] = ''.join(f'cc{n} {{\n  {c}\n' + body(210 + i) + '}\n' for i, (n, c) in enumerate(cc))
for n, c in cc: case('cmt-' + n, 's34.frm', 'cc' + n)
# s35 error line numbers: header on file line 4, error on entry line 5
files['s35.frm'] = ('; one\n\n; three\nerr {\n  ; comment\n\n  z = 0, k = 1:\n  z = z +* 1\n  real(z) <= real(k)\n}\n'
  'err2 { z = 0, k = 1: z = z +* 1, real(z) <= real(k) }\n'
  'err3 {\n  z = 0, k = 1 +* 2:\n  z = z +* 1\n  real(z) <= real(k) +*\n}\n')
for n in ['err', 'err2', 'err3']: case('lineno-' + n, 's35.frm', n)

for fn, txt in files.items():
    with open(os.path.join(here, 'inputs', fn), 'w', newline='') as f:
        f.write(txt)
with open(os.path.join(here, 'cases-s.txt'), 'w') as f:
    f.write('# SPDX-License-Identifier: GPL-3.0-or-later\n# Generated by gen-struct.py: file-structure cases.\n')
    f.write('\n'.join(cases) + '\n')
print(len(cases), 'cases')
