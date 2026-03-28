mod app;
mod game;

use anyhow::Result;
use clap::Parser;

use crate::game::GameKind;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, value_enum)]
    game: Option<GameKind>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut app = app::App::new(args.game);
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
