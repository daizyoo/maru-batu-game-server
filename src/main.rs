mod room;

use std::{io::Result, sync::Mutex};

use actix_web::{web::scope, App, HttpServer};

use room::Room;
use serde::{Deserialize, Serialize};

const ADDRESS: &str = "127.0.0.1:8080";

struct RoomList(Mutex<Vec<Room>>);

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
}

#[derive(Serialize)]
struct Response<T> {
    data: Option<T>,
}

#[actix_web::main]
async fn main() -> Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(RoomList(Mutex::new(Vec::new())))
            .service(scope("/room"))
    })
    .bind(ADDRESS)?
    .run()
    .await?;

    Ok(())
}
