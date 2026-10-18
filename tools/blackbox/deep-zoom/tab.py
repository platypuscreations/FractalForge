# SPDX-License-Identifier: GPL-3.0-or-later
# One line per GIF: deep flag [412], N [414], fractint005 length, status, f64 corners.
#   python3 -I tab.py FILE.gif...
import struct, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif
for f in sys.argv[1:]:
    try:
        g = gif.read_gif(f)
    except Exception as ex:
        print(os.path.basename(f), 'ERROR', ex); continue
    e = g['exts']['fractint001']
    h = lambda o: struct.unpack('<h', e[o:o + 2])[0]
    d = lambda o: struct.unpack('<d', e[o:o + 8])[0]
    e5 = g['exts'].get('fractint005')
    print('%-18s %dx%d type=%d 412=%d 414=%2d 005=%-5s st=%d blocks=%s xmin=%r xmax=%r ymin=%r ymax=%r tol=%r/%r' % (
        os.path.basename(f)[:-4], g['w'], g['h'], h(10), h(412), h(414), len(e5) if e5 else '-', h(202),
        ','.join(x[-3:] for x in g['order'] if x != 'image'), d(12), d(20), d(28), d(36), d(474), d(482)))
