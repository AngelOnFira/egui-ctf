use std::collections::HashSet;

use crate::ClientId;

pub struct Room {
    pub users: HashSet<ClientId>,
}
