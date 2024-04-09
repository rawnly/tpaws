use serde::Deserialize;

// {id,name,entityState,team,description,assignedUser,resourceType}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Assignable {
    pub id: usize,
    pub name: String,
    pub resource_type: String,
    pub description: Option<String>,
    pub entity_type: EntityType,
    pub entity_state: EntityState,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdAndName {
    pub id: usize,
    pub name: String,
}

pub type EntityState = IdAndName;
pub type EntityType = IdAndName;

impl Into<crate::models::v1::assignable::IdAndName> for IdAndName {
    fn into(self) -> crate::models::v1::assignable::IdAndName {
        crate::models::v1::assignable::IdAndName {
            id: self.id,
            name: self.name,
        }
    }
}

#[macro_export]
macro_rules! id_getter {
    ($($struct:ident),+ ) => {
        $(
            impl $struct {
                pub fn id(&self) -> usize {
                    self.id
                }
            }

            impl PartialEq for $struct {
                fn eq(&self, other: &Self) -> bool {
                    self.id() == other.id()
                }
            }
        )*
    };
}

id_getter!(IdAndName);

#[cfg(test)]
mod test {
    #[test]
    fn id_getter() {
        #[derive(Debug)]
        struct Person {
            pub id: usize,
        }
        id_getter!(Person);

        let p1 = Person { id: 1 };
        let p2 = Person { id: 1 };

        assert_eq!(p1, p2);
        assert_eq!(p1.id(), p2.id());
    }
}
