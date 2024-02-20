pub type SlackUserID = &'static str;
pub type Users = [User; 5];

#[derive(Debug, Clone)]
pub struct User {
    pub id: SlackUserID,
    pub name: &'static str,
}

impl User {
    pub const fn new(name: &'static str, id: &'static str) -> Self {
        Self { id, name }
    }
}

pub const USERS: Users = [
    User::new("Federico Vitale", "U067RQQ94NM"),
    User::new("Tommaso", "U05P5J8H1PG"),
    User::new("Roberto Foti", "U05NXMA9R9D"),
    User::new("Walter", "U125H3R89"),
    User::new("Savina", "U01P6NCUB6X"),
];
