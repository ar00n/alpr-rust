# 🎨 ALPR Frontend

![React](https://img.shields.io/badge/react-%2320232a.svg?style=for-the-badge&logo=react&logoColor=%2361DAFB)
![Vite](https://img.shields.io/badge/vite-%23646CFF.svg?style=for-the-badge&logo=vite&logoColor=white)
![TailwindCSS](https://img.shields.io/badge/tailwindcss-%2338B2AC.svg?style=for-the-badge&logo=tailwind-css&logoColor=white)
![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white)

The frontend for the ALPR system provides a modern, responsive dashboard for viewing real-time video streams, recognized license plates, historical snapshot logs, and managing system settings/allow-lists.

---

## 🛠 Tech Stack

* **Build Tool:** [Vite](https://vitejs.dev/)
* **Package Manager:** `pnpm`
* **UI Framework:** React 18+ (TypeScript)
* **Styling:** [Tailwind CSS](https://tailwindcss.com/)
* **Component Library:** [Shadcn UI](https://ui.shadcn.com/)
* **Production Web Server:** Nginx

---

## 🚀 Local Development Setup

### 1. Prerequisites
Ensure you have [Node.js](https://nodejs.org/) (>= 20) and `pnpm` installed. You can enable `pnpm` via corepack:

```bash
corepack enable pnpm
```

### 2. Install Dependencies
Navigate to the `frontend` directory and install the required packages:
```bash
pnpm install
```

### 3. Start the Development Server
```bash
pnpm run dev
```
The application will be available at `http://localhost:5173`.

> 💡 **Note:** For the dashboard to display data and video streams, ensure the **Rust backend** is also running locally (either natively or via Docker) on port `3000`. Vite is typically configured to proxy `/api` requests to this port during development.

---

## 📜 Available Scripts

| Command | Description |
|---|---|
| `pnpm dev` | Starts the Vite development server with Hot Module Replacement (HMR). |
| `pnpm build` | Compiles TypeScript and builds the production-ready static assets into the `dist/` directory. |
| `pnpm preview` | Boots up a local static web server to preview the `dist/` production build. |
| `pnpm lint` | Runs ESLint to catch syntax and style issues. |

---

## 🧩 Adding UI Components

This project utilizes **Shadcn UI** for its core component architecture. If you need to add a new UI component to the project, you can easily pull it in using the CLI:

```bash
pnpm dlx shadcn@latest add <component-name>
```
*Example: `pnpm dlx shadcn@latest add dialog`*

---

## 🐳 Nginx & Production Proxying

In a production environment (such as our Docker Compose stack), the Vite development server is not used. Instead:

1. The frontend is built into static HTML/CSS/JS assets (`pnpm build`).
2. An **Nginx** container serves these static SPA files.
3. Nginx is configured as a reverse proxy, seamlessly routing all frontend API requests to the Axum backend service running on port `3000`.

*Refer to the `nginx.conf` file in this directory for the specific route handling and `proxy_pass` definitions.*