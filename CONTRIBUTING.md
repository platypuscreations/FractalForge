<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Contributing to FractalForge

Thanks for helping. Before you write anything, read
[docs/CLEANROOM.md](docs/CLEANROOM.md). Contributions that break the clean-room
rules can't be accepted, however good they are.

## Sign your commits (DCO)

We use the [Developer Certificate of Origin](https://developercertificate.org/)
instead of a CLA. Add a `Signed-off-by` line to every commit:

```sh
git commit -s
```

By signing off you certify the DCO. For this project that also means you
followed the clean-room rules: you did not read, copy or translate Fractint,
XFractint or Iterated Dynamics source code to write your change.

## Licensing

- All contributions are licensed GPL-3.0-or-later.
- Every source, config and doc file starts with an SPDX header:
  - Rust, WGSL: `// SPDX-License-Identifier: GPL-3.0-or-later`
  - TOML, YAML, shell, Dockerfile: `# SPDX-License-Identifier: GPL-3.0-or-later`
  - Markdown: `<!-- SPDX-License-Identifier: GPL-3.0-or-later -->`
  
  `scripts/check-spdx.sh` checks this, and CI runs it.
- New dependencies must use a GPL-3.0-compatible license. `cargo deny check`
  enforces the allowlist in `deny.toml`. Don't widen it without discussion.

## Before opening a pull request

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
scripts/check-spdx.sh
cargo deny check        # brew install cargo-deny, or cargo install --locked cargo-deny
```
