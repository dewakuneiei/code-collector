#[derive(Clone, PartialEq, Debug)]
pub enum ThemePreference {
    Dark,
    Light,
    System,
}

impl Default for ThemePreference {
    fn default() -> Self {
        Self::System
    }
}