# SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
# SPDX-License-Identifier: Apache-2.0

"""Integration test for the offline-capability server example endpoint."""

import json
import re
from pathlib import Path

import httpx

from fixtures import ProcessUnderTest

READY = re.compile(r"Offline capability endpoint available at (http://\S+)")
PROJECT_ROOT = Path(__file__).resolve().parents[2]


def test_offline_capability_example_endpoint_serves_artifact():
    proc = ProcessUnderTest.spawn(
        [
            "cargo",
            "run",
            "-p",
            "opensovd-examples-server",
            "--example",
            "offline-capability",
        ],
        timeout_seconds=180.0,
        ready_banner=READY,
    )

    try:
        if proc.match is None:
            raise RuntimeError("offline example did not publish endpoint URL")

        url = proc.match.group(1)
        response = httpx.get(url, timeout=10.0)

        assert response.status_code == 200
        assert response.headers["content-type"].startswith("application/json")

        payload = response.json()
        assert "paths" in payload

        expected = json.loads((PROJECT_ROOT / "docs" / "api" / "offline-capability.json").read_text())
        assert payload == expected
    finally:
        proc.close()
