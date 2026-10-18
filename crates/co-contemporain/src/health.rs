//! The échelle de santé (COC.DER.HEALTH-SCALE, COC.CNT.SCALES): how many
//! steps it has, and the states damage reaches on it. The engine computes no
//! play state: the damage taken is counted by whoever displays the scale.

use co_kernel::ValueId;

/// The steps of a full échelle de santé. A shorter scale has its first
/// steps permanently blackened (COC.DER.HEALTH-SCALE).
pub const HEALTH_SCALE_STEPS: i32 = 20;

/// A state of the échelle de santé, reached when damage fills its step
/// (COC.CNT.SCALES). Only the worst state reached applies, and a state whose
/// step is blackened never does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HealthState {
    /// Contusionné.
    Contusionne,
    /// Affaibli.
    Affaibli,
    /// Blessé.
    Blesse,
    /// Mourant.
    Mourant,
}

impl HealthState {
    /// Every state, from the first step reached.
    pub const ALL: [Self; 4] = [
        Self::Contusionne,
        Self::Affaibli,
        Self::Blesse,
        Self::Mourant,
    ];

    /// Returns the kernel identifier of the state.
    #[must_use]
    pub const fn id(self) -> ValueId {
        ValueId::from_static(match self {
            Self::Contusionne => "coc.health-state.contusionne",
            Self::Affaibli => "coc.health-state.affaibli",
            Self::Blesse => "coc.health-state.blesse",
            Self::Mourant => "coc.health-state.mourant",
        })
    }

    /// The French display name of the state.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Contusionne => "Contusionné",
            Self::Affaibli => "Affaibli",
            Self::Blesse => "Blessé",
            Self::Mourant => "Mourant",
        }
    }

    /// The step of the scale whose filling reaches the state.
    #[must_use]
    pub const fn step(self) -> i32 {
        match self {
            Self::Contusionne => 5,
            Self::Affaibli => 10,
            Self::Blesse => 15,
            Self::Mourant => 20,
        }
    }

    /// The malus to Init, DEF and physical actions, as a negative amount.
    #[must_use]
    pub const fn malus(self) -> i32 {
        match self {
            Self::Contusionne => -1,
            Self::Affaibli => -3,
            Self::Blesse => -5,
            Self::Mourant => -10,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn states_worsen_along_the_scale() {
        for pair in HealthState::ALL.windows(2) {
            assert!(pair[0].step() < pair[1].step());
            assert!(pair[0].malus() > pair[1].malus());
        }
        assert!(
            HealthState::ALL
                .iter()
                .all(|state| (1..=HEALTH_SCALE_STEPS).contains(&state.step()))
        );
        assert_eq!(HealthState::Mourant.step(), HEALTH_SCALE_STEPS);
    }

    #[test]
    fn identifiers_are_distinct() {
        let ids: BTreeSet<_> = HealthState::ALL
            .iter()
            .map(|state| state.id().as_str().to_owned())
            .collect();
        assert_eq!(ids.len(), HealthState::ALL.len());
    }
}
