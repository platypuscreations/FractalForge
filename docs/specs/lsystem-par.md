<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# L-system images in `.par` files

Status: **black-box derived**, like [ifs.md](ifs.md) §10, which describes the
same experiments for IFS. Debian `xfractint 20.4.10-5`. No Fractint source,
documentation or bundled `.l` file was used (see [CLEANROOM.md](../CLEANROOM.md)).
The L-system type itself is described elsewhere. This file covers only how
`.par` entries refer to L-systems.

Evidence: `corpus/xfractint/ifs/lpar-*` (command lines in `cases.txt` there,
section 10). Inputs: `tools/blackbox/ifs/par/` (`ip.par`, `lf.l`, `lref.l`).
Batch runs `@ip.par/ENTRY`, in a directory that held only the named files.
Each result is compared pixel for pixel with the same body rendered from an
ordinary `.l` file (`lpar-ref-*`).

## 1. Selecting the entry (confirmed)

- Keys inside the entry: `type=lsystem lfile=FILE lname=NAME`.
- **`lfile=` inside an entry is not looked up in the working directory**,
  whether relative or absolute: `lfile=lf.l lname=lk` (`lpar-rel`) and
  `lfile=/work/lf.l` (`lpar-abs`) give `'lk' file entry item not found`, and
  so does an entry with no `lfile=` (`lpar-none`). This matches `ifsfile=`
  and `formulafile=` ([par.md](par.md) §5.3). **Hypothesis:** only
  XFractint's own directories are searched (not tested).
- `lfile=` on the command line before `@file/entry` works (`lpar-none-cmd`,
  `lpar-rt-wls-cmd`).

## 2. Embedded L-systems (confirmed)

- An entry named **`lsys:NAME`** in the same `.par` is used for
  `lname=NAME` (`lpar-enoname`, `lpar-enoname-only`, `lpar-eself`). It is
  pixel-identical to the same body in a `.l` file (`lpar-ref-eq`).
- Other prefixes are not recognised: `lsystem:`, `l:` and `lsy:` all give
  `not found` (`lpar-alt1`, `-alt2`, `-alt3`). The prefix is
  case-insensitive: `LSYS:lu` works (`lpar-upper`).
- `lname=lsys:lq` in the entry does not find it (`lpar-eprefix`). A plain
  entry `lr` of the `.par` is not used (`lpar-plain`), even with
  `lfile=ip.par` in the entry (`lpar-eselfplain`).
- `lfile=lf.l` on the command line wins over the embedded entry
  (`lpar-enoname-cmd` draws `lf.l`'s `lq`, = `lpar-ref-lq`).
- A `.par` given as `lfile=` on the command line is read as a `.l` file with
  the prefix in the names: `lname=lsys:lq` works (`lpar-cf-pfx`),
  `lname=lq` doesn't (`lpar-cf-nopfx`), and the plain entry `lr` does
  (`lpar-cf-plain`).
- `fractint003` stores the name as typed in `lname=`, without the prefix
  when the entry was embedded.

## 3. What XFractint writes (confirmed)

From the `b` screen (native build, `lpar-w-ls.out.par`):

```
wls                { ; our comment one
  reset=2004 type=lsystem lfile=lf.l lname=lk center-mag=0/0/1/1.3333
  params=2 float=y inside=0 viewwindows=2/0.75/yes/0/0
  }
```

- `lfile=`, `lname=` and `params=2` (the default order) are written, and the
  L-system body is **not** embedded. For an embedded source XFractint writes
  `lfile=ip.par lname=lq`, the `.par` it came from and the name without the
  prefix (`lpar-w-lsemb`).
- Loading the written file back with `lf.l` next to it fails
  (`lpar-rt-wls`: `'lk' file entry item not found`). With `lfile=lf.l` on the
  command line, it reproduces the image (`lpar-rt-wls-cmd` = `lpar-ref-lk`).
  The native build's image equals the amd64 one (`lpar-w-ls.gif` =
  `lpar-ref-lk.gif`).

## 4. Recommendation for FractalForge

Write `lfile=` and `lname=` like XFractint, and also embed the body as
`lsys:NAME { … }` in the same `.par`. When reading, try `lsys:NAME` in the
same file first, then the `lfile=` next to the `.par`, then our own
directories.

## 5. Unknowns

- Which directories XFractint does search for `lfile=` inside an entry.
- Whether the embedded entry may also come before the image entry (it came
  after it in every case here; `frm:` works both ways).
