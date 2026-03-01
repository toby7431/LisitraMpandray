/// Module de base de données — Repository SQLite.
///
/// Réexporte tous les types publics pour que `lib.rs` puisse faire :
/// `use db::{Repository, Member, ...}`
mod error;
mod models;
mod repo;

pub use models::{
    Contribution, ContributionInput, ContributionWithMember,
    Member, MemberInput, MemberWithTotal, YearSummary,
};
pub use repo::Repository;
