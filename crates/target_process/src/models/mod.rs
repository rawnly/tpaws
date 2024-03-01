use color_eyre::eyre::eyre;

pub mod assignable;
pub mod user;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EntityStates {
    Open = 73,
    Planned = 74,
    InStaging = 127,
    InProgress = 75,
}

impl TryFrom<usize> for EntityStates {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            73 => Ok(Self::Open),
            74 => Ok(Self::Planned),
            75 => Ok(Self::InProgress),
            127 => Ok(Self::InStaging),
            _ => Err(eyre!("unable to parse entityState from usize")),
        }
    }
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
