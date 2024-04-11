use color_eyre::eyre::eyre;

pub mod v1;
pub mod v2;

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

impl From<EntityStates> for usize {
    fn from(value: EntityStates) -> Self {
        match value {
            EntityStates::Open => 73,
            EntityStates::Planned => 74,
            EntityStates::InProgress => 75,
            EntityStates::InStaging => 127,
        }
    }
}
