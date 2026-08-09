# 🦀 ALPR Backend (Rust / Axum)

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![SQLite](https://img.shields.io/badge/sqlite-%2307405e.svg?style=for-the-badge&logo=sqlite&logoColor=white)
![ONNX](https://img.shields.io/badge/ONNX-005CED?style=for-the-badge&logo=onnx&logoColor=white)

The backend service handles real-time video processing, license plate detection, optical character recognition (OCR), database storage, custom webhook action execution, and REST API routing.

---

## 🛠 Features

* **High-Speed Inference:** Powered by `ort` (ONNX Runtime) with support for hardware acceleration via **CUDA** or **Intel OpenVINO**.
* **Axum Web Server:** Asynchronous, low-overhead HTTP API.
* **Automated Webhooks & Actions:** Asynchronous background execution of dynamic HTTP requests (`GET`, `POST`, `PUT`, `DELETE`, `PATCH`) triggered on allow-listed plate detections.
* **Encrypted Secrets Management:** Secure encrypted storage for sensitive authentication data (`BASIC`, `BEARER`, `API_KEY`) in SQLite using Ed25519.
* **Auto-Generated OpenAPI/Swagger Docs:** Served dynamically via `utoipa` and `utoipa-swagger-ui`.
* **SQLx Database Integration:** Async SQLite database access with compile-time checked queries and auto-migrations.
* **Video Stream Processing:** GStreamer integration for robust RTSP stream and video file ingestion.

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

The following ONNX models and dictionary files are required for the inference pipeline. 

*Note: These are pre-packaged within the repository (`backend/models/`) and Docker images, requiring no manual downloads.*

```text
backend/models/
├── number-plate-yolo26n.onnx
├── PP-OCRv6_small_rec_onnx.onnx
└── ppocrv6_dict.txt
```

---

## 📋 Environment Variables

The backend accepts the following environment variables (the defaults match the Docker Compose runtime):

| Variable | Default Value | Description |
|---|---|---|
| `DATABASE_URL` | `sqlite:///app/data/anpr.db?mode=rwc` | SQLite connection string |
| `SNAPSHOT_DIR` | `/app/snapshots` | Directory to save captured plate snapshots |
| `SERVER_ADDR` | `0.0.0.0:3000` | Host address and port for Axum server |
| `PRIV_KEY_PATH` | `/app/data/keys/ed_private.pem` | Path to Ed25519 private key |
| `PUB_KEY_PATH` | `/app/data/keys/ed_public.pem` | Path to Ed25519 public key |
| `SQLX_OFFLINE` | `true` (in Docker build) | Skip live DB connection during `sqlx` compilation |
| `ENCRYPTION_KEY` | `None` | Custom secret key used to encrypt/decrypt credentials stored in `auth_data` |
| `ENABLE_OPENVINO`| `None` | Set to enable Intel OpenVINO hardware acceleration |
| `ENABLE_CUDA` | `None` | Set to enable Nvidia CUDA hardware acceleration |

---

## 💻 Local Native Development Setup

If you prefer to build and run the backend natively outside of Docker, you will need the native C dependencies installed on your system.

### Prerequisites (Ubuntu/Debian)

```bash
# 1. Install build essentials, SQLite, GLib, and GStreamer dev dependencies
sudo apt-get update && sudo apt-get install -y \
    build-essential cmake clang pkg-config \
    libssl-dev libsqlite3-dev libglib2.0-dev \
    libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev

# Note: If you want to utilize hardware acceleration (ENABLE_CUDA or ENABLE_OPENVINO), 
# you will also need to install the respective Nvidia CUDA Toolkit or Intel OpenVINO 
# libraries required by ONNX Runtime for your specific OS.
```

### Running Migrations & Backend

```bash
# 1. Install SQLx CLI (optional, but required for running migrations manually)
cargo install sqlx-cli --no-default-features --features sqlite

# 2. Prepare the database
cargo sqlx database setup

# 3. Run SQLx Migrations
cargo sqlx migrate run

# 4. Build and Run in Debug Mode
cargo run
```

---

## 📖 API Documentation & Swagger

The backend automatically generates and serves OpenAPI documentation. Once the backend is running, you can access the Swagger UI at:

* **Interactive UI:** `http://localhost:3000/swagger-ui/`