use seed_seeker::run;

#[tokio::main]
async fn main() {
    run().await.expect("Failed to run");
}
