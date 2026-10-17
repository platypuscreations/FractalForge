# SPDX-License-Identifier: GPL-3.0-or-later
#
# Black-box XFractint environment. Installs Debian's prebuilt non-free binary
# package only: no Fractint source code is fetched, built or inspected.
# Used solely to produce reference files and check interoperability.
# See docs/CLEANROOM.md.

FROM --platform=linux/amd64 debian:trixie-slim

RUN sed -i 's/^Components: main$/Components: main non-free/' /etc/apt/sources.list.d/debian.sources \
 && apt-get update \
 && apt-get install -y --no-install-recommends xfractint xvfb xauth xdotool xfonts-base \
 && rm -rf /var/lib/apt/lists/*

RUN useradd -m bb
USER bb
WORKDIR /work
COPY --chown=bb:bb run-batch.sh /usr/local/bin/run-batch
COPY --chown=bb:bb run-interrupt.sh /usr/local/bin/run-interrupt
COPY --chown=bb:bb run-load.sh /usr/local/bin/run-load
ENTRYPOINT ["run-batch"]
