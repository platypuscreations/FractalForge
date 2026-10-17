<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# FractalForge

FractalForge is a fast, modern fractal explorer written in Rust. It aims to give
long-time Fractint and XFractint users a familiar workflow on macOS, then Linux
and Windows.

It reads and writes Fractint-format PAR, MAP, FRM and GIF files, including the
parameters and resume data stored inside GIFs. That means a long render can be
saved partway through and finished later, in FractalForge or in Fractint.

FractalForge is an independent, clean-room implementation. It contains no
Fractint source code and is not affiliated with the Stone Soup Group.

## Status

Phase 0 (foundation). Nothing renders yet. Roadmap:

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
