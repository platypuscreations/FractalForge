# SPDX-License-Identifier: GPL-3.0-or-later
# Own model of XFractint's IFS type, written from black-box observations only
# (docs/specs/ifs.md). It predicts the pixels of a saved GIF and compares them.
#
#   python3 -I ifsmodel.py IMAGE.gif FILE.ifs ENTRY [rot=a/b/c] [shift=x/y]
#
# Reads corners, maxiter and param 1 from the GIF's fractint001 block, so only
# the 3D viewing values (rotation, xyshift) have to be repeated. Prints the
# number of pixels that differ after mapping both sides to the first palette
# index with the same colour (types.md 1.3). Standard library only.
import math, os, struct, sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'resume'))
from gif import read_gif  # noqa: E402


def glibc_rand(n, seed=1):
    """First n outputs of glibc rand() after srand(seed) (published algorithm)."""
    r = [0] * 34
    r[0] = seed or 1
    for i in range(1, 31):
        hi, lo = divmod(r[i - 1], 127773)
        w = 16807 * lo - 2836 * hi
        r[i] = w + 2147483647 if w < 0 else w
    for i in range(31, 34):
        r[i] = r[i - 31]
    out = []
    k = 34
    while len(out) < n:
        v = (r[k - 31] + r[k - 3]) & 0xffffffff
        r.append(v)
        if k >= 344:
            out.append(v >> 1)
        k += 1
    return out


def f32(v):
    return struct.unpack('<f', struct.pack('<f', v))[0]


def read_entry(path, name):
    """Returns (is3d, list of transforms) following spec section 1."""
    lines = open(path, encoding='latin-1').read().split('\n')
    for i, line in enumerate(lines):
        text = line.split(';')[0]
        if '{' not in text:
            continue
        head = text.split('{')[0].split()
        if not head or head[0].lower() != name.lower():
            continue
        is3d = len(head) > 1 and head[1].lower() == '(3d)'
        nums = []
        for body in lines[i + 1:]:
            body = body.split(';')[0]
            end = '}' in body
            body = body.split('}')[0].replace(',', ' ')
            try:
                nums += [float(t) for t in body.split()]
            except ValueError:
                return is3d, []  # text inside the numbers: invalid IFS definition
            if end:
                n = 13 if is3d else 7
                if not nums or len(nums) % n or len(nums) > 63 * n:
                    return is3d, []  # XFractint reports an error, image stays empty
                return is3d, [nums[j:j + n] for j in range(0, len(nums), n)]
    print('entry not found', file=sys.stderr)
    return False, []


def fix(v):
    """Single-precision value to 16.16 fixed point, truncated toward zero."""
    return int(f32(v) * 65536)


def fmul(a, b):
    p = a * b
    return p >> 16 if p >= 0 else -((-p) >> 16)


def render2d(ts, W, H, corners, maxiter, by_transform):
    xmin, xmax, ymin, ymax = corners
    co = [[fix(v) for v in t] for t in ts]
    cum, t = [], 0
    for c in co:
        t += c[6]
        cum.append(t)
    npts = 1024 * maxiter + 1
    rnd = glibc_rand(npts)
    img = bytearray(W * H)
    x = y = 0
    sx = (W - 1) / (xmax - xmin)
    sy = (H - 1) / (ymax - ymin)
    for n in range(npts):
        r2 = 2 * (rnd[n] & 0x7fff)
        k = next((j for j, c in enumerate(cum) if r2 <= c), len(co) - 1)
        a, b, c, d, e, f, _ = co[k]
        x, y = fmul(a, x) + fmul(b, y) + e, fmul(c, x) + fmul(d, y) + f
        col = math.floor((x / 65536 - xmin) * sx)
        row = math.floor((ymax - y / 65536) * sy)
        if abs(col) + abs(row) > 10000:
            break
        if 0 <= col < W and 0 <= row < H:
            i = row * W + col
            img[i] = (k + 1) & 255 if by_transform else (img[i] + 1) & 15
    return img


def rotation(a, b, c):
    a, b, c = (math.radians(v) for v in (a, b, c))
    ca, sa, cb, sb, cc, sc = math.cos(a), math.sin(a), math.cos(b), math.sin(b), math.cos(c), math.sin(c)
    rx = [[1, 0, 0], [0, ca, sa], [0, -sa, ca]]
    ry = [[cb, 0, -sb], [0, 1, 0], [sb, 0, cb]]
    rz = [[cc, sc, 0], [-sc, cc, 0], [0, 0, 1]]
    mul = lambda p, q: [[sum(p[i][k] * q[k][j] for k in range(3)) for j in range(3)] for i in range(3)]
    return mul(rz, mul(ry, rx))


def render3d(ts, W, H, corners, maxiter, by_transform, rot=(60, 30, 0), shift=(0, 0)):
    xmin, xmax, ymin, ymax = corners
    cum, t = [], 0.0
    for tr in ts:
        t += tr[12]
        cum.append(t * 2147483647)
    npts = 1024 * maxiter + 1
    rnd = glibc_rand(npts)
    R = rotation(*rot)
    x = y = z = 0.0
    pts = []
    for n in range(npts):
        k = next((j for j, c in enumerate(cum) if rnd[n] <= c), len(ts) - 1)
        m = ts[k]
        x, y, z = (m[0] * x + m[1] * y + m[2] * z + m[9],
                   m[3] * x + m[4] * y + m[5] * z + m[10],
                   m[6] * x + m[7] * y + m[8] * z + m[11])
        pts.append((R[0][0] * x + R[0][1] * y + R[0][2] * z,
                    R[1][0] * x + R[1][1] * y + R[1][2] * z, k))
    first = pts[:100]
    cx = (min(p[0] for p in first) + max(p[0] for p in first)) / 2 - shift[0] / 100 * (xmax - xmin)
    cy = (min(p[1] for p in first) + max(p[1] for p in first)) / 2 - shift[1] / 100 * (ymax - ymin)
    sx = (W - 1) / (xmax - xmin)
    sy = (H - 1) / (ymax - ymin)
    img = bytearray(W * H)
    for X, Y, k in pts[100:]:
        col = int((X - cx - xmin) * sx)
        row = int((ymax - (Y - cy)) * sy)
        if abs(col) + abs(row) > 10000:
            break
        if 0 <= col < W and 0 <= row < H:
            i = row * W + col
            img[i] = (k + 1) & 255 if by_transform else (img[i] + 1) & 15
    return img


def main():
    gp, ifs, name = sys.argv[1:4]
    opts = dict(a.split('=', 1) for a in sys.argv[4:])
    g = read_gif(gp)
    b = g['exts']['fractint001']
    corners = struct.unpack('<4d', b[12:44])
    maxiter = struct.unpack('<i', b[408:412])[0]
    by_transform = int(struct.unpack('<d', b[44:52])[0]) != 0
    is3d, ts = read_entry(ifs, name)
    if not ts:
        img = bytearray(g['w'] * g['h'])
    elif is3d:
        rot = tuple(float(v) for v in opts.get('rot', '60/30/0').split('/'))
        shift = tuple(float(v) for v in opts.get('shift', '0/0').split('/'))
        img = render3d(ts, g['w'], g['h'], corners, maxiter, by_transform, rot, shift)
    else:
        img = render2d(ts, g['w'], g['h'], corners, maxiter, by_transform)
    gct = g['gct']
    first = {}
    canon = [first.setdefault(gct[3 * i:3 * i + 3], i) for i in range(256)]
    px = g['pixels']
    bad = sum(1 for i in range(len(px)) if canon[img[i]] != canon[px[i]])
    print('%s: %d of %d pixels differ (%d lit in XFractint, %d in model)' % (
        os.path.basename(gp), bad, len(px), sum(1 for v in px if v), sum(1 for v in img if v)))


if __name__ == '__main__':
    main()
