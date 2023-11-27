// SPDX-FileCopyrightText: 2023 Fondazione LINKS
//
// SPDX-License-Identifier: APACHE-2.0

use actix_web::{web, App, HttpServer, middleware::Logger};
use connector::{controllers, utils::iota::IotaState, repository::postgres_repo::init};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::from_path("./env/.env").expect(".env file not found");
    dotenv::from_path("./env/postgres.env").expect("postgres.env file not found");

    env_logger::init();

    let address = std::env::var("ADDR").expect("$ADDR must be set.");
    let port = std::env::var("PORT").expect("$PORT must be set.").parse::<u16>()?;

    let db_pool = init().await?;

    let iota_state = IotaState::init().await?;
    let iota_state_data = web::Data::new(iota_state);
    
    log::info!("Starting up on {}:{}", address, port);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(iota_state_data.clone())
            .service(web::scope("/api")
                .configure(controllers::identities_controller::scoped_config)
                .configure(controllers::assets_controller::scoped_config)
            )
            .wrap(Logger::default())
    })
    .bind((address, port))?
    .run()
    .await.map_err(anyhow::Error::from)
}