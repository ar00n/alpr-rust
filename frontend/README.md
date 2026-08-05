# 🎨 ALPR Frontend (Vite / React / Shadcn UI)

The frontend for the ALPR system provides a dashboard for viewing real-time streams, recognized license plates, historical snapshot logs, and system settings.

---

## 🛠 Tech Stack

* **Build Tool:** [Vite](https://vitejs.dev/)
* **Package Manager:** `pnpm`
* **UI Framework:** React (TypeScript)
* **Styling & Components:** [Tailwind CSS](https://tailwindcss.com/) + [Shadcn UI](https://ui.shadcn.com/)
* **Production Web Server:** Nginx

---

## 🚀 Local Development Setup

### 1. Prerequisites
Make sure you have [Node.js](https://nodejs.org/) (>= 20) and `pnpm` installed:

```bash
corepack enable pnpm
```

### 2. Install Dependencies
```bash
pnpm install
```

### 3. Start Development Server
```bash
pnpm run dev
```
The application will be available at `http://localhost:5173`.

### 4. Build for Production
```bash
pnpm build
```
The compiled static production assets will be generated in the `dist/` directory.

---

## 🐳 Nginx & Production Proxying

In production (inside Docker), Nginx serves the static SPA build and proxies API requests to the Axum backend service running on port `3000`.

Refer to `nginx.conf` for route handling and proxy pass definitions.