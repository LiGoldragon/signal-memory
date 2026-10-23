//! Falsifiable witnesses for the [`MemoryOperation`] vocabulary and
//! the [`OperationClass`] classification.
//!
//! If a future change drifts the canonical DOTOS text shape, the
//! rkyv archive layout, or the operation-class mapping, exactly one
//! of these tests breaks and names the variant that drifted.

#[cfg(feature = "dotos-text")]
use dotos::{DotosEncode, DotosSource};
use rkyv::rancor::Error as RkyvError;
use signal_memory::{ArchivedMemoryOperation, MemoryOperation, OperationClass, ToMemoryOperation};

const CANONICAL: &str = include_str!("../examples/canonical.dotos");

fn operations() -> [MemoryOperation; 6] {
    [
        MemoryOperation::Assert,
        MemoryOperation::Mutate,
        MemoryOperation::Retract,
        MemoryOperation::Match,
        MemoryOperation::Subscribe,
        MemoryOperation::Validate,
    ]
}

#[test]
fn memory_operation_record_heads_are_stable() {
    let cases = [
        (MemoryOperation::Assert, "Assert"),
        (MemoryOperation::Mutate, "Mutate"),
        (MemoryOperation::Retract, "Retract"),
        (MemoryOperation::Match, "Match"),
        (MemoryOperation::Subscribe, "Subscribe"),
        (MemoryOperation::Validate, "Validate"),
    ];

    for (operation, head) in cases {
        assert_eq!(operation.as_record_head(), head);
        assert_eq!(MemoryOperation::from_record_head(head), Some(operation));
        assert!(
            CANONICAL.lines().any(|line| line == head),
            "examples/canonical.dotos missing {head}"
        );
    }

    assert_eq!(MemoryOperation::from_record_head("Submit"), None);
}

#[test]
fn memory_operation_classes_are_explicit() {
    assert_eq!(MemoryOperation::Assert.class(), OperationClass::Write);
    assert_eq!(MemoryOperation::Mutate.class(), OperationClass::Write);
    assert_eq!(MemoryOperation::Retract.class(), OperationClass::Write);
    assert_eq!(MemoryOperation::Match.class(), OperationClass::Read);
    assert_eq!(MemoryOperation::Subscribe.class(), OperationClass::Stream);
    assert_eq!(
        MemoryOperation::Validate.class(),
        OperationClass::Validation
    );
}

#[test]
fn memory_operation_is_write_matches_class() {
    for operation in operations() {
        let class = operation.class();
        let expected = matches!(class, OperationClass::Write);
        assert_eq!(
            operation.is_write(),
            expected,
            "is_write disagreed with class for {operation:?} ({class:?})"
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExampleCommand {
    Insert,
    Read,
    OpenStream,
}

impl ToMemoryOperation for ExampleCommand {
    fn to_memory_operation(&self) -> MemoryOperation {
        match self {
            Self::Insert => MemoryOperation::Assert,
            Self::Read => MemoryOperation::Match,
            Self::OpenStream => MemoryOperation::Subscribe,
        }
    }
}

#[test]
fn component_commands_project_to_memory_operation_classes() {
    let cases = [
        (ExampleCommand::Insert, MemoryOperation::Assert),
        (ExampleCommand::Read, MemoryOperation::Match),
        (ExampleCommand::OpenStream, MemoryOperation::Subscribe),
    ];

    for (command, operation) in cases {
        assert_eq!(command.to_memory_operation(), operation);
    }
}

#[test]
fn memory_operation_projects_to_itself() {
    for operation in operations() {
        assert_eq!(operation.to_memory_operation(), operation);
    }
}

#[test]
#[cfg(feature = "dotos-text")]
fn memory_operations_round_trip_through_dotos() {
    for operation in operations() {
        let encoded = operation.to_dotos();
        assert_eq!(encoded, operation.as_record_head());

        let decoded = DotosSource::new(&encoded)
            .parse::<MemoryOperation>()
            .expect("decode");
        assert_eq!(decoded, operation);
    }
}

#[test]
fn memory_operations_round_trip_through_rkyv() {
    for operation in operations() {
        let bytes = rkyv::to_bytes::<RkyvError>(&operation).expect("archive");
        let archived =
            rkyv::access::<ArchivedMemoryOperation, RkyvError>(&bytes).expect("access archive");
        let decoded: MemoryOperation =
            rkyv::deserialize::<MemoryOperation, RkyvError>(archived).expect("deserialize");
        assert_eq!(decoded, operation);
    }
}
