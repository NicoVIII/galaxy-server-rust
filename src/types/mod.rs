use std::cmp::Ordering;
use std::collections::HashSet;

type Id = u16;
pub type Offset = u16;

#[derive(Eq)]
pub struct Position(pub Offset, pub Offset);

struct Dot<'a> {
    fields: HashSet<&'a Field<'a>>,
    position: Position,
}

struct Field<'a> {
    assigned_dot: &'a Dot<'a>,
    id: Id,
    position: Position,
}

struct Player {
    id: Id,
    name: String,
    passphrase: String,
    // TODO: websocket
}

/// Main class of this project. Manages all ressources
struct Galaxy<'a> {
    network: Network<'a>,
}

/// Manages the Websocket connection
struct Network<'a> {
    galaxy: &'a Galaxy<'a>,
}

struct GameChange<'a> {
    player: &'a Player,
    affected_field: &'a Field<'a>,
    new_association: &'a Dot<'a>,
    old_association: &'a Dot<'a>,
}

// Implement comparision operators for Position
impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.1 == other.1 {
            self.0.cmp(&other.0)
        } else {
            self.1.cmp(&other.1)
        }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
