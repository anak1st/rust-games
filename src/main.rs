use anyhow::Result;
use clap::Parser;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("World"))]
    name: String,

    #[arg(short, long, default_value_t = 1)]
    count: u8,
}


fn main() -> Result<()> {
    let args = Args::parse();
    for _ in 0..args.count {
        println!("Hello {}!", args.name);
    }
    Ok(())
}
