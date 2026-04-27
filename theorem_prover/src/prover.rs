use crate::Sequent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofStatus {
    NotImplemented,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofResult {
    pub status: ProofStatus,
}

pub fn prove(_sequent: &Sequent) -> ProofResult {
    ProofResult {
        status: ProofStatus::NotImplemented,
    }
}
