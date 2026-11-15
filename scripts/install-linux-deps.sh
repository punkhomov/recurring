#!/usr/bin/env bash
set -euo pipefail

# Keep this list in sync with .github/workflows/ci.yml and the README.
# GPUI on Linux draws through Vulkan and needs X11/Wayland + fontconfig headers.

SUDO=""
if [ "$(id -u)" -ne 0 ]; then
  SUDO="sudo"
fi

$SUDO apt-get update -qq
$SUDO DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
  build-essential \
  pkg-config \
  cmake \
  libxkbcommon-dev \
  libxkbcommon-x11-dev \
  libwayland-dev \
  wayland-protocols \
  libxcb1-dev \
  libxcb-randr0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxcb-xkb-dev \
  libxcb-render0-dev \
  libxcb-shm0-dev \
  libxcb-sync-dev \
  libx11-dev \
  libfontconfig1-dev \
  libfreetype6-dev \
  libvulkan-dev \
  libssl-dev \
  libzstd-dev \
  fonts-dejavu-core

if [ "${1:-}" = "--screenshot" ]; then
  $SUDO DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    xvfb \
    x11-utils \
    xdotool \
    openbox \
    imagemagick \
    mesa-vulkan-drivers
fi
