// SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
// SPDX-License-Identifier: Apache-2.0

use opensovd_client::capability::{
    ApplicabilityError, is_applicable, is_applicable_with_rule_evaluator,
};
use opensovd_models::capability::XSovdApplicability;
use serde_json::{Value, json};

fn rule_evaluator(
    rule_value: &Value,
    source_path: &str,
    actual: &Value,
) -> Result<bool, ApplicabilityError> {
    let expression = rule_value
        .as_str()
        .ok_or_else(|| ApplicabilityError::InvalidRule {
            source_path: source_path.to_string(),
            expr: rule_value.to_string(),
            reason: "rule evaluator expects string expressions".to_string(),
        })?;

    let actual = actual
        .as_str()
        .ok_or_else(|| ApplicabilityError::InvalidActualType {
            source_path: source_path.to_string(),
            mode: opensovd_models::capability::MatchType::Rule,
            expected: "string",
            actual_type: match actual {
                Value::Null => "null",
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Array(_) => "array",
                Value::Object(_) => "object",
            },
        })?;

    if let Some(rhs) = expression.strip_prefix("actual == ") {
        return Ok(actual == parse_string_literal(rhs, expression, source_path)?);
    }
    if let Some(rhs) = expression.strip_prefix("input.actual == ") {
        return Ok(actual == parse_string_literal(rhs, expression, source_path)?);
    }

    Err(ApplicabilityError::InvalidRule {
        source_path: source_path.to_string(),
        expr: expression.to_string(),
        reason: "unsupported rule clause".to_string(),
    })
}

fn parse_string_literal<'a>(
    input: &'a str,
    expr: &str,
    source_path: &str,
) -> Result<&'a str, ApplicabilityError> {
    let trimmed = input.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .ok_or_else(|| ApplicabilityError::InvalidRule {
            source_path: source_path.to_string(),
            expr: expr.to_string(),
            reason: "expected double-quoted string literal".to_string(),
        })
}

fn load_applicability(path: &str) -> XSovdApplicability {
    let root: serde_json::Value =
        serde_json::from_str(include_str!("../../docs/api/offline-capability.json"))
            .unwrap_or_else(|e| panic!("failed to parse offline artifact JSON: {e}"));

    let node = root
        .get("paths")
        .and_then(|v| v.get(path))
        .and_then(|v| v.get("x-sovd-applicability"))
        .unwrap_or_else(|| panic!("missing x-sovd-applicability for path: {path}"));

    serde_json::from_value(node.clone())
        .unwrap_or_else(|e| panic!("failed to deserialize applicability for path {path}: {e}"))
}

#[test]
fn artifact_parse_and_evaluate_happy_path() {
    let applicability =
        load_applicability("/components/PowerSteering/data/SteeringWheelRotationSpeed");

    let result = is_applicable(&applicability, |source| match source {
        "/components/PowerSteering#variant" => Some(json!("Pow_Str_Variant_High")),
        "/components/PowerSteering/data/swVersion" => Some(json!("1.0.7")),
        _ => None,
    });

    assert!(matches!(result, Ok(true)));
}

#[test]
fn rule_expression_is_supported_with_rule_evaluator() {
    let applicability = XSovdApplicability {
        versions: Vec::new(),
        variants: vec![opensovd_models::capability::SourceValueMatch {
            source: "/components/PowerSteering#variant".to_string(),
            match_type: Some(opensovd_models::capability::MatchType::Rule),
            values: vec![serde_json::Value::String(
                "input.actual == \"Pow_Str_Variant_High\"".to_string(),
            )],
        }],
    };

    let result = is_applicable_with_rule_evaluator(
        &applicability,
        |source| match source {
            "/components/PowerSteering#variant" => Some(json!("Pow_Str_Variant_High")),
            _ => None,
        },
        rule_evaluator,
    );

    assert!(matches!(result, Ok(true)));
}

#[test]
fn unknown_rule_field_returns_structured_error() {
    let applicability = XSovdApplicability {
        versions: Vec::new(),
        variants: vec![opensovd_models::capability::SourceValueMatch {
            source: "/components/PowerSteering#variant".to_string(),
            match_type: Some(opensovd_models::capability::MatchType::Rule),
            values: vec![serde_json::Value::String(
                "input.variant == \"Pow_Str_Variant_High\"".to_string(),
            )],
        }],
    };

    let result = is_applicable_with_rule_evaluator(
        &applicability,
        |source| match source {
            "/components/PowerSteering#variant" => Some(json!("Pow_Str_Variant_High")),
            _ => None,
        },
        rule_evaluator,
    );

    assert!(matches!(
        result,
        Err(ApplicabilityError::InvalidRule { .. })
    ));
}

#[test]
fn invalid_rule_expression_returns_structured_error() {
    let applicability = XSovdApplicability {
        versions: Vec::new(),
        variants: vec![opensovd_models::capability::SourceValueMatch {
            source: "/components/PowerSteering#variant".to_string(),
            match_type: Some(opensovd_models::capability::MatchType::Rule),
            values: vec![serde_json::Value::String("input.actual ==".to_string())],
        }],
    };

    let result = is_applicable_with_rule_evaluator(
        &applicability,
        |source| match source {
            "/components/PowerSteering#variant" => Some(json!("Pow_Str_Variant_High")),
            _ => None,
        },
        rule_evaluator,
    );

    assert!(matches!(
        result,
        Err(ApplicabilityError::InvalidRule { .. })
    ));
}
