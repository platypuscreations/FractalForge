# SPDX-License-Identifier: GPL-3.0-or-later
"""Prints status, calc time and the decoded orbit resume block (fractint002) of saves."""
import struct, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gif import read_gif
for path in sys.argv[1:]:
    g = read_gif(path)
    a = g['exts']['fractint001']
    print('%s: status=%d extlen=%d time=%d blocks=%s' % (
        os.path.basename(path), struct.unpack_from('<h', a, 202)[0],
        struct.unpack_from('<H', a, 204)[0], struct.unpack_from('<I', a, 214)[0],
        [k for k in g['exts']]))
    b = g['exts'].get('fractint002')
    if b:
        i32 = lambda o: struct.unpack_from('<i', b, o)[0]
        f64 = lambda o: struct.unpack_from('<d', b, o)[0]
        print('   len=%d [0]=%d [4]=%d [8]=%d [12]=%d [16]=%d f64[20,28,36]=%r %r %r [44..55]=%s [56]=%d [60..]=%s' % (
            len(b), i32(0), i32(4), i32(8), i32(12), i32(16), f64(20), f64(28), f64(36),
            b[44:56].hex(), i32(56), b[60:].hex()))
