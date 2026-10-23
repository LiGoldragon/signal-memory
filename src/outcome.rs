//! Outcome-side Memory classification.
//!
//! Component daemons execute component-local commands and produce
//! component-local effects. [`MemoryOutcome`] is the universal,
//! payloadless classification of those effects for observation.

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::{MemoryOperation, ToMemoryOperation};

/// The closed outcome set visible to generic Memory observers.
///
/// Outcomes do not carry executable or domain payloads. Component
/// events carry those details separately; this enum classifies what
/// happened in the broad workspace vocabulary.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "dotos-text", derive(dotos::DotosEncode, dotos::DotosDecode))]
pub enum MemoryOutcome {
    /// A new typed fact / event / row was appended.
    Asserted,
    /// An existing typed record transitioned at stable identity.
    Mutated,
    /// A typed record was tombstoned, removed, or retracted.
    Retracted,
    /// Typed records were read by key, range, pattern, or plan.
    Matched,
    /// A state-plus-delta observation stream was opened.
    Subscribed,
    /// A dry-run validation or planning request completed.
    Validated,
    /// The request completed without changing observable state.
    NoChange,
}

impl MemoryOutcome {
    /// PascalCase DOTOS record-head identifier for this outcome.
    pub const fn as_record_head(self) -> &'static str {
        match self {
            Self::Asserted => "Asserted",
            Self::Mutated => "Mutated",
            Self::Retracted => "Retracted",
            Self::Matched => "Matched",
            Self::Subscribed => "Subscribed",
            Self::Validated => "Validated",
            Self::NoChange => "NoChange",
        }
    }

    /// Parse a PascalCase DOTOS record-head identifier back to a
    /// [`MemoryOutcome`].
    pub fn from_record_head(name: &str) -> Option<Self> {
        match name {
            "Asserted" => Some(Self::Asserted),
            "Mutated" => Some(Self::Mutated),
            "Retracted" => Some(Self::Retracted),
            "Matched" => Some(Self::Matched),
            "Subscribed" => Some(Self::Subscribed),
            "Validated" => Some(Self::Validated),
            "NoChange" => Some(Self::NoChange),
            _ => None,
        }
    }

    /// Stable second-byte classification code for compact observations.
    pub const fn log_variant(self) -> u64 {
        let byte = match self {
            Self::Asserted => 0,
            Self::Mutated => 1,
            Self::Retracted => 2,
            Self::Matched => 3,
            Self::Subscribed => 4,
            Self::Validated => 5,
            Self::NoChange => 6,
        };
        byte as u64
    }
}

/// Projection from a component-local effect into the universal Memory
/// outcome classification vocabulary.
pub trait ToMemoryOutcome {
    fn to_memory_outcome(&self) -> MemoryOutcome;
}

impl ToMemoryOutcome for MemoryOutcome {
    fn to_memory_outcome(&self) -> MemoryOutcome {
        *self
    }
}

/// Universal observation label produced from a component command and
/// the effect it caused.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "dotos-text", derive(dotos::DotosEncode, dotos::DotosDecode))]
pub struct MemoryObservation {
    pub operation: MemoryOperation,
    pub outcome: MemoryOutcome,
}

impl MemoryObservation {
    pub const fn new(operation: MemoryOperation, outcome: MemoryOutcome) -> Self {
        Self { operation, outcome }
    }

    pub fn from_projection<Command, Effect>(command: &Command, effect: &Effect) -> Self
    where
        Command: ToMemoryOperation,
        Effect: ToMemoryOutcome,
    {
        Self::new(command.to_memory_operation(), effect.to_memory_outcome())
    }

    /// Stable compact observation code: operation byte, outcome byte, class byte.
    pub const fn log_variant(&self) -> u64 {
        let operation = self.operation.log_variant();
        let outcome = self.outcome.log_variant();
        let class = match self.operation.class() {
            crate::OperationClass::Write => 0,
            crate::OperationClass::Read => 1,
            crate::OperationClass::Stream => 2,
            crate::OperationClass::Validation => 3,
        };
        operation | (outcome << 8) | ((class as u64) << 16)
    }
}
