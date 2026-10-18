# SPDX-License-Identifier: GPL-3.0-or-later
# Summarises the interrupt/reload outputs of run-interrupts.sh.
#   python3 -I intsum.py DIR FULL.gif
# FULL.gif is the uninterrupted batch render of the same parameters
# (sem/s-kobig-12.gif).  Per saved GIF prints: status (fractint001 offset
# 202), calculation time (214), blocks, drawn pixels that the full render
# lacks ("extra"), pixels of the full render that are missing, marker
# pixels (colour 7) left, and the keys used.
import sys, os, glob, struct
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'resume'))
sys.dont_write_bytecode = True
from gif import read_gif
d, fullp = sys.argv[1], sys.argv[2]
full = read_gif(fullp)['pixels']
for g in sorted(glob.glob(os.path.join(d, '*.gif'))):
    n = os.path.basename(g)[:-4]
    G = read_gif(g); p = G['pixels']; e = G['exts']['fractint001']
    extra = sum(1 for a, b in zip(p, full) if a and not b)
    missing = sum(1 for a, b in zip(p, full) if b and not a)
    marks = sum(1 for a in p if a == 7)
    k = os.path.join(d, n + '.keys.txt')
    keys = open(k).read().strip() if os.path.exists(k) else ''
    print(f'{n:12s} status={struct.unpack("<h", e[202:204])[0]:2d} time={struct.unpack("<I", e[214:218])[0]:5d} '
          f'blocks={",".join(x[-3:] for x in G["exts"])} extra={extra} missing={missing} markers={marks}  [{keys}]')
