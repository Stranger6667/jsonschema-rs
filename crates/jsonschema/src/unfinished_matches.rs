//! Regex matches the engine could not finish, such as on reaching the backtrack limit.
//!
//! The real outcome of such a match is either `true` or `false`, so validation runs again under
//! each combination of outcomes, and only what every combination agrees on is reported.
use std::borrow::Cow;

use crate::{
    paths::{LazyEvaluationPath, LazyLocation, Location, RefTracker},
    regex::{RegexError, RegexFailureReason},
    LazyInstance, ValidationError,
};

/// Runs past which the outcome of unfinished regex matches is reported as undecided.
const MAX_ASSUMED_RUNS: usize = 64;

#[derive(Clone, PartialEq, Eq)]
struct UnfinishedMatch {
    /// The compiled pattern, so every keyword sharing it assumes the same outcome.
    pattern: Box<str>,
    subject: Box<str>,
}

#[derive(Clone)]
struct AssumedMatch {
    unfinished: UnfinishedMatch,
    matches: bool,
}

/// Where a regex match happens, for reporting it when the engine cannot finish it.
pub(crate) struct MatchSite {
    schema_path: Location,
    evaluation_path: LazyEvaluationPath,
    /// `None` where validation does not track the instance location, as in `is_valid`.
    instance_path: Option<Location>,
    /// The `pattern` keyword's source pattern.
    pattern: Option<String>,
}

impl MatchSite {
    pub(crate) fn schema(schema_path: &Location) -> Self {
        MatchSite {
            schema_path: schema_path.clone(),
            evaluation_path: LazyEvaluationPath::SameAsSchemaPath,
            instance_path: None,
            pattern: None,
        }
    }

    pub(crate) fn located(
        schema_path: &Location,
        tracker: Option<&RefTracker>,
        instance_path: &LazyLocation,
    ) -> Self {
        MatchSite {
            schema_path: schema_path.clone(),
            evaluation_path: crate::paths::capture_evaluation_path(tracker, schema_path),
            instance_path: Some(instance_path.into()),
            pattern: None,
        }
    }

    /// A site in generated code, whose schema path arrives escaped.
    #[cfg(feature = "macros")]
    pub(crate) fn escaped(schema_path: &str, instance_path: Option<Location>) -> Self {
        MatchSite {
            schema_path: Location::from_escaped(schema_path),
            evaluation_path: LazyEvaluationPath::SameAsSchemaPath,
            instance_path,
            pattern: None,
        }
    }

    pub(crate) fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(pattern.to_owned());
        self
    }

    /// Takes `other`'s location when only `other` knows the instance location.
    fn prefer_located(&mut self, other: MatchSite) {
        if self.instance_path.is_none() && other.instance_path.is_some() {
            let pattern = other.pattern.or(self.pattern.take());
            *self = MatchSite { pattern, ..other };
        }
    }
}

pub(crate) struct MatchFailure {
    reason: RegexFailureReason,
    site: MatchSite,
    /// The compiled pattern, named when the site has no `pattern` keyword source.
    compiled: Box<str>,
}

impl MatchFailure {
    fn into_error<'i>(self, root: impl FnOnce() -> LazyInstance<'i>) -> ValidationError<'i> {
        let MatchSite {
            schema_path,
            evaluation_path,
            instance_path,
            pattern,
        } = self.site;
        let root = root().into_cow();
        let (instance_path, instance) = match instance_path {
            Some(path) => {
                let instance = match root {
                    Cow::Borrowed(root) => Cow::Borrowed(
                        root.pointer(path.as_str())
                            .expect("the match site is inside the instance"),
                    ),
                    Cow::Owned(root) => Cow::Owned(
                        root.pointer(path.as_str())
                            .expect("the match site is inside the instance")
                            .clone(),
                    ),
                };
                (path, instance)
            }
            None => (Location::new(), root),
        };
        let instance = LazyInstance::Ready(instance);
        match self.reason {
            RegexFailureReason::FancyRegex(error) => ValidationError::backtrack_limit(
                schema_path,
                evaluation_path,
                instance_path,
                instance,
                error,
            ),
            RegexFailureReason::Panicked => ValidationError::regex_engine_failure(
                schema_path,
                evaluation_path,
                instance_path,
                instance,
                format!(
                    "Regex engine failed to evaluate pattern '{}'",
                    pattern.as_deref().unwrap_or(&self.compiled)
                ),
            ),
        }
    }
}

/// Outcomes one run assumes for unfinished matches, and the first one it met without an outcome.
#[derive(Default)]
pub(crate) struct MatchAssumptions {
    assumed: Vec<AssumedMatch>,
    unassumed: Option<Unassumed>,
    /// The match the error reports, and where this run met it with its instance location.
    reported: Option<UnfinishedMatch>,
    reported_site: Option<MatchSite>,
}

struct Unassumed {
    unfinished: UnfinishedMatch,
    failure: MatchFailure,
}

impl MatchAssumptions {
    /// The outcome this run assumes for a match the engine could not finish, or `false` while
    /// recording it for the next run to assume.
    #[cold]
    pub(crate) fn resolve(
        &mut self,
        subject: &str,
        error: impl RegexError,
        site: impl FnOnce() -> MatchSite,
    ) -> bool {
        if let Some(assumed) = self.assumed.iter().find(|assumed| {
            &*assumed.unfinished.pattern == error.pattern()
                && &*assumed.unfinished.subject == subject
        }) {
            if self.reported_site.is_none() && self.reported.as_ref() == Some(&assumed.unfinished) {
                let site = site();
                if site.instance_path.is_some() {
                    self.reported_site = Some(site);
                }
            }
            return assumed.matches;
        }
        match &mut self.unassumed {
            None => {
                self.unassumed = Some(Unassumed {
                    unfinished: UnfinishedMatch {
                        pattern: error.pattern().into(),
                        subject: subject.into(),
                    },
                    failure: MatchFailure {
                        compiled: error.pattern().into(),
                        reason: error.into_failure_reason(),
                        site: site(),
                    },
                });
            }
            // `is_valid` may meet the match before `validate` meets it again with its instance.
            Some(unassumed)
                if unassumed.failure.site.instance_path.is_none()
                    && &*unassumed.unfinished.pattern == error.pattern()
                    && &*unassumed.unfinished.subject == subject =>
            {
                unassumed.failure.site.prefer_located(site());
            }
            Some(_) => {}
        }
        false
    }
}

pub(crate) enum Outcomes<T> {
    /// One result per combination of outcomes for the unfinished matches.
    Assumed {
        results: Vec<T>,
        failure: MatchFailure,
    },
    /// Too many combinations to try.
    Undecided(MatchFailure),
}

/// Runs `run` under each combination of outcomes for the regex matches the engine could not
/// finish, starting from the first one a run met. The real outcome is one of the combinations, so
/// results shared by all are exact.
///
/// `run` receives the outcomes to assume and leaves the ones it met.
#[cold]
pub(crate) fn assume_each_outcome<T>(
    mut run: impl FnMut(&mut Option<Box<MatchAssumptions>>) -> T,
    first: MatchAssumptions,
) -> Outcomes<T> {
    let Unassumed {
        unfinished,
        mut failure,
    } = first
        .unassumed
        .expect("assumptions exist once a run meets an unfinished match");
    let mut pending = Vec::new();
    push_both_outcomes(&mut pending, &[], &unfinished);
    let mut results = Vec::new();
    let mut runs = 1;
    while let Some(assumed) = pending.pop() {
        if runs == MAX_ASSUMED_RUNS {
            return Outcomes::Undecided(failure);
        }
        runs += 1;
        // A run may meet the match with its instance location where the first run met it only
        // through `is_valid`, e.g. when `validate` returned an earlier error first.
        let mut slot = Some(Box::new(MatchAssumptions {
            assumed,
            reported: Some(unfinished.clone()),
            ..MatchAssumptions::default()
        }));
        let result = run(&mut slot);
        let mut matches = slot.expect("a run keeps its assumptions");
        if let Some(site) = matches.reported_site.take() {
            failure.site.prefer_located(site);
        }
        match matches.unassumed.take() {
            None => results.push(result),
            Some(next) => push_both_outcomes(&mut pending, &matches.assumed, &next.unfinished),
        }
    }
    Outcomes::Assumed { results, failure }
}

fn push_both_outcomes(
    pending: &mut Vec<Vec<AssumedMatch>>,
    assumed: &[AssumedMatch],
    unfinished: &UnfinishedMatch,
) {
    for matches in [true, false] {
        let mut next = assumed.to_vec();
        next.push(AssumedMatch {
            unfinished: unfinished.clone(),
            matches,
        });
        pending.push(next);
    }
}

/// Valid only when every outcome is valid.
pub(crate) fn validity(outcomes: Outcomes<bool>) -> bool {
    match outcomes {
        Outcomes::Assumed { results, .. } => results.into_iter().all(|valid| valid),
        Outcomes::Undecided(_) => false,
    }
}

/// Errors present under every outcome, or the engine failure when the outcomes disagree on
/// validity or share no error.
pub(crate) fn errors<'i>(
    outcomes: Outcomes<Vec<ValidationError<'i>>>,
    instance: impl FnOnce() -> LazyInstance<'i>,
) -> Vec<ValidationError<'i>> {
    match outcomes {
        Outcomes::Assumed { results, failure } => {
            shared_errors(results).unwrap_or_else(|| vec![failure.into_error(instance)])
        }
        Outcomes::Undecided(failure) => vec![failure.into_error(instance)],
    }
}

fn shared_errors(mut results: Vec<Vec<ValidationError<'_>>>) -> Option<Vec<ValidationError<'_>>> {
    let mut shared = results.pop().unwrap_or_default();
    if shared.is_empty() {
        return results.iter().all(Vec::is_empty).then_some(shared);
    }
    shared.retain(|error| {
        results
            .iter()
            .all(|errors| errors.iter().any(|other| same_error(error, other)))
    });
    (!shared.is_empty()).then_some(shared)
}

fn same_error(left: &ValidationError<'_>, right: &ValidationError<'_>) -> bool {
    left.instance_path() == right.instance_path()
        && left.schema_path() == right.schema_path()
        && left.evaluation_path() == right.evaluation_path()
        && left.to_string() == right.to_string()
}

#[cfg(test)]
mod tests {
    use crate::PatternOptions;
    use serde_json::{json, Value};
    use test_case::test_case;

    // With `backtrack_limit(1)` the engine cannot finish matching these against "abc":
    // `(?<=ab)c` would match it and `(?<=x)c` would not.
    const MATCHES: &str = "(?<=ab)c";
    const MISSES: &str = "(?<=x)c";
    const UNDECIDED: &str = "Error executing regex: Max limit for backtracking count exceeded";

    type Outcomes = (bool, Result<(), String>, Vec<String>, (bool, Vec<String>));

    fn validator(schema: &Value) -> crate::Validator {
        crate::options()
            .with_pattern_options(PatternOptions::fancy_regex().backtrack_limit(1))
            .build(schema)
            .expect("schema compiles")
    }

    fn outcomes(schema: &Value, instance: &Value) -> Outcomes {
        let validator = validator(schema);
        let evaluation = validator.evaluate(instance);
        (
            validator.is_valid(instance),
            validator
                .validate(instance)
                .map_err(|error| error.to_string()),
            validator
                .iter_errors(instance)
                .map(|error| error.to_string())
                .collect(),
            (
                evaluation.is_valid(),
                evaluation
                    .iter_errors()
                    .map(|entry| entry.error.to_string())
                    .collect(),
            ),
        )
    }

    fn undecided() -> Outcomes {
        (
            false,
            Err(UNDECIDED.to_owned()),
            vec![UNDECIDED.to_owned()],
            (false, vec![UNDECIDED.to_owned()]),
        )
    }

    fn valid() -> Outcomes {
        (true, Ok(()), Vec::new(), (true, Vec::new()))
    }

    #[test_case(
        &json!({"not": {"pattern": MATCHES}}),
        &json!({"valid": false, "details": [
            {"valid": false, "evaluationPath": "", "schemaLocation": "", "instanceLocation": ""},
            {
                "valid": false,
                "evaluationPath": "/not/pattern",
                "schemaLocation": "/not/pattern",
                "instanceLocation": "",
                "errors": {"pattern": UNDECIDED}
            }
        ]}) ;
        "undecided"
    )]
    #[test_case(
        &json!({"pattern": MATCHES, "maxLength": 1}),
        &json!({"valid": false, "details": [
            {"valid": false, "evaluationPath": "", "schemaLocation": "", "instanceLocation": ""},
            {
                "valid": false,
                "evaluationPath": "/maxLength",
                "schemaLocation": "/maxLength",
                "instanceLocation": "",
                "errors": {"maxLength": "\"abc\" is longer than 1 character"}
            }
        ]}) ;
        "shared error"
    )]
    #[test_case(
        &json!({"anyOf": [{"pattern": MATCHES}, {"type": "string"}]}),
        &json!({"valid": true, "details": [
            {"valid": true, "evaluationPath": "", "schemaLocation": "", "instanceLocation": ""}
        ]}) ;
        "valid under every outcome"
    )]
    fn unfinished_match_evaluation_keeps_only_what_every_outcome_shares(
        schema: &Value,
        expected: &Value,
    ) {
        let evaluation = validator(schema).evaluate(&json!("abc"));
        assert_eq!(
            &serde_json::to_value(evaluation.list()).expect("serializes"),
            expected
        );
    }

    #[test_case(&json!({"not": {"pattern": MATCHES}}), &json!("abc") ; "not over a match")]
    #[test_case(&json!({"not": {"pattern": MISSES}}), &json!("abc") ; "not over a miss")]
    #[test_case(&json!({"if": {"pattern": MATCHES}, "then": {"maxLength": 1}}), &json!("abc") ; "if picks the branch")]
    #[test_case(&json!({"oneOf": [{"pattern": MATCHES}, {"type": "string"}]}), &json!("abc") ; "oneOf counts the branch")]
    #[test_case(&json!({"patternProperties": {MATCHES: {"type": "integer"}}}), &json!({"abc": "x"}) ; "patternProperties applies the subschema")]
    #[test_case(
        &json!({"patternProperties": {MATCHES: {"type": "integer"}, "^z": true}}),
        &json!({"abc": "x"}) ;
        "patternProperties with several patterns"
    )]
    #[test_case(
        &json!({"patternProperties": {MATCHES: true}, "additionalProperties": false}),
        &json!({"abc": 1}) ;
        "additionalProperties excludes the key"
    )]
    #[test_case(
        &json!({"patternProperties": {MATCHES: {"type": "integer"}}, "additionalProperties": {"type": "string"}}),
        &json!({"abc": 1}) ;
        "additionalProperties schema"
    )]
    #[test_case(
        &json!({"properties": {"abc": true}, "patternProperties": {MATCHES: {"type": "integer"}}, "additionalProperties": false}),
        &json!({"abc": "x"}) ;
        "patternProperties over a declared property with additionalProperties false"
    )]
    #[test_case(
        &json!({"properties": {"abc": true}, "patternProperties": {MATCHES: {"type": "integer"}}, "additionalProperties": {}}),
        &json!({"abc": "x"}) ;
        "patternProperties over a declared property with additionalProperties schema"
    )]
    #[test_case(
        &json!({"properties": {"q": true}, "patternProperties": {MATCHES: true}, "additionalProperties": false}),
        &json!({"abc": 1}) ;
        "undeclared property with additionalProperties false"
    )]
    #[test_case(
        &json!({
            "properties": {"q": true},
            "patternProperties": {MATCHES: {"type": "integer"}},
            "additionalProperties": {"type": "string"}
        }),
        &json!({"abc": 1}) ;
        "undeclared property with additionalProperties schema"
    )]
    #[test_case(
        &json!({"patternProperties": {MATCHES: true}, "unevaluatedProperties": false}),
        &json!({"abc": 1}) ;
        "unevaluatedProperties excludes the key"
    )]
    fn unfinished_match_that_decides_the_outcome_is_an_error(schema: &Value, instance: &Value) {
        assert_eq!(outcomes(schema, instance), undecided());
    }

    #[test_case(&json!({"anyOf": [{"pattern": MATCHES}, {"type": "string"}]}) ; "anyOf with an accepting branch")]
    #[test_case(&json!({"anyOf": [{"pattern": MATCHES}, {"not": {"pattern": MATCHES}}]}) ; "anyOf over a match and its negation")]
    #[test_case(
        &json!({"if": {"pattern": MATCHES}, "then": {"minLength": 1}, "else": {"minLength": 1}}) ;
        "if with equal branches"
    )]
    fn unfinished_match_that_does_not_decide_the_outcome_is_ignored(schema: &Value) {
        assert_eq!(outcomes(schema, &json!("abc")), valid());
    }

    #[test]
    fn unfinished_match_keeps_the_errors_every_outcome_shares() {
        let schema = json!({"pattern": MATCHES, "maxLength": 1});
        let error = r#""abc" is longer than 1 character"#.to_owned();
        assert_eq!(
            outcomes(&schema, &json!("abc")),
            (
                false,
                Err(error.clone()),
                vec![error.clone()],
                (false, vec![error])
            )
        );
    }

    #[test]
    fn too_many_unfinished_matches_is_an_error() {
        let branches: Vec<Value> = "defghijk"
            .chars()
            .map(|letter| json!({"anyOf": [{"pattern": format!("(?<={letter})c")}, {"type": "string"}]}))
            .collect();
        assert_eq!(
            outcomes(&json!({"allOf": branches}), &json!("abc")),
            undecided()
        );
    }

    #[test_case(
        &json!({"properties": {"x": {"pattern": MATCHES}}}),
        &json!({"x": "abc"}),
        ("/properties/x/pattern", "/properties/x/pattern", "/x", json!("abc")) ;
        "pattern"
    )]
    #[test_case(
        &json!({"$defs": {"s": {"pattern": MATCHES}}, "properties": {"x": {"$ref": "#/$defs/s"}}}),
        &json!({"x": "abc"}),
        ("/$defs/s/pattern", "/properties/x/$ref/pattern", "/x", json!("abc")) ;
        "pattern behind a reference"
    )]
    #[test_case(
        &json!({"properties": {"o": {"patternProperties": {MATCHES: {"type": "integer"}}}}}),
        &json!({"o": {"abc": "x"}}),
        (
            "/properties/o/patternProperties/(?<=ab)c",
            "/properties/o/patternProperties/(?<=ab)c",
            "/o",
            json!({"abc": "x"}),
        ) ;
        "patternProperties"
    )]
    #[test_case(
        &json!({"properties": {"o": {"patternProperties": {MATCHES: true}, "additionalProperties": false}}}),
        &json!({"o": {"abc": 1}}),
        (
            "/properties/o/patternProperties/(?<=ab)c",
            "/properties/o/patternProperties/(?<=ab)c",
            "/o",
            json!({"abc": 1}),
        ) ;
        "additionalProperties"
    )]
    #[test_case(
        &json!({"properties": {"o": {"patternProperties": {MATCHES: true}, "unevaluatedProperties": false}}}),
        &json!({"o": {"abc": 1}}),
        (
            "/properties/o/patternProperties/(?<=ab)c",
            "/properties/o/patternProperties/(?<=ab)c",
            "/o",
            json!({"abc": 1}),
        ) ;
        "unevaluatedProperties"
    )]
    #[test_case(
        &json!({"properties": {"a": {"not": {"not": {"pattern": MATCHES}}}, "b": {"pattern": MATCHES}}}),
        &json!({"a": "abc", "b": "abc"}),
        (
            "/properties/b/pattern",
            "/properties/b/pattern",
            "/b",
            json!("abc"),
        ) ;
        "pattern met with its instance only under an assumed outcome"
    )]
    fn unfinished_match_error_points_at_the_match(
        schema: &Value,
        instance: &Value,
        expected: (&str, &str, &str, Value),
    ) {
        let error = validator(schema).validate(instance).expect_err("undecided");
        assert_eq!(
            (
                error.schema_path().to_string(),
                error.evaluation_path().to_string(),
                error.instance_path().to_string(),
                error.instance().clone().into_owned(),
                error.to_string(),
            ),
            (
                expected.0.to_owned(),
                expected.1.to_owned(),
                expected.2.to_owned(),
                expected.3,
                UNDECIDED.to_owned(),
            )
        );
    }
}
