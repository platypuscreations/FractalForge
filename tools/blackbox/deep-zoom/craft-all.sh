#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Makes the crafted deep-zoom files (E9) from the E7 base renders.
#   craft-all.sh BASEDIR OUTDIR
# BASEDIR holds base-v1.gif, base-d.gif, base-j.gif (cases-e7.txt).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
b="$1" o="$2"; mkdir -p "$o"
V2=-0.0000000000000012103333333333333333333333333,0.0000000000000014563333333333333333333333333,-1.000000000000001456,-0.999999999999999456,-0.0000000000000012103333333333333333333333333,-1.000000000000001456
V2F=-1.2103333333333333e-15,1.4563333333333333e-15,-1.000000000000001456,-0.999999999999999456
V2C=-0.0000000000000012103333333333333333333333333,0.0000000000000014563333333333333333333333333,-1.000000000000001456,-0.999999999999999456
c() { python3 -I "$here/craft.py" "$@"; }
B="$b/base-v1.gif" D="$b/base-d.gif" J="$b/base-j.gif"
c "$B" "$o/k00-copy.gif"
c "$B" "$o/k01-both.gif"   slots=0-5:$V2 slots=16-21:$V2
c "$B" "$o/k02-first.gif"  slots=0-5:$V2
c "$B" "$o/k03-second.gif" slots=16-21:$V2
c "$B" "$o/k04-f64.gif"    f64corners=$V2F
c "$B" "$o/k05-drop.gif"   drop005
c "$B" "$o/k06-flag0.gif"  slots=0-5:$V2 slots=16-21:$V2 set412=0
c "$B" "$o/k07-N20.gif"    slots=0-5:$V2 slots=16-21:$V2 N=20
c "$B" "$o/k08-N8.gif"     slots=0-5:$V2 slots=16-21:$V2 N=8
c "$B" "$o/k09-f64zero.gif" slots=0-5:$V2 slots=16-21:$V2 f64corners=0,0,0,0
c "$D" "$o/k10-fromD.gif"  add005=12:$V2C
c "$D" "$o/k11-fromDf.gif" add005=12:$V2C f64corners=$V2F
c "$J" "$o/k12-jslot.gif"  slot=6:0 slot=7:-1
c "$J" "$o/k13-jf64.gif"   f64=44:0 f64=52:-1
c "$B" "$o/k15-len.gif"    slots=0-5:$V2 slots=16-21:$V2 set414=16
c "$B" "$o/k16-shallow.gif" slots=0-5:-2.5,1.5,-1.5,1.5,-2.5,-1.5 slots=16-21:-2.5,1.5,-1.5,1.5,-2.5,-1.5
c "$D" "$o/k17-tol474.gif" f64=474:0
c "$D" "$o/k18-tol482.gif" f64=482:0
c "$D" "$o/k19-fromDN16.gif" add005=16:$V2C f64corners=$V2F
c "$D" "$o/k20-flagonly.gif" set412=1 set414=12 f64corners=$V2F
c "$D" "$o/k21-fromDN8.gif" add005=8:$V2C f64corners=$V2F
c "$D" "$o/k22-n0.gif"     add005=12:$V2C set412=0 set414=0
c "$D" "$o/k23-flag0D.gif" add005=12:$V2C set412=0
