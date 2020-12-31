use dotenv::dotenv;

fn main() {
    dotenv().ok();
    weathervane::refresh().unwrap();
}
