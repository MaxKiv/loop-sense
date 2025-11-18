#!/usr/bin/env bash
# Script to pull and save ARM64 Docker images for Raspberry Pi 4

set -e

echo "Pulling ARM64 images for Raspberry Pi 4..."

# Pull ARM64 images
docker pull --platform linux/arm64 alpine:3.21.0
docker pull --platform linux/arm64 influxdb:3-core

echo "Saving images to tar file..."

# Save images to tar file
docker save -o influxdb-images-arm64.tar \
    alpine:3.21.0 \
    influxdb:3-core

echo "Done! Created influxdb-images-arm64.tar"
echo "File size: $(du -h influxdb-images-arm64.tar | cut -f1)"

