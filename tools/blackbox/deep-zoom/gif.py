# SPDX-License-Identifier: GPL-3.0-or-later
# Minimal GIF reader/writer for black-box analysis (own code).
import struct, sys

def read_gif(path):
    d = open(path, 'rb').read()
    assert d[:6] in (b'GIF89a', b'GIF87a')
    w, h, flags, bg, asp = struct.unpack('<HHBBB', d[6:13])
    p = 13
    gct = None
    if flags & 0x80:
        n = 3 * (2 << (flags & 7))
        gct = d[p:p+n]; p += n
    exts = []  # list of (id11, payload)
    pixels = None
    order = []
    while p < len(d):
        b = d[p]
        if b == 0x21:
            label = d[p+1]; p += 2
            subs = []
            while True:
                n = d[p]; p += 1
                if n == 0: break
                subs.append(d[p:p+n]); p += n
            if label == 0xFF:
                ident = subs[0]
                payload = b''.join(subs[1:])
                exts.append((ident.decode('latin1'), payload))
                order.append(ident.decode('latin1'))
            else:
                order.append('ext%02x' % label)
        elif b == 0x2C:
            l, t, iw, ih, f = struct.unpack('<HHHHB', d[p+1:p+10]); p += 10
            if f & 0x80:
                p += 3 * (2 << (f & 7))
            mcs = d[p]; p += 1
            data = bytearray()
            while True:
                n = d[p]; p += 1
                if n == 0: break
                data += d[p:p+n]; p += n
            pixels = lzw_decode(bytes(data), mcs, iw * ih)
            order.append('image')
        elif b == 0x3B:
            break
        else:
            raise ValueError('bad byte %x at %d' % (b, p))
    return dict(w=w, h=h, gct=gct, exts=dict(exts), pixels=pixels, order=order, flags=flags)

def lzw_decode(data, mcs, npix):
    clear = 1 << mcs; eoi = clear + 1
    out = bytearray()
    bitpos = 0; nbits = len(data) * 8
    size = mcs + 1
    table = None; prev = None
    def reset():
        return [bytes([i]) for i in range(clear)] + [b'', b'']
    table = reset()
    val = int.from_bytes(data, 'little')
    while bitpos + size <= nbits:
        code = (val >> bitpos) & ((1 << size) - 1); bitpos += size
        if code == clear:
            table = reset(); size = mcs + 1; prev = None; continue
        if code == eoi: break
        if prev is None:
            e = table[code]; out += e; prev = e; continue
        if code < len(table):
            e = table[code]
            table.append(prev + e[:1])
        else:
            e = prev + prev[:1]
            table.append(e)
        out += e; prev = e
        if len(table) == (1 << size) and size < 12:
            size += 1
        if len(out) >= npix: break
    return bytes(out[:npix])

def lzw_encode_literal(pixels, mcs=8):
    # literal codes only, clear every 254 symbols; code size stays 9
    clear = 1 << mcs; eoi = clear + 1
    bits = []; acc = 0; nb = 0; out = bytearray()
    def put(c):
        nonlocal acc, nb
        acc |= c << nb; nb += mcs + 1
        while nb >= 8:
            out.append(acc & 0xFF); acc >>= 8; nb -= 8
    put(clear)
    for i, px in enumerate(pixels):
        if i and i % 254 == 0: put(clear)
        put(px)
    put(eoi)
    if nb: out.append(acc & 0xFF)
    return bytes(out)

def subblocks(b, n=255):
    o = bytearray()
    for i in range(0, len(b), n):
        c = b[i:i+n]; o.append(len(c)); o += c
    o.append(0)
    return bytes(o)

def write_gif(path, w, h, gct, pixels, exts, literal=False):
    # exts: list of (ident11 str, payload bytes), written after image in order
    o = bytearray(b'GIF89a')
    o += struct.pack('<HHBBB', w, h, 0xD7, 0, 0x31)
    o += gct
    o += b'\x2c' + struct.pack('<HHHHB', 0, 0, w, h, 0)
    o += b'\x08' + subblocks((lzw_encode_literal if literal else lzw_encode)(pixels), 254)
    for ident, payload in exts:
        o += b'\x21\xff\x0b' + ident.encode('latin1') + subblocks(payload)
    o += b'\x3b'
    open(path, 'wb').write(o)

def i32s(payload):
    return list(struct.unpack('<%di' % (len(payload)//4), payload))

if __name__ == '__main__':
    for f in sys.argv[1:]:
        g = read_gif(f)
        e2 = g['exts'].get('fractint002')
        e1 = g['exts']['fractint001']
        st = struct.unpack('<h', e1[202:204])[0]
        print(f, g['w'], g['h'], g['order'], 'status', st, 'pass', chr(e1[200]),
              'r002', i32s(e2)[:12] if e2 else None, 'nz>40:', [ (i*4,v) for i,v in enumerate(i32s(e2)) if i>=10 and v] if e2 else '')

def lzw_encode(pixels, mcs=8):
    # standard variable-width GIF LZW with a clear code when the table fills
    clear = 1 << mcs; eoi = clear + 1
    out = bytearray(); acc = 0; nb = 0
    size = mcs + 1
    def put(c):
        nonlocal acc, nb
        acc |= c << nb; nb += size
        while nb >= 8:
            out.append(acc & 0xFF); acc >>= 8; nb -= 8
    table = {}; nxt = eoi + 1
    put(clear)
    cur = None
    for px in pixels:
        if cur is None: cur = px; continue
        key = (cur, px)
        if key in table: cur = table[key]; continue
        put(cur)
        if nxt < 4096:
            table[key] = nxt; nxt += 1
            if nxt > (1 << size) and size < 12: size += 1
        else:
            put(clear); table = {}; nxt = eoi + 1; size = mcs + 1
        cur = px
    if cur is not None: put(cur)
    put(eoi)
    if nb: out.append(acc & 0xFF)
    return bytes(out)
