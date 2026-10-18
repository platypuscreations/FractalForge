# SPDX-License-Identifier: GPL-3.0-or-later
# Checks that every fractint005 slot in the given files is in the normal form
# produced by dz.slot_encode (re-encoding the decoded value gives the same bytes).
#   python3 -I normchk.py FILE.gif...
import os, struct, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif, dz
ok = bad = 0
for f in sys.argv[1:]:
    g = gif.read_gif(f); e5 = g['exts'].get('fractint005')
    if not e5: continue
    N = struct.unpack('<h', g['exts']['fractint001'][414:416])[0]; L = N + 6
    for i in range(len(e5) // L):
        b = e5[i * L:(i + 1) * L]
        v = dz.slot_decode(b)[0]
        if dz.slot_encode(v, N + 4) == bytes(b): ok += 1
        else:
            bad += 1; print('not normal:', f, i, b.hex(' '))
print('slots in normal form:', ok, 'other:', bad)
