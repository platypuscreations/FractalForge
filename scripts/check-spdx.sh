#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Fails if any tracked source or config file lacks an SPDX license header
# in its first five lines.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

missing=0
while IFS= read -r f; do
  if ! head -n 5 "$f" | grep -q 'SPDX-License-Identifier: GPL-3.0-or-later'; then
    echo "missing SPDX header: $f"
    missing=1
  fi
done < <(git ls-files --cached --others --exclude-standard \
  '*.rs' '*.toml' '*.yml' '*.yaml' '*.sh' '*.md' '*.wgsl' '**/Dockerfile' 'Dockerfile')

if [ "$missing" -ne 0 ]; then
  echo "Add '// SPDX-License-Identifier: GPL-3.0-or-later' (or the '#'/'<!-- -->' form)."
  exit 1
fi
echo "SPDX headers OK"
