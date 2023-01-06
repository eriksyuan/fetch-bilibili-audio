mod bv;
mod cli;
mod config;

use crate::bv::Bv;
use clap::Parser;
use cli::Args;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let bv_list1 = match args.file {
        Some(file) => config::ConfigFile::read(&file).expect("配置文件读取失败"),
        None => Vec::new(),
    };

    let bv_list2 = args
        .bv
        .iter()
        .map(|bv| Bv::new(bv.to_string()))
        .collect::<Vec<Bv>>();

    let bv_list = bv_list1
        .into_iter()
        .chain(bv_list2.into_iter())
        .collect::<Vec<Bv>>();

    for bv in bv_list.iter() {
        let video_info = bv.get_video_info().await.unwrap();

        video_info.get_audio().await.unwrap();
    }

    // println!("{:?}", bv_list);
}
