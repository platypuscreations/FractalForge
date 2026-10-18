# SPDX-License-Identifier: GPL-3.0-or-later
# Summarises black-box outputs for docs/specs/lsystem.md.
#   python3 -I summarise.py DIR [case...]
# For every case NAME in DIR (NAME.gif or NAME.ff.gif, NAME.stopmsg.txt,
# NAME.log) prints one line:
#   NAME WxH pixel-MD5(8 hex) bbox=(xmin,ymin,xmax,ymax of non-zero pixels)
#   n=<non-zero pixel count> col=[(index, count)...]  t=<type> p1=<param 1>
#   corners=<xmin,xmax,ymin,ymax> st=<status at 202> 003=<entry name>
#   blocks=<extension ids>  STOPMSG: ...  LOG: ...
# NOIMAGE means no GIF was saved.
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
        print(f'{n:22s} NOIMAGE{extra}')
        continue
    G = read_gif(g)
    W, H = G['w'], G['h']
    px = bytes(G['pixels'])
    h = hashlib.md5(px).hexdigest()[:8]
    nz = [i for i, v in enumerate(px) if v]
    if nz:
        xs = [i % W for i in nz]; ys = [i // W for i in nz]
        bb = f'({min(xs)},{min(ys)},{max(xs)},{max(ys)})'
    else:
        bb = '-'
    c = collections.Counter(px[i] for i in nz).most_common(6)
    e1 = G['exts']['fractint001']
    typ = struct.unpack('<h', e1[10:12])[0]
    p1 = struct.unpack('<d', e1[44:52])[0]
    co = struct.unpack('<4d', e1[12:44])
    st = struct.unpack('<h', e1[202:204])[0]
    s3 = ''
    e3 = G['exts'].get('fractint003')
    if e3:
        s3 = ' 003=' + repr(e3[:48].split(b'\0')[0].decode('latin1'))
    blocks = ','.join(k[-3:] for k in G['exts'])
    print(f'{n:22s} {W}x{H} {h} bbox={bb} n={len(nz)} col={c} t={typ} p1={p1:g} '
          f'corners={",".join(f"{v:g}" for v in co)} st={st}{s3} blocks={blocks}{extra}')
