# Stage 1: Cargo Chef preparation
FROM rust:1.97-slim-trixie AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

# Stage 2: Calculate dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies & Application (Cached)
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Install build tools, SSL, SQLite, GLib, GStreamer dev libraries, Python/pip, and OpenVINO
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    clang \
    pkg-config \
    wget \
    curl \
    gnupg \
    ca-certificates \
    python3 \
    python3-pip \
    libssl-dev \
    libsqlite3-dev \
    libglib2.0-dev \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    libgstreamer-plugins-bad1.0-dev \
    # && wget -qO- https://apt.repos.intel.com/intel-gpg-keys/GPG-PUB-KEY-INTEL-SW-PRODUCTS.PUB | gpg --dearmor -o /etc/apt/trusted.gpg.d/intel.gpg \
    # && echo "deb https://apt.repos.intel.com/openvino ubuntu24 main" > /etc/apt/sources.list.d/intel-openvino.list \
    # && apt-get update && apt-get install -y --no-install-recommends \
    # openvino-2026.3.0 \
    && rm -rf /var/lib/apt/lists/*

# Install pre-built ONNX Runtime with OpenVINO EP support via PyPI into /opt/ort
# RUN pip install --no-cache-dir --target /opt/ort onnxruntime-openvino

# Set OpenVINO library and pkg-config paths for build stage
# ENV OPENVINO_DIR="/opt/intel/openvino_2026.3.0"
# ENV LD_LIBRARY_PATH="/opt/ort/onnxruntime/capi:/opt/intel/openvino_2026.3.0/runtime/lib/intel64:${LD_LIBRARY_PATH}"
# ENV PKG_CONFIG_PATH="/opt/intel/openvino_2026.3.0/runtime/lib/intel64/pkgconfig:${PKG_CONFIG_PATH}"
# ENV ORT_DYLIB_PATH="/opt/ort/onnxruntime/capi/libonnxruntime.so"

# Cache dependencies
RUN cargo chef cook --release --recipe-path recipe.json

# Build actual application
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Stage 4: Runtime container
FROM debian:trixie-slim AS runtime

# Install GStreamer plugins, GLib, SQLite, OpenVINO repository & runtime packages
RUN apt-get update && apt-get install -y --no-install-recommends \
    wget \
    curl \
    gnupg \
    ca-certificates \
    # && wget -qO- https://apt.repos.intel.com/intel-gpg-keys/GPG-PUB-KEY-INTEL-SW-PRODUCTS.PUB | gpg --dearmor -o /etc/apt/trusted.gpg.d/intel.gpg \
    # && echo "deb https://apt.repos.intel.com/openvino ubuntu24 main" > /etc/apt/sources.list.d/intel-openvino.list \
    && apt-get update && apt-get install -y --no-install-recommends \
    libsqlite3-0 \
    libglib2.0-0 \
    libgstreamer1.0-0 \
    libgstreamer-plugins-base1.0-0 \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav \
    gstreamer1.0-tools \
    # openvino-2026.3.0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 1. Copy compiled binary
COPY --from=builder /app/target/release/alpr_backend /usr/local/bin/alpr_backend

# 2. Copy ONNX Runtime shared objects (including libonnxruntime_providers_openvino.so)
# COPY --from=builder /opt/ort/onnxruntime/capi/*.so* /app/

# 3. Copy models into the image
COPY models /app/models

# 4. Create persistent directories
RUN mkdir -p /app/data /app/snapshots

# Configure runtime environment paths for OpenVINO and ORT Dynamic Loading
# ENV OPENVINO_DIR="/opt/intel/openvino_2026.3.0"
# ENV LD_LIBRARY_PATH="/app:/opt/intel/openvino_2026.3.0/runtime/lib/intel64:${LD_LIBRARY_PATH}"
# ENV ORT_DYLIB_PATH="/app/libonnxruntime.so"

ENV DATABASE_URL="sqlite:///app/data/anpr.db?mode=rwc"
ENV SNAPSHOT_DIR="/app/snapshots"
ENV SERVER_ADDR="0.0.0.0:3000"
ENV PRIV_KEY_PATH="/app/data/keys/ed_private.pem"
ENV PUB_KEY_PATH="/app/data/keys/ed_public.pem"

EXPOSE 3000

CMD ["alpr_backend"]