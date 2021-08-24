use ya_gist_core::{api::Client, models::zeus::apc_dir_purc_aply_e::get as model};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::new_localhost_debug();

    let request = model::Request::default();
    let response: model::Response = request.call(&client).await?;
    println!("Output: {:#?}", &response);

    Ok(())
}
