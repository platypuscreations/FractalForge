#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# queue.sh JOBFILE P : run job lines P at a time.
#  B name VIEW args...            batch render
#  I name delay VIEW args...      interrupted render
#  L name in.gif d1 d2 args...    command-line load (filename=) + Enter (recalculates!)
#  R name in.gif D args...        start XFractint, load in.gif with the r command (resumes),
#                                 press s after D seconds
cd "$(dirname "$0")"
. ./views.sh
run() {
  set -- $1
  kind=$1; shift
  case $kind in
    B) n=$1; v=$2; shift 2; ./batch.sh $n ${!v} "$@";;
    I) n=$1; dl=$2; v=$3; shift 3; ./intr.sh $n $dl ${!v} "$@";;
    L) n=$1; in=$2; d1=$3; d2=$4; shift 4; ./load.sh $n $in $d1 $d2 "$@";;
    R) n=$1; in=$2; d=$3; shift 3
       NOFILEARG=1 ./keys.sh $n $in "s10 kr s3 tin.gif s1 kReturn s$d ks" type=mandel maxiter=10 "$@";;
  esac
}
export -f run
export MS MN J DG MSG MNG JG MO JB BX BOF ORL MSN COMMON
grep -v '^#' "$1" | grep . | xargs -P "$2" -I{} bash -c 'run "$@"' _ {}
