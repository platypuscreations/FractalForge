# SPDX-License-Identifier: GPL-3.0-or-later
"""Runs orbit.pl inside the black-box container (glibc libm) and returns the points."""
import os, subprocess, tempfile, shutil
HERE = os.path.dirname(os.path.abspath(__file__))
_cache = {}
def orbit(typ, count, start, params):
    key = (typ, count, tuple(start), tuple(params))
    if key in _cache:
        return _cache[key]
    work = tempfile.mkdtemp(prefix='orbit-', dir=os.environ.get('ORBIT_TMP', None))
    os.chmod(work, 0o777)
    shutil.copy(os.path.join(HERE, 'orbit.pl'), work)
    args = ['docker', 'run', '--rm', '-v', work + ':/w', '--entrypoint', 'perl',
            'fractalforge-blackbox', '/w/orbit.pl', typ, str(count)] + \
           ['%.17g' % v for v in list(start) + list(params)]
    out = subprocess.run(args, capture_output=True, text=True, check=True).stdout
    pts = [tuple(float(t) for t in line.split()) for line in out.splitlines()]
    os.remove(os.path.join(work, 'orbit.pl')); os.rmdir(work)
    _cache[key] = pts
    return pts
