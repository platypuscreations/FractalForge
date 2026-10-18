# SPDX-License-Identifier: GPL-3.0-or-later
# Converts an Xvfb -fbdir screen dump (XWD, 32 bpp TrueColor) to PNG (own code).
# Usage: python3 -I xwd2png.py IN.xwd OUT.png
import sys, struct, zlib
d = open(sys.argv[1], 'rb').read()
hs, ver, fmt, depth, w, h = struct.unpack('>6I', d[:24])
f = struct.unpack('>25I', d[:100])
bpl = f[12]; ncolors = f[19]
off = hs + ncolors * 12
rows = []
for y in range(h):
    r = d[off + y * bpl: off + y * bpl + w * 4]
    rows.append(b'\x00' + bytes(b for i in range(0, len(r), 4) for b in (r[i + 2], r[i + 1], r[i])))
def ch(t, b): return struct.pack('>I', len(b)) + t + b + struct.pack('>I', zlib.crc32(t + b) & 0xffffffff)
open(sys.argv[2], 'wb').write(b'\x89PNG\r\n\x1a\n' + ch(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0)) + ch(b'IDAT', zlib.compress(b''.join(rows))) + ch(b'IEND', b''))
