// SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
// SPDX-License-Identifier: Apache-2.0

//! Capability description extension types.

use serde::{Deserialize, Serialize};

/// `x-sovd-applicability` extension object.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct XSovdApplicability {
    /// Version identifiers for which the parent element is applicable.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub versions: Vec<SourceValueMatch>,
    /// Variant identifiers for which the parent element is applicable.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub variants: Vec<SourceValueMatch>,
}

/// Helper wrapper for serializing extension payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct SovdApplicabilityExtension {
    /// SOVD applicability metadata.
    #[serde(rename = "x-sovd-applicability")]
    pub x_sovd_applicability: XSovdApplicability,
}

/// Source-based value matching rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct SourceValueMatch {
    /// URI reference to the source value.
    pub source: String,
    /// How to match the source value against `values`.
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_type: Option<MatchType>,
    /// Expected values used by the selected match mode.
    pub values: Vec<serde_json::Value>,
}

/// Supported applicability match modes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    Equals,
    NotEquals,
    Regexp,
    Semver,
    Rule,
}
