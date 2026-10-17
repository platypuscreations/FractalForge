# SPDX-License-Identifier: GPL-3.0-or-later
#
# Variant of tools/blackbox for the .par/.map spec work: same Debian prebuilt
# non-free xfractint package (no source fetched or built), but for the host's
# native architecture, plus tools to take screenshots (xwd, ImageMagick) so
# interactive screens can be driven with xdotool and inspected.
FROM debian:trixie-slim
RUN sed -i 's/^Components: main$/Components: main non-free/' /etc/apt/sources.list.d/debian.sources \
 && apt-get update \
 && apt-get install -y --no-install-recommends xfractint xvfb xauth xdotool xfonts-base x11-apps imagemagick \
 && rm -rf /var/lib/apt/lists/*
RUN useradd -m bb
USER bb
WORKDIR /work
COPY --chown=bb:bb drive.sh /usr/local/bin/drive
ENTRYPOINT ["drive"]
