# SPDX-License-Identifier: GPL-3.0-or-later
# Analysis helpers for the 3D-transform black-box study (own code).
import struct, zlib, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gif import read_gif, write_gif

F3D = [(126,'sphere'),(128,'rotx'),(130,'roty'),(132,'rotz'),(134,'scalex'),
       (136,'scaley'),(138,'rough'),(140,'water'),(142,'fill'),(144,'persp'),
       (146,'xshift'),(148,'yshift'),(150,'lx'),(152,'ly'),(154,'lz'),
       (156,'smooth'),(158,'u158'),(160,'u160'),(162,'u162'),(180,'stereo'),
       (254,'tlo'),(256,'thi'),(258,'amb'),(260,'haze'),(262,'rand')]

def fields(g):
    e = g['exts'].get('fractint001')
    if e is None: return None
    return {n: struct.unpack('<h', e[o:o+2])[0] for o, n in F3D}

def palette(g):
    t = g['gct']; return [tuple(t[3*i:3*i+3]) for i in range(len(t)//3)]

def png(path, w, h, pix, pal, scale=1):
    rows = []
    for y in range(h):
        r = bytearray([0])
        for x in range(w):
            c = pal[pix[y*w+x]]
            r += bytes(c) * scale
        rows.extend([bytes(r)] * scale)
    W, H = w*scale, h*scale
    def chunk(t, d):
        return struct.pack('>I', len(d)) + t + d + struct.pack('>I', zlib.crc32(t+d) & 0xffffffff)
    data = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', W, H, 8, 2, 0, 0, 0))
    data += chunk(b'IDAT', zlib.compress(b''.join(rows), 6)) + chunk(b'IEND', b'')
    open(path, 'wb').write(data)

def topng(gifpath, pngpath=None, scale=1):
    g = read_gif(gifpath)
    png(pngpath or gifpath[:-4] + '.png', g['w'], g['h'], g['pixels'], palette(g), scale)

if __name__ == '__main__':
    cmd = sys.argv[1]
    if cmd == 'png':
        for f in sys.argv[2:]: topng(f)
    elif cmd == 'info':
        for f in sys.argv[2:]:
            g = read_gif(f)
            e = g['exts'].get('fractint001')
            hist = {}
            for p in g['pixels']: hist[p] = hist.get(p, 0) + 1
            print(os.path.basename(f), g['w'], g['h'], g['order'],
                  'type', struct.unpack('<h', e[10:12])[0] if e else None,
                  'status', struct.unpack('<h', e[202:204])[0] if e else None)
            print('  ', fields(g))
            print('   colours used', len(hist), 'top', sorted(hist.items(), key=lambda t: -t[1])[:8])
