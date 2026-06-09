<!--
SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
SPDX-License-Identifier: Apache-2.0
-->

# Offline Capability Applicability

This page documents the offline capability artifact used by OpenSOVD examples.

## Docs API scope (current phase)

Current server-side scope is limited to data docs endpoint coverage:

- `GET /v1/components/{component_id}/data/docs`
- `GET /v1/apps/{app_id}/data/docs`

These endpoints return a scoped OpenAPI payload for the data collection routes of the requested entity.

Out of scope for this phase:

- server-side evaluation of `x-sovd-applicability`
- client-side applicability matching semantics
- full online/offline capability parity in a single step

## Canonical artifact

- JSON artifact: `docs/api/offline-capability.json`
- Generation script: `scripts/generate_offline_capability.py`

The examples read the artifact directly. If the file is missing on disk, the server example serves an embedded fallback copy.

## Supported applicability grammar

The `x-sovd-applicability` extension is modeled by `XSovdApplicability` and `SourceValueMatch`.

Source resolution is value-based. The client resolves each `source` to a `serde_json::Value`, so `equals` and `not_equals` can compare JSON scalars directly instead of coercing everything to strings.

Supported `match` values:

- `equals` (default): any value in `values` must equal `actual`
- `not_equals`: all values in `values` must differ from `actual`
- `regexp`: any regex in `values` must match a string `actual`; the current implementation uses Rust `regex`, so it is narrower than full ECMA-262 support from the ISO text
- `semver`: matches a string `actual` using exact versions, comparator chains, wildcard ranges such as `1.1.x`, and inclusive hyphen ranges such as `1.2.0 - 1.3.0`; this is implemented with Rust `semver` and is intended to cover the ISO examples, not to claim exact NodeJS parity for every corner case
- `rule`: manufacturer-specific. The library does not hard-code one rule language anymore; callers must provide a rule evaluator callback when they want to interpret `MatchType::Rule`

Rule handling in practice:

- `is_applicable(...)` returns `RuleEvaluatorUnavailable` when it encounters `match: rule`
- `is_applicable_with_rule_evaluator(...)` lets the caller plug in its ExVe-specific rule interpreter
- the example client ships with a small rule evaluator only to exercise the sample artifact; it is not presented as a universal ISO rule language

## Validation behavior

Evaluation returns structured errors for invalid inputs:

- missing source resolution
- empty `values`
- non-string actual values used with `regexp`, `semver`, or a string-only rule evaluator
- invalid regex patterns
- invalid semver expressions
- invalid rule expressions reported by the caller-provided evaluator
- missing rule evaluator for `match: rule`

## Standard reference

The implementation and these notes were reviewed against:

- `ISO 17978-3`
