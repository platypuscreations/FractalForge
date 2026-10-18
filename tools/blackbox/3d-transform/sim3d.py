# SPDX-License-Identifier: GPL-3.0-or-later
# Simulator for XFractint's 3D transform (planar and spherical), written from
# black-box observations only (see docs/specs/3d-transform.md). Own code.
#
# Usage: python3 -I sim3d.py CASES.txt INDIR OUTDIR
#   CASES.txt lines:  NAME INPUT ARGS...   (same format as runcases.sh)
#   Prints, per case, the percentage of output pixels whose colour index
#   equals XFractint's (OUTDIR/NAME.gif).
import sys, os, math
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gif import read_gif

DEFAULTS = dict(rotation=(60, 30, 0), scalexyz=(90, 90, 30), waterline=0,
                filltype=0, perspective=0, xyshift=(0, 0), transparent=(0, 0),
                sphere=False, longitude=None, latitude=None, radius=None)


def parse_args(args):
    p = dict(DEFAULTS)
    for a in args:
        if '=' not in a:
            continue
        k, v = a.split('=', 1)
        nums = lambda: tuple(float(t) for t in v.split('/'))
        if k == 'rotation': p['rotation'] = nums()
        elif k == 'scalexyz':
            n = nums(); p['scalexyz'] = n + DEFAULTS['scalexyz'][len(n):]
        elif k == 'roughness': p['scalexyz'] = p['scalexyz'][:2] + (float(v),)
        elif k == 'waterline': p['waterline'] = float(v)
        elif k == 'filltype': p['filltype'] = int(v)
        elif k == 'perspective': p['perspective'] = float(v)
        elif k == 'xyshift': p['xyshift'] = nums()
        elif k == 'transparent': p['transparent'] = nums()
        elif k == 'sphere': p['sphere'] = v.lower().startswith('y')
        elif k == 'longitude': p['longitude'] = nums()
        elif k == 'latitude': p['latitude'] = nums()
        elif k == 'radius': p['radius'] = float(v)
    return p


def canvas_size(w, h):
    """Output size: kept if within 4 % of 4:3, else widened/heightened to 4:3."""
    r = 4*h/(3*w)
    if abs(r - 1) <= 0.04 + 1e-12:
        return w, h
    if w*3 > h*4:                       # too wide: add rows
        return w, int(w*3/4 + 0.5)
    return int(h*4/3 + 0.5), h          # too tall: add columns


def rotate(ax, ay, az):
    cx, sx = math.cos(math.radians(ax)), math.sin(math.radians(ax))
    cy, sy = math.cos(math.radians(ay)), math.sin(math.radians(ay))
    cz, sz = math.cos(math.radians(az)), math.sin(math.radians(az))
    def R(u, v, w):
        v, w = cx*v - sx*w, sx*v + cx*w          # about x (screen y down)
        u, w = cy*u + sy*w, -sy*u + cy*w         # about y
        u, v = cz*u + sz*v, -sz*u + cz*v         # about z
        return u, v, w
    return R


def dda(x0, y0, x1, y1):
    n = max(abs(x1-x0), abs(y1-y0))
    if n == 0:
        return [(x0, y0)]
    rnd = lambda t: int(math.floor(t + 0.5))
    return [(rnd(x0 + (x1-x0)*i/n), rnd(y0 + (y1-y0)*i/n)) for i in range(n+1)]


def tri(a, b, c):
    rows = {}
    for p, q in ((a, b), (b, c), (c, a)):
        for x, y in dda(p[0], p[1], q[0], q[1]):
            lo, hi = rows.get(y, (x, x)); rows[y] = (min(lo, x), max(hi, x))
    return [(x, y) for y, (lo, hi) in rows.items() for x in range(lo, hi+1)]


def project_planar(src, sw, sh, W, H, ncol, p):
    ax, ay, az = p['rotation']; scx, scy, rgh = (t/100 for t in p['scalexyz'])
    R = rotate(ax, ay, az); z0 = (ncol - 1)/2
    hx, hy = (W-1)/2, (H-1)/2
    D = zmax = 0
    if p['perspective']:
        zs = [R(a*scx, b*scy, z*rgh)[2] for a in (-hx, hx) for b in (-hy, hy) for z in (-z0, z0)]
        zmax = max(zs)
        D = int(p['perspective']/100*(zmax - min(zs)) + 1e-9)
    pcx = W//2 - 0.5 - hx; pcy = H//2 - 0.5 - hy
    shx = int(p['xyshift'][0]*W/100); shy = int(p['xyshift'][1]*H/100)
    wl = p['waterline']; tlo, thi = p['transparent']
    pts = {}
    for y in range(sh):
        for x in range(sw):
            c = src[y*sw+x]
            if c < wl: c = int(wl)
            tr = thi != 0 and tlo <= c <= thi   # transparent: kept as a vertex, never drawn
            u, v, w = R((x-hx)*scx, (y-hy)*scy, (c-z0)*rgh)
            if p['perspective']:
                den = D - zmax - w
                if den <= 0: continue          # also: D == 0 draws nothing
                f = D/den
                u = (u + shx - pcx)*f + pcx; v = (v - shy - pcy)*f + pcy   # shift before perspective
            else:
                u += shx; v -= shy
            X = int(W/2 + 0.5 + u); Y = int(H/2 + 0.5 + v)
            bx = by = None
            if p['filltype'] == 4:
                ub, vb, wb = R((x-hx)*scx, (y-hy)*scy, (0-z0)*rgh)
                if p['perspective']:
                    fb = D/(D - zmax - wb) if D - zmax - wb > 0 else 1e9
                    ub = (ub + shx - pcx)*fb + pcx; vb = (vb - shy - pcy)*fb + pcy
                else:
                    ub += shx; vb -= shy
                bx = min(max(int(W/2 + 0.5 + ub), 0), W-1)
                by = min(max(int(H/2 + 0.5 + vb), 0), H-1)
            pts[(x, y)] = (X, Y, c, bx, by, tr)
    return pts


def project_sphere(src, sw, sh, W, H, ncol, p):
    lon = p['longitude'] or tuple(p['rotation'][:2])
    lat = p['latitude'] or (p['rotation'][2], p['scalexyz'][0])
    rad = p['radius'] if p['radius'] is not None else p['scalexyz'][1]
    rgh = p['scalexyz'][2]/100
    tlo, thi = p['transparent']; wl = p['waterline']
    R0 = rad/100*H/2
    pts = {}
    for y in range(sh):
        lo = math.radians(lon[0] + (lon[1]-lon[0])*y/H)
        for x in range(sw):
            c = src[y*sw+x]
            if c < wl: c = int(wl)
            tr = thi != 0 and tlo <= c <= thi
            la = math.radians(lat[0] + (lat[1]-lat[0])*x/W)
            r = R0*(1 - rgh*(ncol-1-c)/800)        # hypothesis, see spec
            if math.cos(la)*math.sin(lo) <= 0: continue
            pts[(x, y)] = (int(W/2 + r*math.sin(la)), int(H/2 + r*math.cos(la)*math.cos(lo)), c, None, None, tr)
    return pts


def render(path, args):
    g = read_gif(path); sw, sh = g['w'], g['h']; src = g['pixels']
    ncol = len(g['gct'])//3
    W, H = canvas_size(sw, sh)
    p = parse_args(args)
    pts = (project_sphere if p['sphere'] else project_planar)(src, sw, sh, W, H, ncol, p)
    img = [None]*(W*H)          # None = background (black)
    def put(L, c):
        for x, y in L:
            if 0 <= x < W and 0 <= y < H: img[y*W+x] = c
    ft = p['filltype'] if not p['sphere'] else 0
    if ft == -1:
        ys = max(H//20, 1); xs = max((ys*W*3)//(4*H), 1)
        rows = list(range(0, sh, ys)); cols = list(range(0, sw, xs))
        if rows[-1] != sh-1: rows.append(sh-1)
        if cols[-1] != sw-1: cols.append(sw-1)
        for ri, y in enumerate(rows):
            for ci, x in enumerate(cols):
                q = pts.get((x, y))
                if q is None or q[5]: continue
                if ri == 0 and ci == 0: put([q[:2]], q[2])
                if ci > 0 and (cols[ci-1], y) in pts:
                    o = pts[(cols[ci-1], y)]; put(dda(o[0], o[1], q[0], q[1]), q[2])
                if ri > 0 and (x, rows[ri-1]) in pts:
                    o = pts[(x, rows[ri-1])]; put(dda(o[0], o[1], q[0], q[1]), q[2])
        return img, W, H
    for y in range(sh):
        for x in range(sw):
            q = pts.get((x, y))
            if q is None: continue
            X, Y, c, bx, by, tr = q
            if tr: continue
            if ft == 0:
                put([(X, Y)], c)
            elif ft == 1:
                o = pts.get((x-1, y))
                if o is None: put([(X, Y)], c)
                else: put(dda(o[0], o[1], X, Y), c)
            elif ft in (2, 3):
                if y == 0:
                    o = pts.get((x-1, y))
                    put([(X, Y)] if o is None else dda(o[0], o[1], X, Y), c)
                    continue
                up = pts.get((x, y-1)); ur = pts.get((x+1, y-1)); lf = pts.get((x-1, y))
                if up and ur: put(tri(q, up, ur), c)
                if up and lf: put(tri(q, lf, up), c)
            elif ft == 4:
                put(dda(X, Y, bx, by), c)
    return img, W, H


def compare(img, W, H, out):
    a = read_gif(out); A = a['pixels']
    if (a['w'], a['h']) != (W, H):
        return None
    pal = a['gct']
    # background is black: XFractint stores the lowest index whose colour is black (0 if none)
    blacks = [i for i in range(len(pal)//3) if pal[3*i:3*i+3] == b'\0\0\0']
    bg = blacks[0] if blacks else 0
    return 100*sum((bg if v is None else v) == A[i] for i, v in enumerate(img))/(W*H)


if __name__ == '__main__':
    cases, indir, outdir = sys.argv[1:4]
    for line in open(cases):
        t = line.split()
        if not t or t[0].startswith('#'): continue
        name, inp, args = t[0], t[1], t[2:]
        out = os.path.join(outdir, name + '.gif')
        if inp == '-' or not os.path.exists(out): continue
        img, W, H = render(os.path.join(indir, inp), args)
        r = compare(img, W, H, out)
        print('%-14s %s' % (name, 'size mismatch' if r is None else '%.3f %%' % r))
