//! What a capacité does, as short texts in our own words, each saying
//! whether the engine computes it (ADR KER-ABILITY-EFFECTS). Which
//! capacités have which effects stays with the engines.

/// Whether the engine computes an effect, or only describes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Application {
    /// The engine counts it in a total of the sheet: a characteristic, a
    /// value (PV, DEF, Init, an attack, DM, dice, critical range, RD, PC,
    /// PM), an option chosen once included. The player need not read it.
    Computed,
    /// No total includes it: a bonus to some tests, a dé bonus, a mastery, a
    /// limit the engine checks, a situational modifier, or what the engine
    /// does not compute. The player reads it to play.
    Described,
}

/// The bonus a described effect gives, computed by the engine, so the
/// sheet can show its amount in place of the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bonus {
    /// The capacité's test bonus numbered `n` from 1, or all of them for 0
    /// (one per characteristic chosen, such as Tatouages).
    Test(u8),
    /// The capacité's dé bonus to tests.
    Die,
}

/// One effect of a capacité: a short French text, never the book's, and
/// whether the engine computes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AbilityEffect {
    text: &'static str,
    application: Application,
    bonus: Option<Bonus>,
}

impl AbilityEffect {
    /// An effect the engine computes.
    #[must_use]
    pub const fn computed(text: &'static str) -> Self {
        Self {
            text,
            application: Application::Computed,
            bonus: None,
        }
    }

    /// An effect the engine only describes.
    #[must_use]
    pub const fn described(text: &'static str) -> Self {
        Self {
            text,
            application: Application::Described,
            bonus: None,
        }
    }

    /// A described effect giving `bonus`, which the engine computes.
    #[must_use]
    pub const fn giving(bonus: Bonus, text: &'static str) -> Self {
        Self {
            text,
            application: Application::Described,
            bonus: Some(bonus),
        }
    }

    /// The text, such as `+3 intimidation`.
    #[must_use]
    pub const fn text(self) -> &'static str {
        self.text
    }

    /// Whether the engine computes it.
    #[must_use]
    pub const fn application(self) -> Application {
        self.application
    }

    /// The bonus it gives, when the engine computes its amount.
    #[must_use]
    pub const fn bonus(self) -> Option<Bonus> {
        self.bonus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_say_whether_they_are_computed() {
        let computed = AbilityEffect::computed("+1 DEF");
        assert_eq!(
            (computed.text(), computed.application()),
            ("+1 DEF", Application::Computed)
        );
        let described = AbilityEffect::described("vision dans le noir 30 m");
        assert_eq!(
            (described.application(), described.bonus()),
            (Application::Described, None)
        );
        let giving = AbilityEffect::giving(Bonus::Test(1), "rang + 2 escalade");
        assert_eq!(
            (giving.application(), giving.bonus()),
            (Application::Described, Some(Bonus::Test(1)))
        );
    }
}
