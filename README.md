# 🚘 High-Performance ALPR / ANPR System

A real-time Automatic License Plate Recognition (ALPR) application built with a high-performance **Rust** backend and a modern **Vite + Shadcn UI** frontend. 

The inference pipeline leverages **ONNX Runtime (ort)** accelerated by **Intel OpenVINO** for low-latency YOLO object detection and PaddleOCR recognition, with full **GStreamer** support for real-time video stream ingestion.

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

  <p align="center">
    <b>Settings 2</b><br>
    <img src="https://github.com/ar00n/alpr-rust/blob/main/demo_pictures/settings2.png" width="800" alt="Settings 2">
  </p>
</details>

---

## 🏗 System Architecture

```
                 +---------------------------------------+
                 |          Client (Browser)             |
                 +-------------------+-------------------+
                                     |
                             Port 8080 (HTTP)
                                     v
+------------------------------------+------------------------------------+
|                         Docker Compose                                  |
|                                                                         |
|  +-----------------------------------+   Proxy / API  +--------------+  |
|  |     Frontend Container (Nginx)    | -------------> |   Backend    |  |
|  |     Vite + React + Shadcn UI      |   (Port 3000)  |  (Axum Web)  |  |
|  +-----------------------------------+                +-------+------+  |
|                                                               |         |
|                                                    SQLite DB  |  ort    |
|                                                   +-----------+-----+   |
|                                                   | ANPR Engine     |   |
|                                                   | (YOLO + PP-OCR) |   |
|                                                   +-----------------+   |
+-------------------------------------------------------------------------+
```

---

## ⚡ Tech Stack

### **Backend**
* **Framework:** Rust, [Axum](https://github.com/tokio-rs/axum)
* **API Documentation:** OpenAPI / Swagger via [Utoipa](https://github.com/juhoteperi/utoipa)
* **ML / Inference Engine:** `ort` (ONNX Runtime) with Intel OpenVINO 2026 Acceleration
* **Video Pipeline:** GStreamer 1.0 (RTSP & video file ingestion)
* **Database:** SQLite managed via [SQLx](https://github.com/launchbadge/sqlx) (with auto-migrations)
* **Key Encryption:** Ed25519 keypair authentication setup

### **Frontend**
* **Framework:** Vite + React (TypeScript)
* **UI & Components:** Tailwind CSS, [Shadcn UI](https://ui.shadcn.com/)
* **Package Manager:** `pnpm`
* **Web Server:** Nginx (Containerized SPA host & reverse proxy)

---

## 🧠 ML Inference Models

The core ANPR pipeline relies on the following model files placed inside `backend/models/`:

| File | Description |
|---|---|
| `number-plate-yolo26n.onnx` | YOLO-based license plate detector |
| `PP-OCRv6_small_rec_onnx.onnx` | PaddleOCR v6 small text recognition ONNX model |
| `ppocrv6_dict.txt` | Character lookup dictionary for OCR decoding |

---

## 🚀 Quick Start (Docker Compose)

### 1. Prerequisites
- [Docker](https://docs.docker.com/get-docker/) & [Docker Compose](https://docs.docker.com/compose/install/) installed.

### 2. Run the Application
From the repository root, start the full stack:

```bash
docker compose up --build -d
```

Once running:
* **Web Application:** `http://localhost:8080`
* **Backend API / Swagger Specs:** `http://localhost:3000/swagger-ui` (must open port in docker-compose.yml)

### 3. Stop the Application
```bash
docker compose down
```

---

## 🛠 Project Documentation

- [Backend Documentation (`backend/README.md`)](./backend/README.md) – Native building, GStreamer/OpenVINO setup, database migrations, and API docs.
- [Frontend Documentation (`frontend/README.md`)](./frontend/README.md) – Local Vite development, component libraries, and scripts.