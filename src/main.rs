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

/// 解析命令行参数并启动应用主循环。
fn main() -> Result<()> {
    let args = Args::parse();
    let mut app = app::App::new(args.game);
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
