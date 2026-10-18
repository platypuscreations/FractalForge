# SPDX-License-Identifier: GPL-3.0-or-later
# Reproduces the orbit-map evidence with the algorithms of
# docs/specs/orbit-maps.md and prints the pixel match per file.
# Own code, black-box derived; standard library only.
#
#   python3 -I verify.py CORPUS-DIR [ORBIT-DIR] [NAME-PREFIX...]
#
# ORBIT-DIR holds orbits computed with the container's glibc libm by
# libm-orbits.sh (needed for martin, chip, quadruptwo and threeply, whose
# chaotic orbits depend on the last bit of sin/cos/log/atan). Without it
# those types use the host's libm and may match less.
import sys, os, math, struct, collections, cmath
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from orbitsim import load, Mapper, compare, sgn
import xmath

# ---------------------------------------------------------------- colours
def start_colour(inside):
    """Starting colour of the time-coloured single-orbit types (§3.3)."""
    if 1 <= inside <= 255:
        return inside
    if inside >= 256:
        return 1
    return 2

def timecolour(j, c0):
    return ((c0 - 1 + (j + 1) // 1001) % 255) + 1

# ---------------------------------------------------------------- steps
def step_fn(t, p, fns):
    a, b, c, d = p[0], p[1], p[2], p[3]
    if t == 'henon':
        return lambda x, y: (1 + y - a * x * x, b * x)
    if t == 'hopalong':
        return lambda x, y: (y - sgn(x) * math.sqrt(abs(b * x - c)), a - x)
    if t == 'martin':
        return lambda x, y: (y - math.sin(x), a - x)
    if t == 'gingerbreadman':
        return lambda x, y: (1 - y + abs(x), x)
    if t == 'chip':
        def f(x, y):
            l1 = math.log(abs(b * x - c)); l2 = math.log(abs(c * x - b))
            return y - sgn(x) * math.cos(l1 * l1) * math.atan(l2 * l2), a - x
        return f
    if t == 'quadruptwo':
        def f(x, y):
            l1 = math.log(abs(b * x - c)); l2 = math.log(abs(c * x - b))
            return y - sgn(x) * math.sin(l1) * math.atan(l2 * l2), a - x
        return f
    if t == 'threeply':
        cb = math.cos(b); s3 = math.sin(a + b + c)
        return lambda x, y: (y - sgn(x) * abs(math.sin(x) * cb + c - x * s3), a - x)
    raise KeyError(t)

START = dict(henon=lambda p: (1.0, 1.0), hopalong=lambda p: (0.0, 0.0),
             martin=lambda p: (0.0, 0.0), gingerbreadman=lambda p: (p[0], p[1]),
             chip=lambda p: (0.0, 0.0), quadruptwo=lambda p: (0.0, 0.0),
             threeply=lambda p: (0.0, 0.0))

def orbit_points(t, p, n, orbitdir, name):
    """Start point plus n iterates."""
    if orbitdir:
        fn = os.path.join(orbitdir, name + '.bin')
        if os.path.exists(fn):
            data = open(fn, 'rb').read()
            if len(data) >= 16 * (n + 1):
                return [struct.unpack_from('<dd', data, 16 * k) for k in range(n + 1)]
    st = step_fn(t, p, None)
    x, y = START[t](p); pts = [(x, y)]
    for _ in range(n):
        try:
            x, y = st(x, y)
        except (ValueError, OverflowError):
            x = y = float('nan')
        pts.append((x, y))
    return pts

def render_single(g, t, orbitdir, name):
    f = g['f']; m = Mapper(g); p = f['params']
    c0 = start_colour(f['inside'])
    n = 1024 * f['maxiter']
    img = [0] * (m.W * m.H)
    for j, (x, y) in enumerate(orbit_points(t, p, n, orbitdir, name)):
        q = m.pix(x, y)
        if q >= 0:
            img[q] = timecolour(j, c0)
    return img

# ---------------------------------------------------------------- hit-count types
def plot_hit(img, q, canon):
    if q >= 0:
        v = canon[img[q]] + 1          # read back the displayed colour, add 1
        img[q] = min(v, 255)

def _inf(f):
    def g(v):
        try:
            return f(v)
        except OverflowError:
            return math.copysign(math.inf, v) if f is math.sinh else math.inf
    return g

FN = {'sin': xmath.xsin, 'cos': xmath.xcos, 'ident': lambda v: v,
      'sqr': lambda v: v * v, 'tan': xmath.xtan,
      'sinh': _inf(math.sinh), 'cosh': _inf(math.cosh)}
FNIDX = {0: 'sin', 9: 'cos', 8: 'ident', 6: 'sqr', 10: 'tan', 2: 'sinh', 3: 'cosh'}

def fnames(g):
    b = g['exts']['fractint001']
    return [FNIDX.get(b[218 + k], '?') for k in range(4)]

def render_icons(g):
    m = Mapper(g); p = g['f']['params']; canon = g['canon']
    lam, al, be, ga, om, deg = p[0], p[1], p[2], p[3], p[4], int(p[5])
    img = [0] * (m.W * m.H)
    x, y = 0.01, 0.003
    plot_hit(img, m.pix(x, y), canon)
    for _ in range(1024 * g['f']['maxiter']):
        zz = x * x + y * y; zr, zi = x, y
        for _ in range(deg - 2):
            zr, zi = zr * x - zi * y, zr * y + zi * x
        zn = x * zr - y * zi
        pp = lam + al * zz + be * zn
        x, y = pp * x + ga * zr - om * y, pp * y - ga * zi + om * x
        plot_hit(img, m.pix(x, y), canon)
    return img

def render_latoocarfian(g):
    m = Mapper(g); p = g['f']['params']; canon = g['canon']
    a, b, c, d = p[:4]
    names = fnames(g)
    if '?' in names or any(nm not in FN for nm in names):
        return None
    f1, f2, f3, f4 = [FN[nm] for nm in names]
    img = [0] * (m.W * m.H)
    x, y = 1.0, 1.0
    plot_hit(img, m.pix(x, y), canon)
    for _ in range(1024 * g['f']['maxiter']):
        if not (math.isfinite(x) and math.isfinite(y)):
            break                      # the orbit has overflowed; nothing more is plotted
        try:
            x, y = f1(y * b) + c * f2(x * b), f3(x * a) + d * f4(y * a)
        except Exception:
            x = y = math.nan
        plot_hit(img, m.pix(x, y), canon)
    return img

# ---------------------------------------------------------------- kamtorus
def render_kamtorus(g):
    m = Mapper(g); p = g['f']['params']
    ang, step, stop, npts = p[0], p[1], p[2], int(p[3])
    ca, sa = math.cos(ang), math.sin(ang)
    c0 = start_colour(g['f']['inside'])
    L = npts + 1
    img = [0] * (m.W * m.H)
    t = 0.0; o = 0
    while t <= stop:
        x = y = t / 3
        for k in range(1, L + 1):
            x, y = x * ca + (x * x - y) * sa, x * sa - (x * x - y) * ca
            q = m.pix(x, y)
            if q >= 0:
                j = (o - 1) * L + k
                img[q] = timecolour(max(j, 0), c0) if o else c0
        o += 1; t += step
    return img

# ---------------------------------------------------------------- popcorn family
def _safe(f):
    def g(v):
        try:
            return f(v)
        except OverflowError:
            return 1.0
    return g

PFN = {'sin': math.sin, 'cos': math.cos, 'sqr': lambda v: v * v, 'ident': lambda v: v,
       'tan': lambda v: math.sin(2 * v) / (math.cos(2 * v) + 1.0),
       'sinh': _safe(math.sinh), 'cosh': _safe(math.cosh)}

def popcorn_step(g):
    p = g['f']['params']; h, t3 = p[0], p[2]
    f1, f2, f3, f4 = [PFN[nm] for nm in fnames(g)]
    return lambda x, y: (x - h * f1(y + f2(t3 * y)), y - h * f3(x + f4(t3 * x)))

def render_popcorn(g):
    m = Mapper(g); f = g['f']; M = f['maxiter']
    xmin, xmax, ymin, ymax = f['corners']; W, H = m.W, m.H
    st = popcorn_step(g)
    img = [0] * (W * H)
    for r in range(H):
        y0 = ymax - r * (ymax - ymin) / (H - 1)
        col = (r + 1) % 256
        for c in range(W):
            x, y = xmin + c * (xmax - xmin) / (W - 1), y0
            for _ in range(M - 1):
                try:
                    x, y = st(x, y)
                except (ValueError, OverflowError):
                    break
                q = m.pix(x, y)
                if q >= 0:
                    img[q] = col
                if x * x + y * y > 4:
                    break
    return img

def render_popcornjul(g):
    m = Mapper(g); f = g['f']; M = f['maxiter']; B = f['bailout'] or 4
    xmin, xmax, ymin, ymax = f['corners']; W, H = m.W, m.H
    st = popcorn_step(g)
    img = [0] * (W * H)
    for r in range(H):
        y0 = ymax - r * (ymax - ymin) / (H - 1)
        for c in range(W):
            x, y = xmin + c * (xmax - xmin) / (W - 1), y0; e = None
            for k in range(1, M + 1):
                try:
                    x, y = st(x, y)
                except (ValueError, OverflowError):
                    e = k; break
                if x * x + y * y > B:
                    e = k; break
            img[r * W + c] = 0 if e is None or e >= M else (e if e <= 255 else ((e - 1) % 255) + 1)
    return img

# ---------------------------------------------------------------- grid types
def render_dynamic(g):
    m = Mapper(g); f = g['f']; M = f['maxiter']; p = f['params']
    n, dt, a, b = int(p[0]), p[1], p[2], p[3]
    xmin, xmax, ymin, ymax = f['corners']
    off = max(f['inside'], 0)
    img = [0] * (m.W * m.H); idx = 0
    S = math.sin
    fi = {'sin': math.sin, 'cos': math.cos, 'sqr': lambda v: v * v, 'ident': lambda v: v}[fnames(g)[0]]
    for i in range(n + 1):
        for j in range(n):
            x = xmin + (j + 0.5) * (xmax - xmin) / n; y = ymax - (i + 0.5) * (ymax - ymin) / n
            col = (idx + off) % 255 + 1
            q = m.pix(x, y)
            if q >= 0: img[q] = col
            for _ in range(M - 1):
                x, y = x - dt * S(y + a * fi(b * y)), y + dt * S(x + a * fi(b * x))
                q = m.pix(x, y)
                if q >= 0: img[q] = col
            idx += 1
    return img

def render_mandelcloud(g):
    m = Mapper(g); f = g['f']; M = f['maxiter']; n = int(f['params'][0])
    xmin, xmax, ymin, ymax = f['corners']
    img = [0] * (m.W * m.H); idx = 0
    for i in range(n + 1):
        for j in range(n):
            cx = xmin + (j + 0.5) * (xmax - xmin) / n; cy = ymax - (i + 0.5) * (ymax - ymin) / n
            col = idx % 255 + 1; x, y = cx, cy
            for _ in range(M - 1):
                x, y = x * x - y * y + cx, 2 * x * y + cy
                if x * x + y * y > 4: break
                q = m.pix(x, y)
                if q >= 0: img[q] = col
            idx += 1
    return img

def render_julia_inverse(g):
    """Approximation only (§9): breadth-first preimage tree from the
    repelling fixed point, at most P3 hits per pixel, colour = hits."""
    m = Mapper(g); p = g['f']['params']
    c = complex(p[0], p[1]); maxhits = int(p[2])
    img = [0] * (m.W * m.H); cnt = collections.Counter()
    zf = 0.5 - cmath.sqrt(0.25 - c)
    queue = collections.deque([zf]); steps = 0
    while queue and steps < 2000000:
        z = queue.popleft(); steps += 1
        q = m.pix(z.real, z.imag)
        if q >= 0:
            if cnt[q] >= maxhits: continue
            cnt[q] += 1; img[q] = cnt[q]
        w = cmath.sqrt(z - c)
        queue.append(-w); queue.append(w)
    return img

TYPES = {88: 'henon', 120: 'hopalong', 122: 'martin', 90: 'gingerbreadman', 166: 'chip',
         167: 'quadruptwo', 168: 'threeply', 159: 'icons', 171: 'latoocarfian', 40: 'kamtorus',
         61: 'popcorn', 105: 'popcornjul', 139: 'dynamic', 146: 'mandelcloud', 145: 'julia_inverse'}

def render(g, name, orbitdir):
    t = TYPES[g['f']['type']]
    if t in START:
        return render_single(g, t, orbitdir, name)
    return globals()['render_' + t](g)

if __name__ == '__main__':
    corpus = sys.argv[1]
    orbitdir = sys.argv[2] if len(sys.argv) > 2 and sys.argv[2] != '-' else None
    prefixes = sys.argv[3:]
    for fn in sorted(os.listdir(corpus)):
        if not fn.endswith('.gif'): continue
        name = fn[:-4]
        if prefixes and not any(name.startswith(pr) for pr in prefixes): continue
        g = load(os.path.join(corpus, fn))
        img = render(g, name, orbitdir)
        if img is None:
            print('%-28s %-14s skipped (functions not modelled)' % (name, TYPES[g['f']['type']])); continue
        pc, mm = compare(g, img)
        print('%-28s %-14s %8.4f %% %6d mismatches' % (name, TYPES[g['f']['type']], pc, len(mm)))
        sys.stdout.flush()
