<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# FractalForge

FractalForge is a fast, modern fractal explorer written in Rust. It aims to give
long-time Fractint and XFractint users a familiar workflow on macOS, then Linux
and Windows.

It reads and writes Fractint-format PAR, MAP, FRM, IFS, L and GIF files, including the
parameters and resume data stored inside GIFs. That means a long render can be
saved partway through and finished later, in FractalForge or in Fractint.

FractalForge is an independent, clean-room implementation. It contains no
Fractint source code and is not affiliated with the Stone Soup Group.

## Status

**v0.1** (macOS), with Phase 3 (formulas) on `main` and Phase 4 in progress.
FractalForge has a
desktop app and a command-line renderer.

- 18 escape-time types (mandel, julia, lambda, mandellambda, mandel4, julia4,
  manowar, manowarj, barnsleym1, barnsleyj1, spider, magnet1m, magnet1j,
  newton, newtbasin, phoenix, mandphoenix, popcornjul), rendered with the
  same pixels as Fractint-format renderers.
- L-systems (`.l`) and iterated function systems (`.ifs`), with
  XFractint's pixels, entries embedded in PAR files and stored in GIFs.
- Orbit-plotting types: henon, hopalong, martin, gingerbreadman, chip,
  quadruptwo, threeply, icons, latoocarfian, kamtorus, popcorn, dynamic,
  mandelcloud and julia_inverse, and the strange attractors lorenz,
  lorenz3d, rossler3d, pickover, lorenz3d1, lorenz3d3, lorenz3d4 and
  kamtorus3d, with 3D rotation, shift and perspective.
- An orbit window (press O) that draws the orbit of the point under the
  pointer, and a Mandelbrot/Julia toggle (Space) for the types that pair up.
- Inside/outside colouring modes, decomposition and biomorph (mandel and
  julia), colour cycling and a palette editor.
- Stop a render, save it as a GIF and resume it later with identical
  results. Partial renders also move both ways between FractalForge and
  XFractint.
- Reads and writes Fractint-format PAR, MAP and GIF files.
- Fractint-format formulas (`.frm`): the formula language with the same
  pixels as XFractint, including symmetry qualifiers, `fn1`–`fn4` and
  formulas embedded in PAR files. GIFs FractalForge saves carry the formula
  text, so they open without the `.frm` file.

From the command line:

```sh
cargo run --release -p fractalforge-cli -- render -o big.gif --size 3840x2160 --maxiter 5000 --stop-after 30
cargo run --release -p fractalforge-cli -- render big.gif --resume -o big-done.gif
cargo run --release -p fractalforge-cli -- render saved-by-fractint.gif -o again.gif
cargo run --release -p fractalforge-cli -- render my.par --entry myentry -o out.png
cargo run --release -p fractalforge-cli -- render --formula my.frm --formula-name myformula -o f.gif
cargo run --release -p fractalforge-cli -- render --lsystem plants.l --lsystem-name fern --params 5 -o fern.gif
cargo run --release -p fractalforge-cli -- render --ifs ferns.ifs -o fern.png
cargo run --release -p fractalforge-cli -- render --type lorenz3d -o lorenz.png
cargo run --release -p fractalforge-cli -- entries my.frm
```

Ctrl-C during a render also saves a resumable GIF.

Not yet: other Fractint types (such as bifurcation, lyapunov, plasma and
icons3d); rotated views; Fractint's `rand` sequence in formulas; resuming
unfinished orbit-plotting images (they are recalculated, as Fractint does
for most of them); screen-reader support; notarized macOS builds.

A few chaotic orbit types (martin, chip, quadruptwo, threeply, latoocarfian,
pickover and the long lorenz3d1/3/4 orbits) draw the same attractors as
XFractint but not identical pixels: their orbits depend on the last bit of
the C library's sine and cosine.

Roadmap:

| Phase | Scope |
|-------|-------|
| 0 | Workspace, CI, license checks, clean-room process, black-box reference corpus |
| 1 | Mandelbrot/Julia engine, CLI, GIF save/resume (native and Fractint-format blocks) |
| 2 | GUI MVP and **v0.1**: zoom, color cycling, PAR/MAP files, about a dozen escape-time types, macOS app |
| 3 | Formula (`.frm`) parser |
| 4 | L-systems, IFS, strange attractors, orbits |
| 5 | Deep zoom (perturbation), 3D, stereograms |
| 6 | Linux and Windows releases |

## Layout

```
crates/core      complex math, fractal types, parameters
crates/render    tile-based checkpointable renderer
crates/formats   PAR, MAP, PNG and GIF89a files
apps/cli         batch renderer
apps/gui         desktop app (eframe)
tools/blackbox   container that runs XFractint as a black box to make reference files
corpus/          reference files saved by XFractint, used for interop tests
docs/            clean-room rules and our own file-format specs
```

## Installing on macOS

Download `FractalForge-<version>-macos-universal.dmg` and drag FractalForge
to Applications. It runs on Apple silicon and Intel Macs with macOS 11 or
later.

The app isn't notarized by Apple yet, so the first time you open it macOS
says it can't check it for malicious software. To open it anyway:

1. Try to open FractalForge once, and close the warning.
2. Open **System Settings → Privacy & Security**, scroll down, and click
   **Open Anyway** next to the FractalForge message.
3. Confirm with your password. After that it opens normally.

Or, in Terminal: `xattr -dr com.apple.quarantine /Applications/FractalForge.app`

Building from source (below) avoids the warning entirely.

## Building

```sh
cargo build --workspace
cargo test --workspace
```

## Contributing

Read [docs/CLEANROOM.md](docs/CLEANROOM.md) and [CONTRIBUTING.md](CONTRIBUTING.md)
first. The clean-room rules are not optional.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
