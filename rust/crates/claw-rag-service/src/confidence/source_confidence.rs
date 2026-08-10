use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceConfidence {
    Low,
    Medium,
    High,
    Variable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourceKind {
    Scada,
    Cmms,
    Weather,
    HumanInput,
    KnowledgeBase,
    FaultGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceConfidenceReport {
    pub source: SourceKind,
    pub confidence: EvidenceConfidence,
    pub rule: String,
}

#[must_use]
pub fn confidence_for_source(source: SourceKind) -> SourceConfidenceReport {
    let confidence = match source {
        SourceKind::Scada | SourceKind::Cmms => EvidenceConfidence::High,
        SourceKind::Weather | SourceKind::KnowledgeBase | SourceKind::FaultGraph => {
            EvidenceConfidence::Medium
        }
        SourceKind::HumanInput => EvidenceConfidence::Variable,
    };
    SourceConfidenceReport {
        source,
        confidence,
        rule: format!("{source:?} evidence confidence defaults to {confidence:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scada_and_cmms_are_high_confidence() {
        assert_eq!(
            confidence_for_source(SourceKind::Scada).confidence,
            EvidenceConfidence::High
        );
        assert_eq!(
            confidence_for_source(SourceKind::Cmms).confidence,
            EvidenceConfidence::High
        );
    }
}
