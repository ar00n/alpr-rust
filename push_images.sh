#!/usr/bin/env bash

set -euo pipefail

# Configuration
REGISTRY="ghcr.io"
REPO_PATH="ar00n/alpr-rust"
TAG="latest"

FRONTEND_IMAGE="${REGISTRY}/${REPO_PATH}/frontend:${TAG}"
BACKEND_IMAGE="${REGISTRY}/${REPO_PATH}/backend:${TAG}"

echo "=========================================="
echo "Starting Build & Push to GHCR"
echo "=========================================="

# 1. Login to GHCR if GH_TOKEN or CR_PAT environment variable is provided
if [ -n "${GH_TOKEN:-}" ]; then
  echo "Logging into ${REGISTRY} using GH_TOKEN..."
  echo "${GH_TOKEN}" | docker login "${REGISTRY}" -u "ar00n" --password-stdin
elif [ -n "${CR_PAT:-}" ]; then
  echo "Logging into ${REGISTRY} using CR_PAT..."
  echo "${CR_PAT}" | docker login "${REGISTRY}" -u "ar00n" --password-stdin
else
  echo "Note: No GH_TOKEN/CR_PAT detected. Assuming you are already logged in to ${REGISTRY}."
fi

# 2. Build and Push Frontend Image
echo ""
echo "--> Building Frontend image: ${FRONTEND_IMAGE}"
docker build -t "${FRONTEND_IMAGE}" ./frontend

echo "--> Pushing Frontend image..."
docker push "${FRONTEND_IMAGE}"

# 3. Build and Push Backend Image
echo ""
echo "--> Building Backend image: ${BACKEND_IMAGE}"
docker build -t "${BACKEND_IMAGE}" ./backend

echo "--> Pushing Backend image..."
docker push "${BACKEND_IMAGE}"

echo ""
echo "=========================================="
echo "Successfully built and pushed all images!"
echo "  - ${FRONTEND_IMAGE}"
echo "  - ${BACKEND_IMAGE}"
echo "=========================================="