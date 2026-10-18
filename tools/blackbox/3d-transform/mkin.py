# SPDX-License-Identifier: GPL-3.0-or-later
# Writes synthetic input GIFs for 3D-transform tests (own code).
# Usage: python3 -I mkin.py OUT.gif W H EXPR [noblock] [template.gif]
#   EXPR: Python expression in x, y, W, H giving the colour index (0-255).
# Palette: uniq.map-style distinct colours (i*4 mod 256, ...) so that every
# index has its own RGB. With a template, its fractint001 block is copied
# (type and everything else kept); otherwise none is written.
# 'black=K' sets entry K to black.
# 'bits=N' writes a 2**N-entry global colour table instead (no Fractint block).
import sys, os, struct, math
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gif import read_gif, write_gif

def pal_uniq():
    # 256 distinct 6-bit-representable colours
    out = bytearray()
    for i in range(256):
        r6, g6, b6 = (i & 63), ((i >> 6) * 16 + (i & 15)) & 63, 63 - (i & 63)
        if i >= 64: b6 = (b6 + 21 * (i >> 6)) & 63
        for v in (r6, g6, b6): out.append((v << 2) | (v >> 4))
    return bytes(out)

def main():
    out, W, H, expr = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4]
    opts = sys.argv[5:]
    pix = bytearray()
    for y in range(H):
        for x in range(W):
            pix.append(int(eval(expr, {'math': math}, dict(x=x, y=y, W=W, H=H))) & 255)
    pal = pal_uniq()
    assert len(set(pal[3*i:3*i+3] for i in range(256))) == 256
    for o in opts:
        if o.startswith('black='):  # put pure black at entry K
            k = int(o[6:]); pal = pal[:3*k] + b'\0\0\0' + pal[3*k+3:]
    exts = []
    tmpl = [o for o in opts if o.endswith('.gif')]
    if tmpl:
        e = bytearray(read_gif(tmpl[0])['exts']['fractint001'])
        exts.append(('fractint001', bytes(e)))
    bits = [int(o[5:]) for o in opts if o.startswith('bits=')]
    if bits:
        # smaller global colour table: 2**bits entries, LZW min code size = max(2, bits)
        import struct
        from gif import subblocks, lzw_encode
        n = 1 << bits[0]; mcs = max(2, bits[0])
        o = bytearray(b'GIF89a') + struct.pack('<HHBBB', W, H, 0x80 | 0x70 | (bits[0]-1), 0, 0)
        o += pal[:3*n] + b'\x2c' + struct.pack('<HHHHB', 0, 0, W, H, 0)
        o += bytes([mcs]) + subblocks(lzw_encode(bytes(pix), mcs), 254) + b'\x3b'
        open(out, 'wb').write(o)
        return
    write_gif(out, W, H, pal, bytes(pix), exts)

main()
