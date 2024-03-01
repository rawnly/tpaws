pub mod assignable;
pub mod user;

pub enum EntityStates {
    Open = 73,
    Planned = 74,
    InStaging = 127,
    InProgress = 75,
}

impl Into<usize> for EntityStates {
    fn into(self) -> usize {
        match self {
            Self::Open => 73,
            Self::Planned => 74,
            Self::InProgress => 75,
            Self::InStaging => 127,
        }
    }
}
