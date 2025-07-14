#!/usr/bin/env bash
#
# To capture a snapshot of an influxdb3 instance running in docker, run `just docker-save-db-volume`

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
