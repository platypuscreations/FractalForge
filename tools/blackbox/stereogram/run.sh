#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
# Host wrapper: run.sh WORKDIR "ACTIONS" xfractint-args...
# Copies run-keys-fb.sh into WORKDIR and runs it in the black-box container.
set -u
here="$(cd "$(dirname "$0")" && pwd)"
d="$1"; shift
cp "$here/run-keys-fb.sh" "$d/"
docker run --rm -v "$d:/work" --entrypoint bash fractalforge-blackbox /work/run-keys-fb.sh "$@" 2>&1 | grep -v 'requested image.s platform'
