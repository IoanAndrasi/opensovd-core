#!/usr/bin/env python3
# SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
# SPDX-License-Identifier: Apache-2.0

import argparse
import json
from pathlib import Path


def build_document() -> dict:
    return {
        "openapi": "3.1.0",
        "info": {
            "title": "OpenSOVD Offline Capability Description",
            "version": "0.1.0",
            "description": "Offline capability artifact with x-sovd-applicability examples.",
        },
        "servers": [{"url": "https://{sovd-server-host}/sovd/v1"}],
        "paths": {
            "/components/PowerSteering/data/SteeringWheelRotationSpeed": {
                "x-sovd-applicability": {
                    "variants": [
                        {
                            "source": "/components/PowerSteering#variant",
                            "values": ["Pow_Str_Variant_High"],
                        }
                    ],
                    "versions": [
                        {
                            "source": "/components/PowerSteering/data/swVersion",
                            "match": "semver",
                            "values": ["1.0.x"],
                        }
                    ],
                },
                "get": {
                    "summary": "Read steering wheel rotation speed",
                    "responses": {
                        "200": {
                            "description": "Successful response",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {"value": {"type": "number"}},
                                    }
                                }
                            },
                        }
                    },
                },
            },
            "/components/PowerSteering/data/SteeringMonitors": {
                "get": {
                    "responses": {
                        "200": {
                            "description": "Successful response",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "SteeringTorque": {"type": "number"},
                                            "SteeringWheelAngle": {"type": "number"},
                                            "SteeringWheelRotationSpeed": {"type": "number"},
                                        },
                                    }
                                }
                            },
                        }
                    },
                },
                "x-sovd-applicability": {
                    "versions": [
                        {
                            "source": "/components/PowerSteering/data/swVersion",
                            "match": "semver",
                            "values": ["1.1.x"],
                        }
                    ]
                },
            },
            "/components/PowerSteering/data/SteeringAssistMode": {
                "x-sovd-applicability": {
                    "variants": [
                        {
                            "source": "/components/PowerSteering#variant",
                            "match": "rule",
                            "values": ['actual == "Pow_Str_Variant_High"'],
                        }
                    ]
                },
                "get": {
                    "responses": {
                        "200": {
                            "description": "Successful response",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {"value": {"type": "string"}},
                                    }
                                }
                            },
                        }
                    },
                },
            },
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate an offline capability artifact for OpenSOVD docs"
    )
    parser.add_argument(
        "--output",
        default="docs/api/offline-capability.json",
        help="Output file path",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(build_document(), indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
