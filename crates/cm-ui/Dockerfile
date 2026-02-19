# Tauri build environment for Cabinet Maker UI
#
# Build:   docker build -t cabinet-maker-ui .
# Dev:     docker run -it --rm -v $(pwd)/../..:/workspace -w /workspace/crates/cm-ui cabinet-maker-ui cargo tauri dev
# Build:   docker run -it --rm -v $(pwd)/../..:/workspace -w /workspace/crates/cm-ui cabinet-maker-ui cargo tauri build
#
# For GUI display, add: -e DISPLAY=$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix

FROM rust:1.88-bookworm

# Tauri system dependencies (Debian/Ubuntu)
RUN apt-get update && apt-get install -y --no-install-recommends \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libsoup-3.0-dev \
    libjavascriptcoregtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    wget \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Node.js (for SvelteKit frontend)
RUN wget -qO- https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

# Install Tauri CLI
RUN cargo install tauri-cli --version "^2"

WORKDIR /workspace

# Pre-install frontend deps if package-lock exists
COPY frontend/package.json frontend/package-lock.json* /tmp/frontend/
RUN cd /tmp/frontend && npm install --prefer-offline 2>/dev/null || true

CMD ["bash"]
