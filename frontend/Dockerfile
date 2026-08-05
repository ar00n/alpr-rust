# Stage 1: Build Vite App
FROM node:24-slim AS builder
ENV CI=true
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

WORKDIR /app
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./

# Use Docker BuildKit cache for pnpm
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile

COPY . .
RUN pnpm build

# Stage 2: Nginx Web Server
FROM nginx:latest
# Copy our custom nginx config for SPA routing and API proxying
COPY nginx.conf /etc/nginx/conf.d/default.conf

COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]