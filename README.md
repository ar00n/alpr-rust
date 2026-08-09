# 🚘 High-Performance ALPR / ANPR System

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/react-%2320232a.svg?style=for-the-badge&logo=react&logoColor=%2361DAFB)
![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)
![ONNX](https://img.shields.io/badge/ONNX-005CED?style=for-the-badge&logo=onnx&logoColor=white)

A real-time Automatic License Plate Recognition (ALPR) application built with a high-performance **Rust** backend and a modern **Vite + Shadcn UI** frontend. 

The inference pipeline leverages **ONNX Runtime (ort)**—with support for **CUDA** and **Intel OpenVINO** hardware acceleration—for low-latency YOLO object detection and PaddleOCR recognition. It includes full **GStreamer** integration for robust, real-time video stream ingestion.

⚡ **Automated Action Engine:** When a license plate matching your **Allow List** is detected, the system can automatically execute custom webhooks and HTTP actions to trigger smart barriers, send notifications, or integrate with external APIs (e.g., Home Assistant, IoT controllers, security logs).

---

<details>
  <summary>📸 <b>Click to view Dashboard & Snapshot Screenshots</b></summary>
  <br>

  <p align="center">
    <b>Real-Time Stream Overview</b><br>
    <img src="https://github.com/ar00n/alpr-rust/blob/main/demo_pictures/live.jpg" width="800" alt="Live">
  </p>
  
  <p align="center">
    <b>License Plate Detection Log</b><br>
    <img src="https://github.com/ar00n/alpr-rust/blob/main/demo_pictures/history.jpg" width="800" alt="Detections">
  </p>

  <p align="center">
    <b>License Plate Allow List</b><br>
    <img src="https://github.com/ar00n/alpr-rust/blob/main/demo_pictures/allowlist.jpg?raw=true" width="800" alt="Allowlist">
  </p>

  <p align="center">
    <b>Settings</b><br>
    <img src="https://github.com/ar00n/alpr-rust/blob/main/demo_pictures/settings.png" width="800" alt="Settings">
  </p>
</details>

---

## 🏗 System Architecture

```text
                                  🌐 Internet
                                       | (No Port Forwarding Required)
                                       v
+-------------------------------------------------------------------------+
|                              Docker Compose                             |
|                                                                         |
|  +----------------------+        +-----------------------------------+  |
|  |  Cloudflare Tunnel   | -----> |     Frontend Container (Nginx)    |  |
|  |  (cloudflared)       |        |     Vite + React + Shadcn UI      |  |
|  +----------------------+        +-----------------------------------+  |
|                                                    |                    |
|                                           Proxy / API (Port 3000)       |
|                                                    v                    |
|                                          +-------------------+          |
|                                          |      Backend      |          |
|                                          |    (Axum Web)     |          |
|                                          +---------+---------+          |
|                                                    |                    |
|                                         SQLite DB  |  ort               |
|                                        +-----------+-----+              |
|                                        | ANPR Engine     |              |
|                                        | (YOLO + PP-OCR) |              |
|                                        +-------+---------+              |
+------------------------------------------------|------------------------+
                                                 |
                              Allow-list Match   | (HTTP Webhooks)
                              Triggers Action    v
                                     +-----------------------+
                                     |  External Services    |
                                     | (Gates, APIs, Alerts) |
                                     +-----------------------+
```

---

## ⚡ Key Features

- **Real-Time Detection & Recognition:** YOLO object detection combined with PaddleOCR for high-accuracy text extraction.
- **Hardware Acceleration:** Native support for CUDA and Intel OpenVINO for high-throughput AI inference.
- **RTSP & Video Stream Ingestion:** Native hardware-accelerated video streaming powered by GStreamer.
- **Secure Remote Access (Zero Trust):** Built-in Cloudflare Tunnel support for secure remote access without router port forwarding.
- **Allow List Management:** Maintain a secure database of authorized vehicles/license plates.
- **Automated Webhooks / Custom Actions:** Dispatch dynamic HTTP requests automatically upon detecting allow-listed vehicles.
- **Secure Encrypted Storage:** Sensitive HTTP authentication details (tokens, passwords) are encrypted using Ed25519 before being stored in the database.

---

## 🛠 Tech Stack

### **Backend**
* **Framework:** Rust, [Axum](https://github.com/tokio-rs/axum)
* **API Documentation:** OpenAPI / Swagger via [Utoipa](https://github.com/juhoteperi/utoipa)
* **ML / Inference Engine:** `ort` (ONNX Runtime) 
* **Video Pipeline:** GStreamer 1.0 (RTSP & video file ingestion)
* **Database:** SQLite managed via [SQLx](https://github.com/launchbadge/sqlx) (with auto-migrations)
* **Key Encryption:** Ed25519 keypair authentication & encrypted payload secret storage

### **Frontend**
* **Framework:** Vite + React (TypeScript)
* **UI & Components:** Tailwind CSS, [Shadcn UI](https://ui.shadcn.com/)
* **Package Manager:** `pnpm`
* **Web Server:** Nginx (Containerized SPA host & reverse proxy)

---

## ⚙️ Custom Actions / Webhooks

When a license plate is recognized and matched against the **Allow List**, the backend triggers all configured **Custom Actions**. This allows seamless integration with relays, gate controllers, Home Assistant, Slack/Discord webhooks, or custom APIs.

### Action Capabilities
* **HTTP Methods Supported:** `GET`, `POST`, `PUT`, `DELETE`, `PATCH`
* **Authentication Types:** `NONE`, `BASIC`, `BEARER`, `API_KEY` (Auth secrets are securely stored as encrypted JSON).
* **Dynamic Body Templating:** Inject variable values into request payloads. Example: `{"plate": "${LICENCE_PLATE}"}`
* **Custom Headers:** Configurable JSON object containing key-value request headers.

---

## 🧠 ML Inference Models

The core ANPR pipeline relies on the following model files (located inside `backend/models/`). 
*Note: These are pre-packaged within the repository and pre-built Docker images, requiring no manual downloads.*

| File | Description |
|---|---|
| `number-plate-yolo26n.onnx` | YOLO-based license plate bounding box detector |
| `PP-OCRv6_small_rec_onnx.onnx` | PaddleOCR v6 small text recognition model |

---

## 🚀 Quick Start (Docker Compose)

The easiest way to run the application is using the pre-built Docker images hosted on GitHub.

### 1. Prerequisites
- [Git](https://git-scm.com/) installed.
- [Docker](https://docs.docker.com/get-docker/) & [Docker Compose](https://docs.docker.com/compose/install/) installed.

### 2. Configure Environment & Cloudflare Tunnel (Optional)
To securely access your dashboard over the internet **without port forwarding**, you can provide a Cloudflare Tunnel token.

1. Clone the repository:
   ```bash
   git clone https://github.com/ar00n/alpr-rust.git
   cd alpr-rust
   ```
2. Create a `.env` file in the root directory:
   ```bash
   touch .env
   ```
3. Add your Cloudflare Tunnel token to the `.env` file (if you have one):
   ```env
   CLOUDFLARE_TOKEN=your_cloudflare_tunnel_token_here
   ```

### 3. Run the Application
From the repository root, start the full stack using the production compose file:

```bash
docker compose -f docker-compose.prod.yml up -d
```

**Accessing the System:**
* **Local Web Application:** `http://localhost:8080` (or via your Cloudflare Tunnel domain if configured).
* **Backend API / Swagger Docs:** `http://localhost:3000/swagger-ui` *(Note: Port 3000 must be exposed in your compose file to access this directly).*

### 4. Stop the Application
```bash
docker compose -f docker-compose.prod.yml down
```

---

## 📖 Project Documentation

- [Backend Documentation (`backend/README.md`)](./backend/README.md) – Native building, GStreamer setup, custom action handlers, database migrations, and API docs.
- [Frontend Documentation (`frontend/README.md`)](./frontend/README.md) – Local Vite development, component libraries, and scripts.