# SPDX-License-Identifier: GPL-3.0-or-later
# Convert an Xvfb framebuffer dump (XWD, 24/32 bpp ZPixmap) to PNG. Own code.
import struct, sys, zlib
def conv(src, dst, crop=None):
    d = open(src, 'rb').read()
    hdr = struct.unpack('>25I', d[:100])
    hsize, ver, fmt, depth, w, h = hdr[0:6]
    bpl = hdr[12]; bpp = hdr[11]; ncolors = hdr[19]
    off = hsize + ncolors * 12
    rows = []
    x0, y0, x1, y1 = crop or (0, 0, w, h)
    for y in range(y0, y1):
        r = d[off + y * bpl: off + (y + 1) * bpl]
        row = bytearray(b'\0')
        for x in range(x0, x1):
            b, g, rr = r[4*x], r[4*x+1], r[4*x+2]
            row += bytes((rr, g, b))
        rows.append(bytes(row))
    raw = b''.join(rows)
    def chunk(t, p): return struct.pack('>I', len(p)) + t + p + struct.pack('>I', zlib.crc32(t + p) & 0xffffffff)
    png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', x1-x0, y1-y0, 8, 2, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(raw, 6)) + chunk(b'IEND', b'')
    open(dst, 'wb').write(png)
if __name__ == '__main__':
    crop = tuple(map(int, sys.argv[3].split(','))) if len(sys.argv) > 3 else None
    conv(sys.argv[1], sys.argv[2], crop)
