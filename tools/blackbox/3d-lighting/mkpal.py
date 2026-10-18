# SPDX-License-Identifier: GPL-3.0-or-later
# Writes flat 400x300 GIFs whose only colour index IDX has a chosen RGB
# (all other palette entries from uniq.map), to see how stereo maps colours.
# Usage: python3 -I mkpal.py OUT.gif IDX R G B
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gif import write_gif
out, idx, r, g, b = sys.argv[1], *map(int, sys.argv[2:6])
here = os.path.dirname(os.path.abspath(__file__))
vals = []
for line in open(os.path.join(here, 'inputs', 'uniq.map')):
    vals += [int(v) for v in line.split()[:3]]
vals[3*idx:3*idx+3] = [r, g, b]
write_gif(out, 400, 300, bytes(vals), bytes([idx]) * 120000, [])
