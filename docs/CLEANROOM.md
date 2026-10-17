<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Clean-room rules

FractalForge is GPL-3.0-or-later. Fractint's license does not allow modified
source to be distributed or the program to be used commercially, so it is not
GPL-compatible. Debian ships XFractint in non-free for this reason. Any code
derived from Fractint, XFractint or Iterated Dynamics (a fork whose relicensing
is not clean) would make FractalForge impossible to distribute.

So we reimplement from scratch and stay compatible with the file formats and the
familiar workflow.

## Never

- Read, search, paste, summarize or translate Fractint, XFractint or Iterated
  Dynamics **source code**. That includes headers, structs, build files and
  diffs. It doesn't matter whether you got it from a tarball, a website, a code
  search or an AI assistant's memory.
- Ask an AI assistant to "port", "translate" or "do it the way Fractint's
  `foo()` does". Prompt from our specs and the math instead.
- Copy wording from Fractint's help text or documentation. It is copyrighted
  prose. You may use it to learn what the program *does*, then describe that
  in your own words.
- Bundle Fractint's `.map`, `.par`, `.frm`, `.l` or `.ifs` collections,
  including the ones Debian's xfractint package installs. Their copyrights vary.
  We ship palettes and examples we make ourselves. Users can load their own.
- Use the name "Fractint" for the product. Say "reads and writes
  Fractint-format files".

## Allowed inputs

- Published mathematics: papers, textbooks, and general descriptions of
  algorithms such as escape-time iteration, solid guessing and boundary tracing.
- **Black-box behavior.** Run the original program (see `tools/blackbox`), give
  it inputs, and observe its screens, output files and messages.
- File formats described **in our own words** in `docs/specs/*.md`.

## File formats

- Learn the layout of Fractint's GIF extension blocks **only** by analyzing
  bytes of files XFractint saves (`corpus/xfractint`). Vary one parameter at a
  time and diff the results.
- Write what you learn in `docs/specs/gif-fractint-blocks.md`. Implementers
  work from that spec, never from files or notes outside the repo.
- Ideally the person who writes a format spec is not the one who implements it.
  When that isn't possible, finish and commit the spec before you start the code.
- Reverse engineering a file format for interoperability is generally
  considered lawful. This is not legal advice.

## The black-box container

`tools/blackbox` installs Debian's **prebuilt** xfractint binary. It never
downloads or builds Fractint source. The package also installs text files under
`/usr/share/xfractint`, such as the help file and the source-license notice.
Don't open them for implementation work, and don't copy them out.

## If you slip

If you have read Fractint source code, say so in your pull request. Don't work
on the parts of FractalForge it covers. Someone else will take those parts.
