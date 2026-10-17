# SPDX-License-Identifier: GPL-3.0-or-later
# model.py INT REF : reader model for fractint002 (our own black-box derived rules).
# Predicts every pixel of an interrupted save from the completed render REF of the same
# parameters and pass mode, then counts mismatches. Guessing: N passes from image width.
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gif

SYM = {0: 'none', 16: 'none', 17: 'x', 51: 'origin'}
FORCE = os.environ.get('FORCE_SYM')  # e.g. FORCE_SYM=xy to test another mirror kind

def guess_passes(w):
    # observed: 200, 267 -> 2; 400 -> 3; 800 -> 4 (hypothesis for other widths)
    n, s = 0, float(w)
    while s >= 100: s /= 2; n += 1
    return n

def items(f):
    count = f[1]
    return [dict(zip(('left', 'right', 'top', 'bot', 'row', 'sym', 'pass', 'col'), f[2+8*i:10+8*i]))
            for i in range(count)]

def predict(ref, w, h, mode, f):
    pred = bytearray(ref)  # outside every pending item: final
    state = ['final'] * (w * h)
    for it in items(f):
        L, Rt, T, B = it['left'], it['right'], it['top'], it['bot']
        R, C, p = it['row'], it['col'], it['pass']
        sym = FORCE or SYM.get(it['sym'], '?')
        last = T + (B - T) // 2 if sym != 'none' else B  # last directly computed row
        def blk(r, c, k):
            r0 = r - r % k; c0 = c - c % k   # blocks sit on the image-wide grid
            return ref[r0*w+c0]
        def before(r, c, k=1):  # is block containing (r,c) before the current point?
            r0 = r - r % k; c0 = c - c % k
            return r0 < R or (r0 == R and c0 < C)
        lastc = L + (Rt - L) // 2 if sym == 'xy' else Rt
        for r in range(T, last + 1):
            for c in range(L, lastc + 1):
                i = r*w+c
                if mode == '1':
                    v, s = (ref[i], 'final') if before(r, c) else (0, 'empty')
                elif mode == '2':
                    if p == 0:
                        v, s = (blk(r, c, 2), 'prov') if before(r, c, 2) else (0, 'empty')
                    else:
                        v, s = (ref[i], 'final') if before(r, c) else (blk(r, c, 2), 'prov')
                elif mode == 'g':
                    N = guess_passes(w); bp = 2 ** (N - 1 - p)
                    if r < R:
                        v, s = blk(r, c, bp), ('final' if bp == 1 else 'prov')
                    elif r < R + 2 * bp:
                        v, s = None, 'band'
                    elif p > 0:
                        v, s = blk(r, c, 2 * bp), 'prov'
                    elif r == R + 2 * bp and c % (2 * bp) == 0:
                        v, s = ref[i], 'lookahead'   # exact if already computed, else 0
                    else:
                        v, s = 0, 'empty'
                else:
                    v, s = None, 'unknown'
                pred[i] = 0 if v is None else v
                state[i] = s
        if sym == 'xy':  # columns mirrored inside each computed row first
            for r in range(T, last + 1):
                for c in range(lastc + 1, Rt + 1):
                    pred[r*w+c] = pred[r*w+L+Rt-c]; state[r*w+c] = state[r*w+L+Rt-c]
        if sym != 'none':
            for r in range(last + 1, B + 1):
                for c in range(L, Rt + 1):
                    mr = T + B - r
                    mc = L + Rt - c if sym == 'origin' else c
                    pred[r*w+c] = pred[mr*w+mc]; state[r*w+c] = state[mr*w+mc]
    return pred, state

if __name__ == '__main__':
    a = gif.read_gif(sys.argv[1]); b = gif.read_gif(sys.argv[2])
    w, h = a['w'], a['h']
    mode = chr(a['exts']['fractint001'][200])
    e2 = a['exts'].get('fractint002')
    if not e2:
        d = sum(x != y for x, y in zip(a['pixels'], b['pixels']))
        print(os.path.basename(sys.argv[1]), 'complete; diff vs ref', d); sys.exit()
    f = gif.i32s(e2)
    pred, state = predict(b['pixels'], w, h, mode, f)
    import collections
    bad = collections.Counter(); tot = collections.Counter()
    for i in range(w*h):
        tot[state[i]] += 1
        if state[i] == 'band': continue
        if state[i] == 'lookahead':
            if a['pixels'][i] not in (0, pred[i]): bad['lookahead'] += 1
            continue
        if pred[i] != a['pixels'][i]: bad[state[i]] += 1
    # in pass-0 guessing, look-ahead grid points below the band may already be exact
    print(os.path.basename(sys.argv[1]), 'mode', mode, 'items', [list(x.values()) for x in items(f)],
          'states', dict(tot), 'mismatch', dict(bad))
