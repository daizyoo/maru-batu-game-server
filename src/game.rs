use actix_web::Responder;
use serde::{Deserialize, Serialize};

use crate::{room::User, Response};

type Field = [[Option<Square>; 3]; 3];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Square {
    Maru,
    Batu,
}

#[derive(Debug, Serialize, Deserialize)]
struct Game {
    field: Field,
    turn: User,
    winner: Option<User>,
}

pub async fn sync() -> impl Responder {
    Response::ok(0)
}
