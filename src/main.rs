mod bv;
mod cli;
mod config;
mod ffmpeg;
// mod upload;
mod utils;

use std::fs;

use crate::bv::Bv;
use clap::Parser;
use cli::Args;
// use upload::User;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let bv_list1 = match args.list {
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

    let mut audios = Vec::new();

    // 下载音频
    for bv in bv_list.iter() {
        let video_info = bv.get_video_info().await.unwrap();
        let files = video_info.get_audios().await.unwrap();
        let input = files.to_str().unwrap().to_string();
        audios.push(input);
    }
    // 通过FFmpeg转码
    for audio in audios.iter() {
        ffmpeg::transform_format_code(&audio).unwrap();
        fs::remove_file(&audio).unwrap();
        
    }
}
