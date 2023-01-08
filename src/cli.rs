use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// 包含bv列表的相对文件路径
    #[arg(short, long)]
    pub list: Option<String>,

    /// Bid
    #[arg(short, long)]
    pub bv: Vec<String>,

    /// 是否下载全部音质
    #[arg(short, long)]
    pub all: bool,

    /// 下载完成后上传至网易云云盘（需要二维码登录）
    #[arg(short, long)]
    pub upload: bool,

    #[arg(short, long, default_value = "mp3")]
    pub format: String,
}
