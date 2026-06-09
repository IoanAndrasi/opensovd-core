// SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
// SPDX-License-Identifier: Apache-2.0

//! Offline capability applicability evaluation utilities.

use opensovd_models::capability::{MatchType, SourceValueMatch, XSovdApplicability};
use semver::{Version, VersionReq};
use serde_json::Value;

/// Structured errors produced while evaluating applicability constraints.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApplicabilityError {
    #[error("missing source value for '{source_path}'")]
    MissingSource { source_path: String },
    #[error("empty values are not allowed for {mode:?} on source '{source_path}'")]
    EmptyValues {
        source_path: String,
        mode: MatchType,
    },
    #[error(
        "invalid actual value type for {mode:?} on source '{source_path}': expected {expected}, got {actual_type}"
    )]
    InvalidActualType {
        source_path: String,
        mode: MatchType,
        expected: &'static str,
        actual_type: &'static str,
    },
    #[error("invalid regex '{pattern}' for source '{source_path}': {message}")]
    InvalidRegex {
        source_path: String,
        pattern: String,
        message: String,
    },
    #[error("invalid semver expression '{expr}' for source '{source_path}'")]
    InvalidSemver { source_path: String, expr: String },
    #[error("invalid rule expression '{expr}' for source '{source_path}': {reason}")]
    InvalidRule {
        source_path: String,
        expr: String,
        reason: String,
    },
    #[error(
        "no rule evaluator is configured for '{expr}' on source '{source_path}'; rule matching is manufacturer-specific"
    )]
    RuleEvaluatorUnavailable { source_path: String, expr: String },
}

/// Evaluate whether an applicability extension matches the resolved source values.
///
/// All `versions` entries and all `variants` entries must match.
///
/// `rule` matching is manufacturer-specific. Callers needing `MatchType::Rule`
/// should use [`is_applicable_with_rule_evaluator`].
pub fn is_applicable<F>(
    applicability: &XSovdApplicability,
    resolve: F,
) -> Result<bool, ApplicabilityError>
where
    F: FnMut(&str) -> Option<Value>,
{
    is_applicable_with_rule_evaluator(applicability, resolve, |rule, source_path, _actual| {
        Err(ApplicabilityError::RuleEvaluatorUnavailable {
            source_path: source_path.to_string(),
            expr: value_to_string(rule),
        })
    })
}

/// Evaluate applicability with a caller-supplied rule evaluator.
///
/// The rule evaluator is responsible for interpreting `MatchType::Rule` values,
/// which are manufacturer-specific according to ISO 17978-3.
pub fn is_applicable_with_rule_evaluator<F, R>(
    applicability: &XSovdApplicability,
    mut resolve: F,
    mut rule_evaluator: R,
) -> Result<bool, ApplicabilityError>
where
    F: FnMut(&str) -> Option<Value>,
    R: FnMut(&Value, &str, &Value) -> Result<bool, ApplicabilityError>,
{
    Ok(
        matches_group(&applicability.versions, &mut resolve, &mut rule_evaluator)?
            && matches_group(&applicability.variants, &mut resolve, &mut rule_evaluator)?,
    )
}

/// Lossy convenience helper for legacy callers that only need a boolean.
///
/// Any evaluation error is treated as non-applicable.
pub fn is_applicable_lossy<F>(applicability: &XSovdApplicability, resolve: F) -> bool
where
    F: FnMut(&str) -> Option<Value>,
{
    is_applicable(applicability, resolve).unwrap_or(false)
}

fn matches_group<F, R>(
    rules: &[SourceValueMatch],
    resolve: &mut F,
    rule_evaluator: &mut R,
) -> Result<bool, ApplicabilityError>
where
    F: FnMut(&str) -> Option<Value>,
    R: FnMut(&Value, &str, &Value) -> Result<bool, ApplicabilityError>,
{
    for rule in rules {
        let actual = resolve(&rule.source).ok_or_else(|| ApplicabilityError::MissingSource {
            source_path: rule.source.clone(),
        })?;
        if !evaluate_match_with_rule_evaluator(&actual, rule, rule_evaluator)? {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Evaluate a single source value against a matching rule.
///
/// `rule` matching is manufacturer-specific. Callers needing `MatchType::Rule`
/// should use [`evaluate_match_with_rule_evaluator`].
pub fn evaluate_match(actual: &Value, rule: &SourceValueMatch) -> Result<bool, ApplicabilityError> {
    evaluate_match_with_rule_evaluator(actual, rule, &mut |rule_value, source_path, _actual| {
        Err(ApplicabilityError::RuleEvaluatorUnavailable {
            source_path: source_path.to_string(),
            expr: value_to_string(rule_value),
        })
    })
}

/// Evaluate a single source value with a caller-supplied rule evaluator.
pub fn evaluate_match_with_rule_evaluator<R>(
    actual: &Value,
    rule: &SourceValueMatch,
    rule_evaluator: &mut R,
) -> Result<bool, ApplicabilityError>
where
    R: FnMut(&Value, &str, &Value) -> Result<bool, ApplicabilityError>,
{
    let mode = rule.match_type.unwrap_or(MatchType::Equals);
    if rule.values.is_empty() {
        return Err(ApplicabilityError::EmptyValues {
            source_path: rule.source.clone(),
            mode,
        });
    }

    match mode {
        MatchType::Equals => Ok(rule.values.iter().any(|value| actual == value)),
        MatchType::NotEquals => Ok(rule.values.iter().all(|value| actual != value)),
        MatchType::Regexp => {
            let actual = actual
                .as_str()
                .ok_or_else(|| ApplicabilityError::InvalidActualType {
                    source_path: rule.source.clone(),
                    mode,
                    expected: "string",
                    actual_type: value_type_name(actual),
                })?;

            for pattern in &rule.values {
                let pattern = pattern
                    .as_str()
                    .ok_or_else(|| ApplicabilityError::InvalidRegex {
                        source_path: rule.source.clone(),
                        pattern: value_to_string(pattern),
                        message: "regex patterns must be strings".to_string(),
                    })?;
                let re = regex::Regex::new(pattern).map_err(|error| {
                    ApplicabilityError::InvalidRegex {
                        source_path: rule.source.clone(),
                        pattern: pattern.to_string(),
                        message: error.to_string(),
                    }
                })?;
                if re.is_match(actual) {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        MatchType::Semver => {
            let actual = actual
                .as_str()
                .ok_or_else(|| ApplicabilityError::InvalidActualType {
                    source_path: rule.source.clone(),
                    mode,
                    expected: "string",
                    actual_type: value_type_name(actual),
                })?;

            for expr in &rule.values {
                let expr_string = expr
                    .as_str()
                    .map_or_else(|| value_to_string(expr), ToString::to_string);
                if semver_match(actual, &expr_string).map_err(|()| {
                    ApplicabilityError::InvalidSemver {
                        source_path: rule.source.clone(),
                        expr: expr_string.clone(),
                    }
                })? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        MatchType::Rule => {
            for expr in &rule.values {
                if rule_evaluator(expr, &rule.source, actual)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
    }
}

fn semver_match(actual: &str, expr: &str) -> Result<bool, ()> {
    let current = Version::parse(actual).map_err(|_| ())?;
    let req = normalize_semver_expr(expr)?;
    let req = VersionReq::parse(&req).map_err(|_| ())?;
    Ok(req.matches(&current))
}

fn normalize_semver_expr(expr: &str) -> Result<String, ()> {
    let trimmed = expr.trim();

    if let Some((left, right)) = trimmed.split_once(" - ") {
        let left = left.trim();
        let right = right.trim();
        Version::parse(left).map_err(|_| ())?;
        Version::parse(right).map_err(|_| ())?;
        return Ok(format!(">={left}, <={right}"));
    }

    let wildcard = normalize_semver_wildcards(trimmed);
    if VersionReq::parse(&wildcard).is_ok() {
        return Ok(wildcard);
    }

    Version::parse(trimmed).map_err(|_| ())?;
    Ok(format!("={trimmed}"))
}

fn normalize_semver_wildcards(expr: &str) -> String {
    expr.split('.')
        .map(|part| {
            if part.eq_ignore_ascii_case("x") {
                "*".to_string()
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(string) => string.clone(),
        _ => value.to_string(),
    }
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use opensovd_models::capability::{MatchType, SourceValueMatch, XSovdApplicability};
    use serde_json::{Value, json};

    use super::{
        ApplicabilityError, evaluate_match, evaluate_match_with_rule_evaluator, is_applicable,
        is_applicable_with_rule_evaluator,
    };

    fn rule(source: &str, mode: MatchType, values: &[Value]) -> SourceValueMatch {
        SourceValueMatch {
            source: source.to_string(),
            match_type: Some(mode),
            values: values.to_vec(),
        }
    }

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
                mode: MatchType::Rule,
                expected: "string",
                actual_type: super::value_type_name(actual),
            })?;

        expression
            .split(';')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .try_fold(true, |matched, clause| {
                if !matched {
                    return Ok(false);
                }
                evaluate_rule_clause(clause, actual, source_path).map(|result| matched && result)
            })
    }

    fn evaluate_rule_clause(
        clause: &str,
        actual: &str,
        source_path: &str,
    ) -> Result<bool, ApplicabilityError> {
        if let Some(rhs) = clause.strip_prefix("actual == ") {
            return Ok(actual == parse_string_literal(rhs, clause, source_path)?);
        }
        if let Some(rhs) = clause.strip_prefix("input.actual == ") {
            return Ok(actual == parse_string_literal(rhs, clause, source_path)?);
        }
        if let Some(rhs) = clause.strip_prefix("source == ") {
            return Ok(source_path == parse_string_literal(rhs, clause, source_path)?);
        }

        Err(ApplicabilityError::InvalidRule {
            source_path: source_path.to_string(),
            expr: clause.to_string(),
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

    #[test]
    fn test_evaluate_match_equals_string() {
        let got = evaluate_match(
            &json!("1.2.3"),
            &rule(
                "/components/c1/data/swVersion",
                MatchType::Equals,
                &[json!("1.2.3")],
            ),
        );
        assert!(matches!(got, Ok(true)));
    }

    #[test]
    fn test_evaluate_match_equals_boolean_any_value() {
        let got = evaluate_match(
            &json!(true),
            &rule(
                "/components/c1/data/enabled",
                MatchType::Equals,
                &[json!(false), json!(true)],
            ),
        );
        assert!(matches!(got, Ok(true)));
    }

    #[test]
    fn test_evaluate_match_not_equals() {
        let got = evaluate_match(
            &json!("1.2.3"),
            &rule(
                "/components/c1/data/swVersion",
                MatchType::NotEquals,
                &[json!("2.0.0")],
            ),
        );
        assert!(matches!(got, Ok(true)));
    }

    #[test]
    fn test_evaluate_match_regexp() {
        let got = evaluate_match(
            &json!("Pow_Str_Variant_High"),
            &rule(
                "/components/c1#variant",
                MatchType::Regexp,
                &[json!("^Pow_Str_.*_High$")],
            ),
        );
        assert!(matches!(got, Ok(true)));
    }

    #[test]
    fn test_evaluate_match_regexp_requires_string_actual() {
        let got = evaluate_match(
            &json!(123),
            &rule(
                "/components/c1#variant",
                MatchType::Regexp,
                &[json!("^123$")],
            ),
        );
        assert!(matches!(
            got,
            Err(ApplicabilityError::InvalidActualType { .. })
        ));
    }

    #[test]
    fn test_evaluate_match_semver_wildcard() {
        let got = evaluate_match(
            &json!("1.0.5"),
            &rule(
                "/components/c1/data/swVersion",
                MatchType::Semver,
                &[json!("1.0.x")],
            ),
        );
        assert!(matches!(got, Ok(true)));
    }

    #[test]
    fn test_evaluate_match_semver_range() {
        let got = evaluate_match(
            &json!("1.2.3"),
            &rule(
                "/components/c1/data/swVersion",
                MatchType::Semver,
                &[json!("1.2.0 - 1.3.0")],
            ),
        );
        assert!(matches!(got, Ok(true)));
    }

    #[test]
    fn test_evaluate_match_rule_requires_evaluator() {
        let got = evaluate_match(
            &json!("A34+B12"),
            &rule(
                "/components/c1#variant",
                MatchType::Rule,
                &[json!("actual == \"A34+B12\"")],
            ),
        );
        assert!(matches!(
            got,
            Err(ApplicabilityError::RuleEvaluatorUnavailable { .. })
        ));
    }

    #[test]
    fn test_evaluate_match_rule_with_rule_evaluator() {
        let got = evaluate_match_with_rule_evaluator(
            &json!("A34+B12"),
            &rule(
                "/components/c1#variant",
                MatchType::Rule,
                &[json!("actual == \"A34+B12\"")],
            ),
            &mut rule_evaluator,
        );
        assert!(matches!(got, Ok(true)));
    }

    #[test]
    fn test_evaluate_match_rule_supports_boolean_composition() {
        let got = evaluate_match_with_rule_evaluator(
            &json!("A34+B12"),
            &rule(
                "/components/c1#variant",
                MatchType::Rule,
                &[json!(
                    "actual == \"A34+B12\"; source == \"/components/c1#variant\""
                )],
            ),
            &mut rule_evaluator,
        );
        assert!(matches!(got, Ok(true)));
    }

    #[test]
    fn test_evaluate_match_rejects_empty_values() {
        let got = evaluate_match(
            &json!("A34+B12"),
            &rule("/components/c1#variant", MatchType::Rule, &[]),
        );
        assert!(matches!(got, Err(ApplicabilityError::EmptyValues { .. })));
    }

    #[test]
    fn test_evaluate_match_invalid_rule_expression() {
        let got = evaluate_match_with_rule_evaluator(
            &json!("A34+B12"),
            &rule(
                "/components/c1#variant",
                MatchType::Rule,
                &[json!("input.variant == \"A34+B12\"")],
            ),
            &mut rule_evaluator,
        );
        assert!(matches!(got, Err(ApplicabilityError::InvalidRule { .. })));
    }

    #[test]
    fn test_evaluate_match_semver_wildcard_non_match_is_false() {
        let got = evaluate_match(
            &json!("1.0.7"),
            &rule(
                "/components/c1/data/swVersion",
                MatchType::Semver,
                &[json!("1.1.x")],
            ),
        );
        assert!(matches!(got, Ok(false)));
    }

    #[test]
    fn test_evaluate_match_semver_comparator_chain() {
        let got = evaluate_match(
            &json!("1.2.3"),
            &rule(
                "/components/c1/data/swVersion",
                MatchType::Semver,
                &[json!(">=1.0.0, <2.0.0")],
            ),
        );
        assert!(matches!(got, Ok(true)));
    }

    #[test]
    fn test_is_applicable_requires_all_rules() {
        let app = XSovdApplicability {
            versions: vec![rule(
                "/components/c1/data/swVersion",
                MatchType::Semver,
                &[json!("1.0.x")],
            )],
            variants: vec![rule(
                "/components/c1#variant",
                MatchType::Equals,
                &[json!("Pow_Str_Variant_High")],
            )],
        };

        let matched = is_applicable(&app, |source| match source {
            "/components/c1/data/swVersion" => Some(json!("1.0.7")),
            "/components/c1#variant" => Some(json!("Pow_Str_Variant_High")),
            _ => None,
        });

        assert!(matches!(matched, Ok(true)));
    }

    #[test]
    fn test_is_applicable_errors_on_missing_source() {
        let app = XSovdApplicability {
            versions: vec![rule(
                "/components/c1/data/swVersion",
                MatchType::Semver,
                &[json!("1.0.x")],
            )],
            variants: Vec::new(),
        };

        let matched = is_applicable(&app, |_source| None);
        assert!(matches!(
            matched,
            Err(ApplicabilityError::MissingSource { .. })
        ));
    }

    #[test]
    fn test_is_applicable_with_rule_evaluator() {
        let app = XSovdApplicability {
            versions: Vec::new(),
            variants: vec![rule(
                "/components/c1#variant",
                MatchType::Rule,
                &[json!("input.actual == \"Pow_Str_Variant_High\"")],
            )],
        };

        let matched = is_applicable_with_rule_evaluator(
            &app,
            |source| match source {
                "/components/c1#variant" => Some(json!("Pow_Str_Variant_High")),
                _ => None,
            },
            rule_evaluator,
        );

        assert!(matches!(matched, Ok(true)));
    }
}
