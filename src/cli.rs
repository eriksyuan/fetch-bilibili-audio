use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// 包含bv列表的相对文件路径,默认值"input.txt"
    #[arg(short, long)]
    pub file: Option<String>,

    /// Bid
    #[arg(short, long)]
    pub bv: Vec<String>,
}


