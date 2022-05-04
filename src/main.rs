mod db;

use std::env;
use rocket::{get, routes};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    cli_main(&args[1..]).await;
}

async fn cli_main(args: &[String]) {
    match args.len() {
        0 => {
            let server: Result<(), rocket::Error> = rocket::build()
                .mount("/", routes![index])
                .launch()
                .await;

            if let Err(e) = server {
                panic!("Failed to start a server: {e}");
            }
        }
        _ => match args.first().unwrap().as_str() {
            "db" => db::cli_main(&args[1..]).await.unwrap_or_else(|e| {
                panic!("{e}");
            }),
            _ => {
                panic!("Unknown action");
            }
        },
    }
}