// SPDX-FileCopyrightText: Copyright (c) 2026 Contributors to the Eclipse Foundation
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::print_stdout)]

//! Offline capability applicability example.
//!
//! Loads an offline capability artifact and evaluates `x-sovd-applicability`
//! rules against resolved source values.
//!
//! ```text
//! cargo run -p opensovd-examples-client --example offline-capability
//! cargo run -p opensovd-examples-client --example offline-capability -- --file docs/api/offline-capability.json
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use opensovd_client::capability::{ApplicabilityError, is_applicable_with_rule_evaluator};
use opensovd_models::capability::XSovdApplicability;
use serde_json::Value;

#[derive(Parser)]
#[command(name = "offline-capability")]
#[command(about = "Evaluate x-sovd-applicability rules from an offline artifact")]
struct Cli {
    /// Path to the offline capability artifact JSON file.
    #[arg(long)]
    file: Option<String>,
}

fn default_artifact_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/api/offline-capability.json")
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
    if let Some(rhs) = expression.strip_prefix("source == ") {
        return Ok(source_path == parse_string_literal(rhs, expression, source_path)?);
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let file = cli.file.map_or_else(default_artifact_path, PathBuf::from);

    let content = fs::read_to_string(&file)?;
    let root: Value = serde_json::from_str(&content)?;

    let sources = HashMap::from([
        (
            "/components/PowerSteering#variant".to_string(),
            Value::String("Pow_Str_Variant_High".to_string()),
        ),
        (
            "/components/PowerSteering/data/swVersion".to_string(),
            Value::String("1.0.7".to_string()),
        ),
    ]);

    let Some(paths) = root.get("paths").and_then(Value::as_object) else {
        println!("No paths section found in {}", file.display());
        return Ok(());
    };

    for (path, node) in paths {
        let app_node = node.get("x-sovd-applicability");
        if app_node.is_none() {
            println!("{path} -> no x-sovd-applicability (treated as applicable)");
            continue;
        }

        let applicability: XSovdApplicability = serde_json::from_value(
            app_node
                .cloned()
                .ok_or("missing x-sovd-applicability value")?,
        )?;

        match is_applicable_with_rule_evaluator(
            &applicability,
            |source| sources.get(source).cloned(),
            rule_evaluator,
        ) {
            Ok(matched) => println!("{path} -> applicable={matched}"),
            Err(err) => println!("{path} -> error={err}"),
        }
    }

    Ok(())
}
