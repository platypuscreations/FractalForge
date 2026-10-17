<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Black-box XFractint

A Linux container that runs XFractint under a virtual X display, so we can see
what it does without ever looking at its code. It installs Debian's prebuilt
non-free `xfractint` package. Read [docs/CLEANROOM.md](../../docs/CLEANROOM.md)
first.

```sh
tools/blackbox/gen-corpus.sh                       # all cases -> tools/blackbox/out/
docker run --rm -v "$PWD/out:/work" fractalforge-blackbox type=mandel savename=ff.gif
docker run --rm -v "$PWD/out:/work" --entrypoint run-interrupt \
  fractalforge-blackbox 10 type=mandel maxiter=20000 savename=ff.gif
```

- `run-batch` (the default entrypoint) adds `batch=yes` and passes the
  remaining arguments to XFractint.
- `run-interrupt DELAY ...` starts an interactive render, presses **s** after
  DELAY seconds and keeps the partial save.

## Quirks

- The image is linux/amd64. On Apple Silicon it runs under qemu, which is slow.
  It also breaks `xvfb-run`, so the scripts start Xvfb directly.
- XFractint needs the `9x15` X core font (from `xfonts-base`).
- Save names are cut to DOS 8.3 form and auto-numbered on collisions. Use a
  short name such as `ff.gif` in an empty directory.
