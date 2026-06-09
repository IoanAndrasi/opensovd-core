#!/usr/bin/env bash
# SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

cd "$(dirname "$0")/.."

echo "[1/4] Generate offline capability artifact"
python3 scripts/generate_offline_capability.py --output docs/api/offline-capability.json

echo "[2/4] Start offline-capability server example"
cargo run -p opensovd-examples-server --example offline-capability >/tmp/offline-capability-demo.log 2>&1 &
SERVER_PID=$!
trap 'kill $SERVER_PID 2>/dev/null || true' EXIT

echo "[3/4] Probe endpoint"
READY=0
for _ in $(seq 1 30); do
  if curl -fsS http://127.0.0.1:7790/offline-capability >/tmp/offline-capability.json 2>/dev/null; then
    READY=1
    break
  fi
  sleep 1
done

if [ "$READY" -ne 1 ]; then
  echo "offline-capability server did not become ready on 127.0.0.1:7790" >&2
  echo "--- server log ---" >&2
  tail -n 80 /tmp/offline-capability-demo.log >&2 || true
  exit 1
fi

echo "[4/4] Show key fields"
python3 - <<'PY'
import json
with open('/tmp/offline-capability.json', 'r', encoding='utf-8') as f:
    doc = json.load(f)
print('openapi:', doc.get('openapi'))
print('paths:', ', '.join(doc.get('paths', {}).keys()))
has_app = any('x-sovd-applicability' in v for v in doc.get('paths', {}).values())
print('has_x_sovd_applicability:', has_app)
PY

echo "Demo complete."
