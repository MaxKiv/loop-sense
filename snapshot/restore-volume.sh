#!/usr/bin/env bash

set -e

MARKER="/data/.restored"

if [ -f "$MARKER" ]; then
  echo "[restore] Snapshot already restored, skipping."
else
  echo "[restore] Restoring snapshot to volume..."
  tar -xzf /backup/influxdb3-data.tar.gz -C /data
  touch "$MARKER"
  echo "[restore] Done."
fi
