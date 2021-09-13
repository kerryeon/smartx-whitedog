#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_json;

mod api;
mod routes;
mod status;

#[tokio::main]
async fn main() {
    match try_main().await {
        Ok(()) => {}
        Err(e) => error!("Aborted: {}\n{:#?}", &e, &e),
    }
}

async fn try_main() -> anyhow::Result<()> {
    ya_gist_zeus_core::init::init_logger();

    let cors = {
        use rocket::http::Method;
        use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};

        let allowed_origins = AllowedOrigins::all();

        // You can also deserialize this
        CorsOptions {
            allowed_origins,
            allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
            allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
            allow_credentials: true,
            ..Default::default()
        }
        .to_cors()?
    };

    self::routes::mount(
        rocket::build()
            .manage(ya_gist_zeus_client::ZeusClient::infer().await?)
            .attach(cors),
    )
    .launch()
    .await?;
    Ok(())
}
