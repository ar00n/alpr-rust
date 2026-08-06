# 🦀 ALPR Backend (Rust / Axum)

The backend service handles real-time video processing, license plate detection, optical character recognition (OCR), database storage, custom webhook action execution, and REST API routing.

---

## 🛠 Features

* **High-Speed Inference:** Powered by `ort` (ONNX Runtime) utilizing OpenVINO hardware acceleration.
* **Axum Web Server:** Asynchronous, low-overhead HTTP API.
* **Automated Webhooks & Actions:** Asynchronous background execution of dynamic HTTP requests (`GET`, `POST`, `PUT`, `DELETE`, `PATCH`) triggered on allow-listed plate detections.
* **Encrypted Secrets Management:** Secure encrypted storage for sensitive authentication data (`BASIC`, `BEARER`, `API_KEY`) in SQLite.
* **Auto-Generated OpenAPI/Swagger Docs:** Served dynamically via `utoipa` and `utoipa-swagger-ui`.
* **SQLx Database Integration:** Async SQLite database access with compile-time checked queries and auto-migrations.
* **Video Stream Processing:** GStreamer integration for RTSP streams and video files.

---

## ⚡ Custom Action Engine

When the inference pipeline detects a license plate that exists in the **Allow List**, the backend spawns asynchronous background tasks to execute all configured `custom_actions`.

### Execution Workflow
1. **Plate Recognition:** The ANPR pipeline detects and decodes a license plate, then checks if it matches an entry in the `allow_list` database table.
2. **Action Query:** Registered actions are loaded from the `custom_actions` SQLite table.
3. **Secret Decryption:** If `auth_type` requires credentials (`BASIC`, `BEARER`, `API_KEY`), `auth_data` is decrypted using the secret key supplied via `ENCRYPTION_KEY`.
4. **Template Replacement:** Dynamic placeholders inside `body_template`, `url`, or headers (such as `${LICENCE_PLATE}`) are replaced with the recognized plate string.
5. **Async HTTP Request:** The HTTP call is dispatched asynchronously via an internal HTTP client without blocking the inference pipeline or video stream ingestion.

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
| `ENCRYPTION_KEY` | `None` | Custom secret key used to encrypt/decrypt credentials stored in `auth_data` |
| `ENABLE_OPENVINO` | `None` | Use OpenVINO acceleration for ML processing |
| `ENABLE_CUDA` | `None` | Use CUDA acceleration for ML processing |

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

# 2. Install Intel OpenVINO 2026 runtime & headers
# (Follow Intel OpenVINO APT repository setup for your OS version)
```

### Running Migrations & Backend

```bash
# 1. Install SQLx CLI (optional, for running migrations manually)
cargo install sqlx-cli --no-default-features --features sqlite

# 2. Prepare database
cargo sqlx database setup

# 3. Run SQLx Migrations
cargo sqlx migrate run

# 4. Build and Run in Debug Mode
cargo run
```

---

## 📖 API Documentation & Swagger

Once the backend is running, access Swagger UI at:
* **Interactive UI:** `http://localhost:3000/swagger-ui/`