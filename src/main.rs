use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    weathervane::refresh().await.unwrap();
}
