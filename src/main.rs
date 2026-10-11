mod app;
mod utils;

use app::App;

fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}
