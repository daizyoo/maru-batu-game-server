mod game;
mod room;

use std::{
    collections::HashMap,
    io::Result,
    sync::{mpsc::Sender, Mutex},
};

use actix_web::{
    web::{post, scope, Data},
    App, HttpResponse, HttpServer,
};

use game::Game;
use room::{Room, RoomInfo};
use serde::Serialize;

const ADDRESS: &str = "127.0.0.1:8080";

struct RoomList(Mutex<Vec<Room>>);

struct WaitRoomList(Mutex<Vec<WaitRoom>>);

struct WaitGameList(Mutex<HashMap<String, GameWaitRoom>>);

struct GameWaitRoom {
    sender: Sender<Game>,
}

struct WaitRoom {
    sender: Sender<Room>,
    room_info: RoomInfo,
}

search!(WaitRoom { String } => fn value(&self) -> Self::Value {
    self.room_info.name()
});

#[derive(Debug, Serialize)]
struct Response<T> {
    data: Option<T>,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let room_list = Data::new(RoomList(Mutex::new(Vec::new())));
    let wait_room_list = Data::new(WaitRoomList(Mutex::new(Vec::new())));
    let wait_game_list = Data::new(WaitGameList(Mutex::new(HashMap::new())));

    HttpServer::new(move || {
        App::new()
            .service(
                scope("/room")
                    .route("/create", post().to(room::create))
                    .route("/delete", post().to(room::delete))
                    .route("/enter", post().to(room::enter))
                    .route("/search", post().to(room::search)),
            )
            .service(
                scope("/game")
                    .route("/sync", post().to(game::sync))
                    .route("/wait", post().to(game::wait)),
            )
            .app_data(room_list.clone())
            .app_data(wait_room_list.clone())
            .app_data(wait_game_list.clone())
    })
    .bind(ADDRESS)?
    .run()
    .await?;

    Ok(())
}

pub trait Search {
    type Value: PartialEq;

    fn value(&self) -> Self::Value;
}

pub fn search_vec<'a, T: Search>(vec: &'a Vec<T>, v: T::Value) -> Option<&'a T> {
    vec.iter().find(|&x| x.value() == v)
}

#[macro_export]
macro_rules! search {
    ($s:ident { $t:ty } => $f:item) => {
        impl crate::Search for $s {
            type Value = $t;
            $f
        }
    };
}

impl<T: Serialize> Response<T> {
    fn new(data: Option<T>) -> Response<T> {
        Response { data }
    }
    fn ok(data: T) -> HttpResponse {
        HttpResponse::Ok().json(Response::new(Some(data)))
    }
    fn error() -> HttpResponse {
        HttpResponse::Ok().json(Response::<T>::new(None))
    }
}
