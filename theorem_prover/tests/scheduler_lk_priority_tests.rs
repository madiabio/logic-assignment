use theorem_prover::ast::{Formula, Symbol, Term, Var};
use theorem_prover::{
    BranchState, Rule, ScheduleResult, ScheduledRule, Sequent, Side,
    record_quantifier_term, quantified_occurrence_key,
};
use theorem_prover::proof::search::scheduler::schedule_next_rules_lk_priority;

fn var(name: &str) -> Var { Var { name: name.to_owned() } }
fn variable(name: &str) -> Term { Term::Var(var(name)) }
fn constant(name: &str) -> Term { Term::Const(Symbol::User(name.to_owned())) }

fn all_rules_are(result: ScheduleResult, pred: impl Fn(&Rule) -> bool) -> Vec<ScheduledRule> {
    let ScheduleResult::Rules(rules) = result else { panic!("expected ScheduleResult::Rules") };
    assert!(!rules.is_empty());
    for rule in &rules {
        let r = match rule {
            ScheduledRule::Standard(rm) => &rm.rule,
            ScheduledRule::ForAllL { rule_match, .. } => &rule_match.rule,
            ScheduledRule::ExistsR { rule_match, .. } => &rule_match.rule,
        };
        assert!(pred(r), "unexpected rule: {r:?}");
    }
    rules
}

#[test]
fn class1_closing_beats_class2_non_branching_propositional() {
    let sequent = Sequent {
        left: vec![
            Formula::and(vec![Formula::atom("a"), Formula::atom("b")]),
            Formula::atom("p"),
        ],
        right: vec![Formula::atom("p")],
    };
    let result = schedule_next_rules_lk_priority(&sequent, &BranchState::new(), 1);
    all_rules_are(result, |r| matches!(r, Rule::Id | Rule::TopR | Rule::BottomL));
}

#[test]
fn class2_non_branching_beats_class3_branching_propositional() {
    let sequent = Sequent {
        left: vec![Formula::and(vec![Formula::atom("a"), Formula::atom("b")])],
        right: vec![Formula::and(vec![Formula::atom("c"), Formula::atom("d")])],
    };
    let result = schedule_next_rules_lk_priority(&sequent, &BranchState::new(), 1);
    all_rules_are(result, |r| {
        matches!(r, Rule::AndL | Rule::OrR | Rule::ImpliesR | Rule::NotL | Rule::NotR)
    });
}

#[test]
fn class3_branching_beats_class4_eigenvariable_quantifiers() {
    let sequent = Sequent {
        left: vec![Formula::or(vec![Formula::atom("a"), Formula::atom("b")])],
        right: vec![Formula::ForAll(
            vec![var("X")],
            Box::new(Formula::predicate("p", vec![variable("X")])),
        )],
    };
    let result = schedule_next_rules_lk_priority(&sequent, &BranchState::new(), 1);
    all_rules_are(result, |r| matches!(r, Rule::AndR | Rule::OrL | Rule::ImpliesL));
}

#[test]
fn class4_eigenvariable_beats_class5_visible_term_reusable() {
    let sequent = Sequent {
        left: vec![
            Formula::ForAll(
                vec![var("X")],
                Box::new(Formula::predicate("p", vec![variable("X")])),
            ),
            Formula::predicate("q", vec![constant("a")]),
        ],
        right: vec![Formula::ForAll(
            vec![var("Z")],
            Box::new(Formula::predicate("r", vec![variable("Z")])),
        )],
    };
    let result = schedule_next_rules_lk_priority(&sequent, &BranchState::new(), 1);
    all_rules_are(result, |r| matches!(r, Rule::ForAllR | Rule::ExistsL));
}

#[test]
fn class5_visible_term_beats_class6_fresh_fallback() {
    let term_a = constant("a");
    let sequent = Sequent {
        left: vec![
            Formula::ForAll(
                vec![var("X")],
                Box::new(Formula::predicate("p", vec![variable("X")])),
            ),
            Formula::predicate("q", vec![term_a.clone()]),
        ],
        right: vec![Formula::atom("goal")],
    };
    let result = schedule_next_rules_lk_priority(&sequent, &BranchState::new(), 1);
    let ScheduleResult::Rules(rules) = result else { panic!("expected Rules") };
    assert!(rules.iter().any(|r| matches!(r,
        ScheduledRule::ForAllL { term, fresh_fallback: false, .. } if term == &term_a
    )));
    assert!(!rules.iter().any(|r| matches!(r, ScheduledRule::ForAllL { fresh_fallback: true, .. })));
}

#[test]
fn class6_fresh_fallback_scheduled_when_all_visible_terms_used() {
    let term_a = constant("a");
    let quantified = Formula::ForAll(
        vec![var("X")],
        Box::new(Formula::predicate("p", vec![variable("X")])),
    );
    let sequent = Sequent {
        left: vec![quantified, Formula::predicate("q", vec![term_a.clone()])],
        right: vec![Formula::atom("goal")],
    };
    let mut state = BranchState::new();
    let key = quantified_occurrence_key(&sequent, Side::Left, 0)
        .expect("quantified formula should produce a key");
    record_quantifier_term(&mut state, &key, &term_a, false);

    let result = schedule_next_rules_lk_priority(&sequent, &state, 1);
    let ScheduleResult::Rules(rules) = result else { panic!("expected Rules") };
    assert!(rules.iter().any(|r| matches!(r, ScheduledRule::ForAllL { fresh_fallback: true, .. })));
}

#[test]
fn reports_quantifier_exhausted_when_budget_spent() {
    let term_w = constant("w");
    let quantified = Formula::Exists(
        vec![var("X")],
        Box::new(Formula::predicate("p", vec![variable("X")])),
    );
    let sequent = Sequent {
        left: Vec::new(),
        right: vec![quantified, Formula::predicate("p", vec![term_w.clone()])],
    };
    let mut state = BranchState::new();
    let key = quantified_occurrence_key(&sequent, Side::Right, 0)
        .expect("quantified formula should produce a key");
    record_quantifier_term(&mut state, &key, &term_w, true);

    let result = schedule_next_rules_lk_priority(&sequent, &state, 1);
    assert!(matches!(result, ScheduleResult::QuantifierExhausted));
}
