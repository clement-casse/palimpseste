//! The speciality a capacité asks the player to choose: free text no rule
//! reads. Which capacités ask for one stays with the engines (ADR
//! KER-SPECIALITY).

/// What the player chose for a capacité asking for a speciality: a
/// discipline, an approach, a vehicle, an origin.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Speciality(String);

impl Speciality {
    /// Returns the speciality as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Speciality {
    fn from(speciality: &str) -> Self {
        Self(speciality.to_owned())
    }
}

impl From<String> for Speciality {
    fn from(speciality: String) -> Self {
        Self(speciality)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specialities_convert_from_text() {
        assert_eq!(Speciality::from("Peinture").as_str(), "Peinture");
        assert_eq!(
            Speciality::from("Peinture".to_owned()),
            Speciality::from("Peinture")
        );
    }
}
