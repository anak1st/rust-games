mod app;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("World"))]
    name: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut app = app::App::default();
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
