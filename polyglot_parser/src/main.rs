mod parser;
mod tree;
use crate::parser::*;
use clap::Parser;

// TODO! Check all values on list and dictionary too see if all have the same type

#[derive(clap::Parser)]
#[clap(version, about)]
struct Cli {
    file: std::path::PathBuf,
    #[clap(short, long)]
    debug: bool
}

fn main() -> Result<(), ParseErr> {
    let cli = Cli::parse();
    let main = parse(&cli.file, cli.debug)?;
    
    let buffer = serde_yaml::to_string(&main).unwrap();
    if cli.debug { println!("{buffer}") }
    let path = cli.file.with_extension("yml");
    std::fs::write(path, buffer).unwrap();
    Ok(())
}