//! Items carried: the name of an item no rule reads, and a quantity. Which
//! items are content stays with the engines (ADR ENG-ITEMS).

/// The name of a free text item.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemName(String);

impl ItemName {
    /// Returns the name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ItemName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl From<String> for ItemName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

/// How many of an item the character carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quantity(u32);

impl Quantity {
    /// Creates a quantity.
    #[must_use]
    pub const fn new(quantity: u32) -> Self {
        Self(quantity)
    }

    /// Returns the quantity.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for Quantity {
    fn from(quantity: u32) -> Self {
        Self(quantity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_quantities_convert_from_primitives() {
        assert_eq!(ItemName::from("rope").as_str(), "rope");
        assert_eq!(ItemName::from("rope".to_owned()), ItemName::from("rope"));
        assert_eq!(Quantity::from(3), Quantity::new(3));
        assert_eq!(Quantity::new(u32::MAX).get(), u32::MAX);
    }
}
