#!/usr/bin/env bash
# ==============================================================================
# LEAN 4 Game - Linux Desktop Launcher
# ==============================================================================
# Resolves common WebKitGTK issues on Linux (Ubuntu 24.04, Debian, Arch, Fedora)
# ==============================================================================

# 1. Disable DMA-BUF renderer (fixes blank/white window on NVIDIA, Mesa, and Wayland)
export WEBKIT_DISABLE_DMABUF_RENDERER=1

# 2. Disable Bubblewrap sandbox (fixes WebKitWebProcess crash on Ubuntu 24.04 / AppArmor)
export WEBKIT_FORCE_SANDBOX=0

# 3. Pass through any arguments (e.g. --game real-analysis, --software-render, etc.)
exec cargo run "$@"
