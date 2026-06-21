use std::collections::HashMap;

/// A boolean condition node that can be nested to form logic chains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition {
    /// Looks up a named key in the context; true if the key maps to `true`.
    Fact(String),
    /// True only when every inner condition is true.
    And(Vec<Condition>),
    /// True when at least one inner condition is true.
    Or(Vec<Condition>),
    /// Inverts the result of the inner condition.
    Not(Box<Condition>),
}

/// Evaluates a [`Condition`] tree against a context map.
///
/// The context is a simple `&HashMap<&str, bool>` that supplies the truth
/// value for every named [`Condition::Fact`].  Unknown facts default to
/// `false`.
pub fn evaluate(condition: &Condition, context: &HashMap<&str, bool>) -> bool {
    match condition {
        Condition::Fact(key) => *context.get(key.as_str()).unwrap_or(&false),
        Condition::And(children) => children.iter().all(|c| evaluate(c, context)),
        Condition::Or(children) => children.iter().any(|c| evaluate(c, context)),
        Condition::Not(inner) => !evaluate(inner, context),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(pairs: &[(&'static str, bool)]) -> HashMap<&'static str, bool> {
        pairs.iter().cloned().collect()
    }

    // ── Fact ──────────────────────────────────────────────────────────────────

    #[test]
    fn fact_true_when_context_is_true() {
        let c = ctx(&[("door_open", true)]);
        assert!(evaluate(&Condition::Fact("door_open".into()), &c));
    }

    #[test]
    fn fact_false_when_context_is_false() {
        let c = ctx(&[("door_open", false)]);
        assert!(!evaluate(&Condition::Fact("door_open".into()), &c));
    }

    #[test]
    fn fact_false_when_key_missing() {
        let c = ctx(&[]);
        assert!(!evaluate(&Condition::Fact("missing_key".into()), &c));
    }

    // ── AND ───────────────────────────────────────────────────────────────────

    #[test]
    fn and_true_when_all_children_true() {
        let c = ctx(&[("a", true), ("b", true)]);
        let cond = Condition::And(vec![
            Condition::Fact("a".into()),
            Condition::Fact("b".into()),
        ]);
        assert!(evaluate(&cond, &c));
    }

    #[test]
    fn and_false_when_one_child_false() {
        let c = ctx(&[("a", true), ("b", false)]);
        let cond = Condition::And(vec![
            Condition::Fact("a".into()),
            Condition::Fact("b".into()),
        ]);
        assert!(!evaluate(&cond, &c));
    }

    #[test]
    fn and_true_when_empty() {
        // vacuously true — mirrors Rust's Iterator::all behaviour
        let cond = Condition::And(vec![]);
        assert!(evaluate(&cond, &ctx(&[])));
    }

    // ── OR ────────────────────────────────────────────────────────────────────

    #[test]
    fn or_true_when_at_least_one_child_true() {
        let c = ctx(&[("a", false), ("b", true)]);
        let cond = Condition::Or(vec![
            Condition::Fact("a".into()),
            Condition::Fact("b".into()),
        ]);
        assert!(evaluate(&cond, &c));
    }

    #[test]
    fn or_false_when_all_children_false() {
        let c = ctx(&[("a", false), ("b", false)]);
        let cond = Condition::Or(vec![
            Condition::Fact("a".into()),
            Condition::Fact("b".into()),
        ]);
        assert!(!evaluate(&cond, &c));
    }

    #[test]
    fn or_false_when_empty() {
        // vacuously false — mirrors Iterator::any behaviour
        let cond = Condition::Or(vec![]);
        assert!(!evaluate(&cond, &ctx(&[])));
    }

    // ── NOT ───────────────────────────────────────────────────────────────────

    #[test]
    fn not_inverts_true_to_false() {
        let c = ctx(&[("flag", true)]);
        let cond = Condition::Not(Box::new(Condition::Fact("flag".into())));
        assert!(!evaluate(&cond, &c));
    }

    #[test]
    fn not_inverts_false_to_true() {
        let c = ctx(&[("flag", false)]);
        let cond = Condition::Not(Box::new(Condition::Fact("flag".into())));
        assert!(evaluate(&cond, &c));
    }

    // ── Nested chains ─────────────────────────────────────────────────────────

    #[test]
    fn nested_and_inside_or() {
        // (a AND b) OR c
        let c = ctx(&[("a", true), ("b", false), ("c", true)]);
        let cond = Condition::Or(vec![
            Condition::And(vec![
                Condition::Fact("a".into()),
                Condition::Fact("b".into()),
            ]),
            Condition::Fact("c".into()),
        ]);
        assert!(evaluate(&cond, &c));
    }

    #[test]
    fn nested_or_inside_and() {
        // a AND (b OR c)
        let c = ctx(&[("a", true), ("b", false), ("c", true)]);
        let cond = Condition::And(vec![
            Condition::Fact("a".into()),
            Condition::Or(vec![
                Condition::Fact("b".into()),
                Condition::Fact("c".into()),
            ]),
        ]);
        assert!(evaluate(&cond, &c));
    }

    #[test]
    fn not_wrapping_and() {
        // NOT (a AND b) — a=true, b=false → AND=false → NOT=true
        let c = ctx(&[("a", true), ("b", false)]);
        let cond = Condition::Not(Box::new(Condition::And(vec![
            Condition::Fact("a".into()),
            Condition::Fact("b".into()),
        ])));
        assert!(evaluate(&cond, &c));
    }

    #[test]
    fn deeply_nested_chain() {
        // NOT ( (a OR b) AND (NOT c) )
        // a=false, b=true, c=false → OR=true, NOT c=true → AND=true → NOT=false
        let c = ctx(&[("a", false), ("b", true), ("c", false)]);
        let cond = Condition::Not(Box::new(Condition::And(vec![
            Condition::Or(vec![
                Condition::Fact("a".into()),
                Condition::Fact("b".into()),
            ]),
            Condition::Not(Box::new(Condition::Fact("c".into()))),
        ])));
        assert!(!evaluate(&cond, &c));
    }

    #[test]
    fn deeply_nested_chain_alternate_context() {
        // Same tree, c=true now → NOT c=false → AND=false → NOT=true
        let c = ctx(&[("a", false), ("b", true), ("c", true)]);
        let cond = Condition::Not(Box::new(Condition::And(vec![
            Condition::Or(vec![
                Condition::Fact("a".into()),
                Condition::Fact("b".into()),
            ]),
            Condition::Not(Box::new(Condition::Fact("c".into()))),
        ])));
        assert!(evaluate(&cond, &c));
    }
}
