use tokio::main;
use teo_result::Result;
use teo::app::App;
use teo::cli::entrance::Entrance;

#[main]
async fn main() -> Result<()> {
    let app = App::new_with_entrance_and_runtime_version(Some(Entrance::CLI), None)?;
    app.run().await
}
