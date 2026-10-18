# SPDX-License-Identifier: GPL-3.0-or-later
# Summarises black-box outputs for docs/specs/frm-syntax.md.
#   python3 -I summarise.py DIR [case...]
# For every case NAME in DIR (files NAME.gif or NAME.ff.gif, NAME.stopmsg.txt,
# NAME.log) prints one line:
#   NAME  pixel-MD5(8 hex)  [(colour index, count) x 4 most common]
#   t=<fractint001 type, i16 at 10>  sym=<i16 at 124>
#   003=<entry name> f40..=<non-zero bytes at offsets 40-65 as off:value>
#   STOPMSG: <stopmsg.txt, lines joined with " | ">  LOG: <console output>
# NOIMAGE means no GIF was saved.  The qemu platform warning and the
# closing banner are dropped from LOG.
import sys, os, glob, hashlib, struct, collections
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'resume'))
from gif import read_gif  # our own minimal GIF reader

d = sys.argv[1]
def base(p):
    b = os.path.basename(p)
    for suf in ('.ff.gif', '.gif', '.stopmsg.txt', '.log'):
        if b.endswith(suf):
            return b[:-len(suf)]
    return None
names = sys.argv[2:] or sorted(set(filter(None, (base(p) for p in glob.glob(os.path.join(d, '*'))))))
for n in names:
    extra = ''
    sm = os.path.join(d, n + '.stopmsg.txt')
    if os.path.exists(sm):
        extra += ' STOPMSG: ' + ' | '.join(l.rstrip() for l in open(sm, errors='replace').read().strip().splitlines())
    lg = os.path.join(d, n + '.log')
    if os.path.exists(lg):
        log = ' '.join(l.strip(' \t\r\a') for l in open(lg, errors='replace').read().splitlines()
                       if l.strip(' \t\r\a') and 'WARNING: The requested' not in l and 'Thank You' not in l)
        if log:
            extra += ' LOG: ' + log
    g = next((p for p in (os.path.join(d, n + '.gif'), os.path.join(d, n + '.ff.gif')) if os.path.exists(p)), None)
    if g is None:
        print(f'{n:26s} NOIMAGE{extra}')
        continue
    G = read_gif(g)
    px = bytes(G['pixels'])
    h = hashlib.md5(px).hexdigest()[:8]
    c = collections.Counter(px).most_common(4)
    e1 = G['exts']['fractint001']
    typ = struct.unpack('<h', e1[10:12])[0]
    sym = struct.unpack('<h', e1[124:126])[0]
    s3 = ''
    e3 = G['exts'].get('fractint003')
    if e3:
        nm = e3[:40].split(b'\0')[0].decode('latin1')
        fl = ','.join(f'{i}:{e3[i]}' for i in range(40, len(e3)) if e3[i])
        s3 = f' 003={nm!r} f40..={fl or "-"}'
    print(f'{n:26s} {h} {c} t={typ} sym={sym}{s3}{extra}')
