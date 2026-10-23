//! Falsifiable witnesses for outcome-side Memory classification.

#[cfg(feature = "dotos-text")]
use dotos::{DotosEncode, DotosSource};
use rkyv::rancor::Error as RkyvError;
use signal_memory::{
    ArchivedMemoryObservation, ArchivedMemoryOutcome, MemoryObservation, MemoryOperation,
    MemoryOutcome, ToMemoryOperation, ToMemoryOutcome,
};

const CANONICAL: &str = include_str!("../examples/canonical.dotos");

fn outcomes() -> [MemoryOutcome; 7] {
    [
        MemoryOutcome::Asserted,
        MemoryOutcome::Mutated,
        MemoryOutcome::Retracted,
        MemoryOutcome::Matched,
        MemoryOutcome::Subscribed,
        MemoryOutcome::Validated,
        MemoryOutcome::NoChange,
    ]
}

#[test]
fn memory_observation_projects_to_short_header() {
    let observation = MemoryObservation::new(MemoryOperation::Assert, MemoryOutcome::Asserted);

    assert_eq!(
        observation.log_variant().to_le_bytes(),
        [0, 0, 0, 0, 0, 0, 0, 0]
    );

    let observation = MemoryObservation::new(MemoryOperation::Match, MemoryOutcome::Matched);
    assert_eq!(
        observation.log_variant().to_le_bytes(),
        [3, 3, 1, 0, 0, 0, 0, 0]
    );
}

#[test]
fn memory_outcome_record_heads_are_stable() {
    let cases = [
        (MemoryOutcome::Asserted, "Asserted"),
        (MemoryOutcome::Mutated, "Mutated"),
        (MemoryOutcome::Retracted, "Retracted"),
        (MemoryOutcome::Matched, "Matched"),
        (MemoryOutcome::Subscribed, "Subscribed"),
        (MemoryOutcome::Validated, "Validated"),
        (MemoryOutcome::NoChange, "NoChange"),
    ];

    for (outcome, head) in cases {
        assert_eq!(outcome.as_record_head(), head);
        assert_eq!(MemoryOutcome::from_record_head(head), Some(outcome));
        assert!(
            CANONICAL.lines().any(|line| line == head),
            "examples/canonical.dotos missing {head}"
        );
    }

    assert_eq!(MemoryOutcome::from_record_head("Changed"), None);
}

#[test]
fn memory_operation_projects_to_short_header_root_byte() {
    let cases = [
        (MemoryOperation::Assert, 0),
        (MemoryOperation::Mutate, 1),
        (MemoryOperation::Retract, 2),
        (MemoryOperation::Match, 3),
        (MemoryOperation::Subscribe, 4),
        (MemoryOperation::Validate, 5),
    ];

    for (operation, byte) in cases {
        assert_eq!(
            operation.log_variant().to_le_bytes(),
            [byte, 0, 0, 0, 0, 0, 0, 0]
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExampleCommand {
    Insert,
    Read,
}

impl ToMemoryOperation for ExampleCommand {
    fn to_memory_operation(&self) -> MemoryOperation {
        match self {
            Self::Insert => MemoryOperation::Assert,
            Self::Read => MemoryOperation::Match,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExampleEffect {
    Inserted,
    Read,
}

impl ToMemoryOutcome for ExampleEffect {
    fn to_memory_outcome(&self) -> MemoryOutcome {
        match self {
            Self::Inserted => MemoryOutcome::Asserted,
            Self::Read => MemoryOutcome::Matched,
        }
    }
}

#[test]
fn component_effects_project_to_memory_outcomes() {
    let cases = [
        (ExampleEffect::Inserted, MemoryOutcome::Asserted),
        (ExampleEffect::Read, MemoryOutcome::Matched),
    ];

    for (effect, outcome) in cases {
        assert_eq!(effect.to_memory_outcome(), outcome);
    }
}

#[test]
fn memory_outcome_projects_to_itself() {
    for outcome in outcomes() {
        assert_eq!(outcome.to_memory_outcome(), outcome);
    }
}

#[test]
fn memory_observation_composes_command_and_effect_projection() {
    let asserted =
        MemoryObservation::from_projection(&ExampleCommand::Insert, &ExampleEffect::Inserted);
    let matched = MemoryObservation::from_projection(&ExampleCommand::Read, &ExampleEffect::Read);

    assert_eq!(
        asserted,
        MemoryObservation::new(MemoryOperation::Assert, MemoryOutcome::Asserted)
    );
    assert_eq!(
        matched,
        MemoryObservation::new(MemoryOperation::Match, MemoryOutcome::Matched)
    );
}

#[test]
#[cfg(feature = "dotos-text")]
fn memory_outcomes_round_trip_through_dotos() {
    for outcome in outcomes() {
        let encoded = outcome.to_dotos();
        assert_eq!(encoded, outcome.as_record_head());

        let decoded = DotosSource::new(&encoded)
            .parse::<MemoryOutcome>()
            .expect("decode");
        assert_eq!(decoded, outcome);
    }
}

#[test]
#[cfg(feature = "dotos-text")]
fn memory_observation_round_trips_through_dotos() {
    let observation = MemoryObservation::new(MemoryOperation::Subscribe, MemoryOutcome::Subscribed);
    let encoded = observation.to_dotos();
    let decoded = DotosSource::new(&encoded)
        .parse::<MemoryObservation>()
        .expect("decode");
    assert_eq!(decoded, observation);
}

#[test]
fn memory_outcomes_round_trip_through_rkyv() {
    for outcome in outcomes() {
        let bytes = rkyv::to_bytes::<RkyvError>(&outcome).expect("archive");
        let archived =
            rkyv::access::<ArchivedMemoryOutcome, RkyvError>(&bytes).expect("access archive");
        let decoded: MemoryOutcome =
            rkyv::deserialize::<MemoryOutcome, RkyvError>(archived).expect("deserialize");
        assert_eq!(decoded, outcome);
    }
}

#[test]
fn memory_observation_round_trips_through_rkyv() {
    let observation = MemoryObservation::new(MemoryOperation::Retract, MemoryOutcome::Retracted);
    let bytes = rkyv::to_bytes::<RkyvError>(&observation).expect("archive");
    let archived =
        rkyv::access::<ArchivedMemoryObservation, RkyvError>(&bytes).expect("access archive");
    let decoded: MemoryObservation =
        rkyv::deserialize::<MemoryObservation, RkyvError>(archived).expect("deserialize");
    assert_eq!(decoded, observation);
}
