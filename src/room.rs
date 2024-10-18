use std::sync::mpsc::channel;

use actix_web::{
    web::{Data, Json},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{game::Square, search, search_vec, WaitRoom, WaitRoomList};
use crate::{Response, RoomList};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    name: String,
    square: Square,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    name: String,
    user1: User,
    user2: User,
}

#[derive(Debug, Deserialize)]
pub struct RoomInfo {
    name: String,
    user: User,
}

impl RoomInfo {
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

search!(RoomInfo { String } => fn value(&self) -> Self::Value {
    self.name()
});
search!(Room { String  } => fn value(&self) -> Self::Value {
    self.name.clone()
});

pub async fn create(
    room_list: Data<RoomList>,
    wait_rooms: Data<WaitRoomList>,
    Json(room_info): Json<RoomInfo>,
) -> impl Responder {
    let (sender, receiver) = channel();
    wait_rooms
        .0
        .lock()
        .unwrap()
        .push(WaitRoom { sender, room_info });

    if let Ok(room) = receiver.recv() {
        room_list.0.lock().unwrap().push(room.clone());
        info!("create room");
        return Response::ok(room);
    } else {
        error!("create room error")
    }

    Response::<Room>::error()
}

pub async fn enter(
    wait_rooms: Data<WaitRoomList>,
    Json(room_info): Json<RoomInfo>,
) -> impl Responder {
    let wait_rooms = wait_rooms.0.lock().unwrap();
    if let Some(wait_room) = search_vec(&wait_rooms, room_info.name) {
        let room = Room {
            name: wait_room.room_info.name.clone(),
            user1: wait_room.room_info.user.clone(),
            user2: room_info.user,
        };
        if let Err(e) = wait_room.sender.send(room.clone()) {
            error!("{:?}", e);
            return Response::<Room>::error();
        }

        info!("{:#?}", room);

        return Response::ok(room);
    }

    Response::<Room>::error()
}

pub async fn delete(room_list: Data<RoomList>, Json(room): Json<RoomInfo>) -> impl Responder {
    let mut room_list = room_list.0.lock().unwrap();
    if let Some(i) = room_list.iter().position(|r| r.name == room.name) {
        let res = room_list.remove(i);
        println!("delete room: {:?}", res);
    }

    HttpResponse::Ok()
}

pub async fn search(room_list: Data<RoomList>, Json(room): Json<Room>) -> impl Responder {
    let room_list = room_list.0.lock().unwrap();

    let room = search_vec(&room_list, room.name);

    HttpResponse::Ok().json(Response { data: room })
}
