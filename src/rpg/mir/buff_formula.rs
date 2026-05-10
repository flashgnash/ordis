use regex::Regex;
use std::collections::HashMap;

use crate::db::models::Gauge;

pub struct ParsedFormula {
    pub target: String,
    pub op: String,
    pub expr: String,
}

static OPERATORS: &[&str] = &["+=", "-=", "*=", "/="];

pub fn parse_formula(formula: &str) -> Option<ParsedFormula> {
    for op in OPERATORS {
        if let Some(idx) = formula.find(op) {
            if idx > 0 {
                let target = formula[..idx].trim().to_lowercase();
                let expr = formula[idx + op.len()..].trim().to_string();
                if !target.is_empty() && !expr.is_empty() {
                    return Some(ParsedFormula {
                        target,
                        op: op.to_string(),
                        expr,
                    });
                }
            }
        }
    }
    None
}

pub fn build_context(
    stats: Option<&serde_json::Value>,
    special_stats: Option<&serde_json::Value>,
    gauges: &[Gauge],
) -> HashMap<String, f64> {
    let mut ctx: HashMap<String, f64> = HashMap::new();

    if let Some(obj) = stats.and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(n) = v.as_f64() {
                ctx.insert(k.to_lowercase(), n);
            }
        }
    }

    if let Some(obj) = special_stats.and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(n) = v.as_f64() {
                ctx.insert(k.to_lowercase(), n);
            }
        }
    }

    for gauge in gauges {
        let name = gauge.name.to_lowercase();
        ctx.insert(name.clone(), gauge.max as f64);
        ctx.insert(format!("{name}.max"), gauge.max as f64);
    }

    ctx
}

pub fn evaluate(expr: &str, ctx: &HashMap<String, f64>) -> Option<f64> {
    let mut keys: Vec<&str> = ctx.keys().map(|s| s.as_str()).collect();
    // longest first to avoid partial substitution (e.g. "health.max" before "health")
    keys.sort_by(|a, b| b.len().cmp(&a.len()));

    let mut resolved = expr.to_string();
    for key in keys {
        let escaped = regex::escape(key);
        let pattern = format!(r"(?i)(?<![a-zA-Z0-9_.]){escaped}(?![a-zA-Z0-9_.])");
        if let Ok(re) = Regex::new(&pattern) {
            resolved = re
                .replace_all(&resolved, ctx[key].to_string())
                .to_string();
        }
    }

    meval::eval_str(&resolved).ok()
}

pub fn compute_effective(
    buffs: &[serde_json::Value],
    target_names: &[&str],
    base_value: f64,
    ctx: &HashMap<String, f64>,
) -> f64 {
    let mut effective = base_value;
    for buff in buffs {
        let formulae_val = buff.get("Formulae").or_else(|| buff.get("formulae"));
        if let Some(formulae) = formulae_val.and_then(|v| v.as_array()) {
            for formula_val in formulae {
                if let Some(formula) = formula_val.as_str() {
                    if let Some(parsed) = parse_formula(formula) {
                        if target_names
                            .iter()
                            .any(|t| t.eq_ignore_ascii_case(&parsed.target))
                        {
                            if let Some(val) = evaluate(&parsed.expr, ctx) {
                                effective = match parsed.op.as_str() {
                                    "+=" => effective + val,
                                    "-=" => effective - val,
                                    "*=" => effective * val,
                                    "/=" => {
                                        if val != 0.0 {
                                            effective / val
                                        } else {
                                            effective
                                        }
                                    }
                                    _ => effective,
                                };
                            }
                        }
                    }
                }
            }
        }
    }
    effective
}
