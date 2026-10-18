# SPDX-License-Identifier: GPL-3.0-or-later
# Predicts XFractint stereo=1 / stereo=2 output from the plain (stereo=0)
# render of the same top-view scene, per docs/specs/3d-lighting.md, and
# reports the pixel match.  Own code, black-box derived.
# Usage: python3 -I stereo_check.py MONO.gif STEREO.gif MODE SHIFT [CROP]
#   SHIFT = horizontal displacement of eye A in pixels (eye B gets -SHIFT)
#   CROP  = crop=a/b/c/d percentages (default 4/0/0/4)
# Colour 0 in MONO is taken as background (a limitation: a surface of
# colour 0 cannot be told apart from background in the mono image).
import sys, os, collections
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gif import read_gif

def canon(v):  # saver artefact with the default palette (types.md 1.3)
    return 0 if v == 16 or v >= 248 else 15 if v == 31 else v

def eye_value(c):
    return 128 - (c + 1) // 2

def main():
    mono, st, mode, shift = read_gif(sys.argv[1]), read_gif(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
    crop = [int(v) for v in (sys.argv[5] if len(sys.argv) > 5 else '4/0/0/4').split('/')]
    W, H, m = mono['w'], mono['h'], mono['pixels']
    a_ok = lambda x: x >= 1 and W * crop[0] / 100 < x < W - W * crop[1] / 100
    b_ok = lambda x: x >= 1 and W * crop[2] / 100 < x < W - W * crop[3] / 100
    out = bytearray(W * H)
    for y in range(H):
        for x in range(W):
            xa, xb = x - shift, x + shift          # source columns for eye A / eye B
            ca = m[y * W + xa] if 0 <= xa < W and a_ok(x) else 0
            cb = m[y * W + xb] if 0 <= xb < W and b_ok(x) else 0
            if mode == 1:
                if (x + y) % 2 == 0:
                    v = eye_value(ca) if ca else 0
                else:
                    v = min(255, eye_value(cb) + 128) if cb else 0
            else:
                la = eye_value(ca) // 9 + 1 if ca else 0
                lb = eye_value(cb) // 9 + 1 if cb else 0
                v = (lb << 4) + la
            out[y * W + x] = canon(v)
    sp = st['pixels']
    d = collections.Counter((sp[k], out[k]) for k in range(W * H) if sp[k] != out[k])
    ok = W * H - sum(d.values())
    print('match %d/%d (%.3f%%)' % (ok, W * H, 100.0 * ok / (W * H)), 'mismatch (got,predicted):', d.most_common(6))

main()
