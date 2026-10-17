# SPDX-License-Identifier: GPL-3.0-or-later
# final.py OUT REF : is OUT complete and pixel-identical to REF?
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif, struct
a = gif.read_gif(sys.argv[1]); b = gif.read_gif(sys.argv[2])
e1 = a['exts']['fractint001']
st = struct.unpack('<h', e1[202:204])[0]
d = [i for i in range(len(b['pixels'])) if a['pixels'][i] != b['pixels'][i]]
e2 = a['exts'].get('fractint002')
print(os.path.basename(sys.argv[1]), 'status', st, 'pass', chr(e1[200]), '002', gif.i32s(e2)[:10] if e2 else None,
      'diff', len(d), [(i // a['w'], i % a['w']) for i in d[:4]])
