use std::sync::mpsc::channel;

use actix_web::{
    web::{Data, Json},
    Responder,
};
use serde::{Deserialize, Serialize};

use tracing::{error, info};

use crate::{room::User, GameWaitRoom, Response, WaitGameList};

type Field = [[Option<Square>; 3]; 3];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Square {
    Maru,
    Batu,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    field: Field,
    turn: User,
    winner: Option<User>,
}

#[derive(Deserialize)]
pub struct SyncGame {
    game: Game,
    room: String,
}

pub async fn sync(wait_list: Data<WaitGameList>, Json(game): Json<SyncGame>) -> impl Responder {
    let mut wait_list = wait_list.0.lock().unwrap();
    info!("sync {}", game.room);
    if let Some(wait) = wait_list.get_mut(&game.room) {
        if let Err(e) = wait.sender.send(game.game) {
            error!("send error: {:?}", e);
        }
        info!("sync game");
        Response::ok(true)
    } else {
        error!("map get_mut error");
        Response::ok(false)
    }
}

#[derive(Deserialize)]
pub struct RoomName {
    name: String,
}

pub async fn wait(wait_list: Data<WaitGameList>, Json(info): Json<RoomName>) -> impl Responder {
    let mut wait_list = wait_list.0.lock().unwrap();
    let (sender, receiver) = channel();
    if wait_list.get(&info.name).is_none() {
        wait_list.insert(
            info.name.clone(),
            GameWaitRoom {
                sender: sender.clone(),
            },
        );
    }

    info!("wait {}", info.name);
    if let Some(wait_room) = wait_list.get_mut(&info.name) {
        wait_room.sender = sender;
        if let Ok(g) = receiver.recv() {
            info!("recv {:#?}", g);
            return Response::ok(g);
        }
    }

    error!("error");

    Response::<Game>::error()
}
