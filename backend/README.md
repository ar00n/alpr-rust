# 🦀 ALPR Backend (Rust / Axum)

The backend service handles real-time video processing, license plate detection, optical character recognition (OCR), database storage, and REST API routing.

---

## 🛠 Features

* **High-Speed Inference:** Powered by `ort` (ONNX Runtime) utilizing OpenVINO hardware acceleration.
* **Axum Web Server:** Asynchronous, low-overhead HTTP API.
* **Auto-Generated OpenAPI/Swagger Docs:** Served dynamically via `utoipa` and `utoipa-swagger-ui`.
* **SQLx Database Integration:** Async SQLite database access with compile-time checked queries and auto-migrations.
* **Video Stream Processing:** GStreamer integration for RTSP streams and video files.

---

## 🧠 Model Configuration

Ensure the following files are present in `backend/models/`:

```text
backend/models/
├── number-plate-yolo26n.onnx
├── PP-OCRv6_small_rec_onnx.onnx
└── ppocrv6_dict.txt
```

---

## 📋 Environment Variables

The backend accepts the following environment variables (defaults match Docker runtime):

| Variable | Default Value | Description |
|---|---|---|
| `DATABASE_URL` | `sqlite:///app/data/anpr.db?mode=rwc` | SQLite connection string |
| `SNAPSHOT_DIR` | `/app/snapshots` | Directory to save captured plate snapshots |
| `SERVER_ADDR` | `0.0.0.0:3000` | Host address and port for Axum server |
| `PRIV_KEY_PATH` | `/app/data/keys/ed_private.pem` | Path to Ed25519 private key |
| `PUB_KEY_PATH` | `/app/data/keys/ed_public.pem` | Path to Ed25519 public key |
| `SQLX_OFFLINE` | `true` (in Docker build) | Skip live DB connection during `sqlx` compilation |
| `ENCRYPTION_KEY` | `None` | Custom encryption key for action credentials |
| `ENABLE_OPENVINO` | `None` | Use OpenVINO for ML processing |
| `ENABLE_CUDA` | `None` | Use CUDA for ML processing |
---

## 💻 Local Native Development Setup

To build and run the backend natively outside Docker, you will need the native dependencies installed on your system.

### Prerequisites (Ubuntu/Debian)

```bash
# 1. Install build essentials, SQLite, GLib, and GStreamer dev dependencies
sudo apt-get update && sudo apt-get install -y \
    build-essential cmake clang pkg-config \
    libssl-dev libsqlite3-dev libglib2.0-dev \
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev

# 2. Install Intel OpenVINO 2026.3 runtime & headers
# (Follow Intel OpenVINO APT repository setup for your OS version)
```

### Running Migrations & Backend

```bash
# 1. Install SQLx CLI (optional, for running migrations manually)
cargo install sqlx-cli --no-default-features --features sqlite

# 2. Prepare database
cargo sqlx database setup

# 2. Run SQLx Migrations
cargo sqlx migrate run

# 3. Build and Run in Debug Mode
cargo run
```

---

## 📖 API Documentation & Swagger

Once the backend is running, access Swagger UI at:
* **Interactive UI:** `http://localhost:3000/swagger-ui/`