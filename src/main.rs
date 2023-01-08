mod bv;
mod cli;
mod config;
mod upload;
mod utils;

use crate::bv::Bv;
use clap::Parser;
use cli::Args;
use upload::User;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let user = User::login().await.unwrap();


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

    for bv in bv_list.iter() {
        let video_info = bv.get_video_info().await.unwrap();

        let audios = video_info.get_audios(args.all, &args.format).await.unwrap();

        for audio in audios.iter() {
            // todo!(“中文乱码”)
            let file_name = audio.file_name().unwrap().to_str().unwrap().to_string();
            if args.upload {
                println!("开始上传{:?}", file_name);
                user.upload(audio.to_str().unwrap(), file_name)
                    .await
                    .unwrap();
            }
        }
    // }
}
