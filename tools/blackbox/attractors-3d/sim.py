# SPDX-License-Identifier: GPL-3.0-or-later
"""Black-box model of XFractint's ODE / 3D orbit types (own code, own names).

Usage: python3 -I sim.py [--type NAME] FILE.gif [...]
Re-renders each GIF from the parameters stored in its fractint001 block with
the hypothesised algorithm (docs/specs/attractors-3d.md) and prints the share
of pixels whose RGB colour matches, and the share whose drawn/blank state
matches. pickover needs Docker (orbit.pl runs inside the black-box image so
that sin/cos come from the same glibc as XFractint); set ORBIT_TMP to a
writable directory Docker can mount if the default temp dir is not shared.
"""
import math, struct, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gif import read_gif

TYPE_NAMES = {63: 'lorenz', 84: 'lorenz3d', 124: 'lorenz3d1', 125: 'lorenz3d3',
              126: 'lorenz3d4', 86: 'rossler3d', 89: 'pickover', 42: 'kamtorus3d',
              160: 'icons3d'}


def load(path):
    g = read_gif(path)
    b = g['exts']['fractint001']
    f64 = lambda o: struct.unpack_from('<d', b, o)[0]
    i16 = lambda o: struct.unpack_from('<h', b, o)[0]
    g['type'] = i16(10)
    g['corners'] = (f64(12), f64(20), f64(28), f64(36))
    g['params'] = [f64(44), f64(52), f64(270), f64(278)] + [f64(354 + 8 * k) for k in range(6)]
    g['maxiter'] = struct.unpack_from('<i', b, 408)[0]
    g['rot'] = (i16(128), i16(130), i16(132))
    g['persp'] = i16(144)
    g['shift'] = (i16(146), i16(148))
    return g


# ---------------------------------------------------------------- equations
def lorenz_step(p):
    dt, a, b, c = p[0], p[1], p[2], p[3]
    def step(x, y, z):
        xdt = dt * x
        ydt = dt * y
        return (x + ((dt * a) * y - (dt * a) * x),
                y + (((dt * b) * x - ydt) - xdt * z),
                z + (ydt * x - (dt * c) * z))
    return step


def rossler_step(p):
    dt, a, b, c = p[0], p[1], p[2], p[3]
    def step(x, y, z):
        xdt, ydt, zdt = dt * x, dt * y, dt * z
        return (x + (-ydt - zdt),
                y + (xdt + (dt * a) * y),
                z + (((dt * b) + xdt * z) - (dt * c) * z))
    return step


def proto_step(p):
    # one-lobe ("proto") Lorenz system; arithmetic order NOT known (see spec)
    dt, a, b, c = p[0], p[1], p[2], p[3]
    def step(x, y, z):
        r = math.sqrt(x * x + y * y)
        return (x + dt * (-(a + 1) * x + (a - b + z) * y + (1 - a) * r),
                y + dt * ((b - a - z) * x - (a + 1) * y + (b + a - z) * r),
                z + dt * (y / 2 - c * z))
    return step


def cover_step(n):
    # n-fold cover of the one-lobe system (W**n = u + iv); arithmetic order NOT known
    def mk(p):
        dt, a, b, c = p[0], p[1], p[2], p[3]
        def step(x, y, z):
            W = complex(x, y)
            w = W ** n
            u, v, r = w.real, w.imag, abs(w)
            du = -(a + 1) * u + (a - b + z) * v + (1 - a) * r
            dv = (b - a - z) * u - (a + 1) * v + (b + a - z) * r
            dW = W * complex(du, dv) / (n * w)
            return (x + dt * dW.real, y + dt * dW.imag, z + dt * (v / 2 - c * z))
        return step
    return mk


PREFIX = {'lorenz3d': 100, 'rossler3d': 500, 'pickover': 100,
          'lorenz3d1': 750, 'lorenz3d3': 750, 'lorenz3d4': 750}
POINTS = {'pickover'}          # types plotted as single points instead of lines
EXTERNAL = {'pickover'}        # orbits computed with glibc libm via orbit.pl
STEPS = {'lorenz': lorenz_step, 'lorenz3d': lorenz_step, 'rossler3d': rossler_step,
         'lorenz3d1': proto_step, 'lorenz3d3': cover_step(3), 'lorenz3d4': cover_step(4)}
START = {'lorenz': (1.0, 1.0, 1.0), 'lorenz3d': (1.0, 1.0, 1.0), 'rossler3d': (1.0, 1.0, 1.0),
         'pickover': (1.0, 1.0, 1.0), 'lorenz3d1': (1.0, 1.0, 1.0),
         'lorenz3d3': (1.0, 1.0, 1.0), 'lorenz3d4': (1.0, 1.0, 1.0)}


def external_step(typ, g):
    import orbit
    pts = iter(orbit.orbit(typ, 1024 * g['maxiter'] + PREFIX[typ] + 2, START[typ], g['params'][:4]))
    return lambda x, y, z: next(pts)


# ---------------------------------------------------------------- drawing
def line(a, b):
    """Pixels from a to b: major axis stepped, minor coordinate rounded half up."""
    (x0, y0), (x1, y1) = a, b
    dx, dy = x1 - x0, y1 - y0
    m = max(abs(dx), abs(dy))
    if m == 0:
        return [a]
    f = lambda s0, d, i: (2 * (s0 * m + i * d) + m) // (2 * m)
    return [(f(x0, dx, i), f(y0, dy, i)) for i in range(m + 1)]


FAR_COL, FAR_ROW = 6000, 4800   # hypothesis: true limits lie in [5413, 6400) and [4200, 5400)


def colour_2d(i):
    """Colour of the segment that ends at iterate i (i >= 1)."""
    c = 2 if i <= 999 else 3 + (i - 1000) // 1001
    return (c - 1) % 255 + 1


def rot_matrix(rx, ry, rz):
    def R(axis, a):
        c, s = math.cos(a), math.sin(a)
        if axis == 0:
            return [[1, 0, 0], [0, c, -s], [0, s, c]]
        if axis == 1:
            return [[c, 0, s], [0, 1, 0], [-s, 0, c]]
        return [[c, -s, 0], [s, c, 0], [0, 0, 1]]
    M = [[1, 0, 0], [0, 1, 0], [0, 0, 1]]
    for axis, deg in enumerate((rx, ry, rz)):
        A = R(axis, -math.radians(deg))
        M = [[sum(A[i][k] * M[k][j] for k in range(3)) for j in range(3)] for i in range(3)]
    return M


def render(g, typ=None):
    """Re-renders a lorenz / 3D orbit image (see docs/specs/attractors-3d.md)."""
    W, H = g['w'], g['h']
    x0, x1, y0, y1 = g['corners']
    typ = typ or TYPE_NAMES[g['type']]
    step = external_step(typ, g) if typ in EXTERNAL else STEPS[typ](g['params'])
    N = 1024 * g['maxiter']
    img = [0] * (W * H)
    def toint(t):
        # out-of-range conversions: x86-64 cvttsd2si yields INT_MIN (hypothesis)
        if not (-2147483648.0 < t < 2147483648.0):
            return -2147483648
        return int(t)
    tocol = lambda u: toint((u - x0) / (x1 - x0) * (W - 1))
    torow = lambda v: toint((y1 - v) / (y1 - y0) * (H - 1))
    inside = lambda q: 0 <= q[0] < W and 0 <= q[1] < H
    far = lambda q: abs(q[0]) > FAR_COL or abs(q[1]) > FAR_ROW
    def plot(prev, cur, c, points):
        if points or not inside(prev):
            if inside(cur):
                img[cur[1] * W + cur[0]] = c
            return
        if inside(cur):
            for (px, py) in line(prev, cur):
                img[py * W + px] = c
    x, y, z = START[typ]
    if typ == 'lorenz':
        prev = (tocol(x), torow(z))
        for i in range(1, N + 1):
            x, y, z = step(x, y, z)
            cur = (tocol(x), torow(z))
            if far(cur):
                break
            plot(prev, cur, colour_2d(i), False)
            prev = cur
        return img
    # 3D types
    M = rot_matrix(*g['rot'])
    view = lambda x, y, z: (M[0][0] * x + M[0][1] * y + M[0][2] * z,
                            M[1][0] * x + M[1][1] * y + M[1][2] * z,
                            M[2][0] * x + M[2][1] * y + M[2][2] * z)
    lo = [math.inf] * 3; hi = [-math.inf] * 3
    nbox = PREFIX[typ]
    for i in range(1, nbox + 1):
        x, y, z = step(x, y, z)
        uvd = view(x, y, z)
        for k in range(3):
            if uvd[k] < lo[k]:
                lo[k] = uvd[k]
            elif uvd[k] > hi[k]:          # "else if": see spec section 4.2
                hi[k] = uvd[k]
    cu = (lo[0] + hi[0]) / 2 - g['shift'][0] * (x1 - x0) / 100
    cv = (lo[1] + hi[1]) / 2 - g['shift'][1] * (y1 - y0) / 100
    ext = hi[2] - lo[2]
    persp = g['persp']
    def pix(x, y, z):
        u, v, d = view(x, y, z)
        u -= cu; v -= cv
        if persp:
            den = (persp / 100) * ext + (d - hi[2])
            if not den > 0:
                return None                # behind the viewer: plotting stops
            f = (persp / 100) * ext / den
            u *= f; v *= f
        return (tocol(u), torow(v))
    x, y, z = step(x, y, z)          # iterate nbox+1: first "previous" point
    prev = pix(x, y, z)
    if prev is None or far(prev):
        return img
    points = typ in POINTS
    if points and inside(prev):
        img[prev[1] * W + prev[0]] = colour_2d(nbox)
    for i in range(nbox + 2, N + 2):
        x, y, z = step(x, y, z)
        cur = pix(x, y, z)
        if cur is None or far(cur):
            break
        plot(prev, cur, colour_2d(i - 1), points)
        prev = cur
    return img


def render_kamtorus3d(g):
    """Partial model (see spec): orbits o = 0, step, 2*step, ... (repeated addition),
    each started at x = y = o/3 and iterated P4+1 times; point (x, y, o) is
    rotated like the other 3D types but NOT centred; stops at the first far point."""
    W, H = g['w'], g['h']
    x0, x1, y0, y1 = g['corners']
    a, step, _stop, n = g['params'][:4]
    n = int(n) + 1
    ca, sa = math.cos(a), math.sin(a)
    M = rot_matrix(*g['rot'])
    img = [0] * (W * H)
    i = 0
    o = 0.0
    while True:
        x = y = o / 3
        for _ in range(n):
            x2 = x * x
            x, y = x * ca + (x2 - y) * sa, x * sa - (x2 - y) * ca
            i += 1
            u = M[0][0] * x + M[0][1] * y + M[0][2] * o
            v = M[1][0] * x + M[1][1] * y + M[1][2] * o
            fc = (u - x0) / (x1 - x0) * (W - 1)
            fr = (y1 - v) / (y1 - y0) * (H - 1)
            if not (-6000 <= fc <= 6000 and -4800 <= fr <= 4800):
                return img
            c, r = int(fc), int(fr)
            if 0 <= c < W and 0 <= r < H:
                img[r * W + c] = colour_2d(i - n)
        o += step


def compare(g, img):
    """Pixels whose RGB differs (indices with equal RGB count as equal, types.md §1.3)."""
    px, pal = g['pixels'], g['gct']
    rgb = lambda c: pal[3 * c:3 * c + 3]
    return sum(1 for a, b in zip(px, img) if a != b and rgb(a) != rgb(b))


def compare_lit(g, img):
    """Pixels where XFractint and the model disagree on drawn / not drawn."""
    return sum(1 for a, b in zip(g['pixels'], img) if (a == 0) != (b == 0))


if __name__ == '__main__':
    args = sys.argv[1:]
    typ = None
    if args and args[0] == '--type':
        typ = args[1]; args = args[2:]
    for path in args:
        g = load(path)
        t = typ or TYPE_NAMES.get(g['type'])
        if t == 'kamtorus3d':
            img = render_kamtorus3d(g)
        elif t not in STEPS and t not in EXTERNAL:
            print('%-28s type=%-3d not modelled' % (os.path.basename(path), g['type']))
            continue
        else:
            img = render(g, typ)
        n = g['w'] * g['h']
        bad, badlit = compare(g, img), compare_lit(g, img)
        print('%-28s type=%-3d colour match %8.4f%% (%6d px)   drawn/blank match %8.4f%% (%6d px)' % (
            os.path.basename(path), g['type'], 100 - 100 * bad / n, bad, 100 - 100 * badlit / n, badlit))
