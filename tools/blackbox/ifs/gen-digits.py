# SPDX-License-Identifier: GPL-3.0-or-later
# Writes an .ifs file of "digit" entries dK: K transforms x' = x/K + i/K,
# y' = y + 1/512, so x holds the recent choices as base-K digits and y counts
# points. Each probability is 1/K (PROB=one writes 1 instead).
# Usage: python3 -I gen-digits.py OUT.ifs [K ...]. Own test input.
import sys
ks = [int(a) for a in sys.argv[2:]] or [2, 3, 4, 10, 16, 32, 33]
u = '0.001953125'
with open(sys.argv[1], 'w') as f:
    f.write('; SPDX-License-Identifier: GPL-3.0-or-later\n')
    f.write('; Black-box test entries for XFractint\'s IFS type (FractalForge, own file).\n')
    for k in ks:
        f.write('d%d {\n' % k)
        for i in range(k):
            f.write('  %.10f 0 0 1 %.10f %s %.8f\n' % (1.0/k, i/float(k), u, 1.0/k))
        f.write('}\n')
