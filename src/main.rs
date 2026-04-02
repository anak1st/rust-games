mod app;
mod game;

use anyhow::Result;
use clap::Parser;

use crate::game::{GameKind, RenderMode};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, value_enum)]
    game: Option<GameKind>,
    #[arg(long, value_enum, default_value_t = RenderMode::Double)]
    render_mode: RenderMode,
}

/// 解析命令行参数并启动应用主循环。
fn main() -> Result<()> {
    let args = Args::parse();
    let mut app = app::App::new(args.game, args.render_mode);
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
