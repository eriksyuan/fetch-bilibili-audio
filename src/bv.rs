use std::{
    env,
    fs::File,
    io::{Read, Write},
};

use anyhow::{anyhow, Result};
use reqwest::header::HeaderMap;
use serde_json::Value;
use ua_generator::ua::spoof_ua;

#[derive(Debug, Clone)]
pub struct Bv {
    pub bv: String,
}

impl Bv {
    pub fn new(bv: String) -> Self {
        Self { bv }
    }

    pub async fn get_video_info(&self) -> Result<VideoInfo> {
        let len = self.bv.len();
        if len < 12_usize {
            return Err(anyhow!("bv is too short"));
        }
        let (bvid, p) = if len == 12_usize {
            (self.bv.clone(), "1".to_string())
        } else {
            (self.bv[..12].to_string(), self.bv[13..].to_string())
        };

        let url = "https://api.bilibili.com/x/web-interface/view?bvid=".to_string() + &bvid;

        println!("{}", url);
        let data = reqwest::get(url).await?.text().await?;

        let json_data: Value = serde_json::from_str(data.as_str()).unwrap();

        let title = &json_data["data"]["title"];

        let cid = &json_data["data"]["pages"][p.parse::<usize>().unwrap() - 1]["cid"];

        println!("{:?}", cid);
        Ok(VideoInfo {
            title: title.as_str().unwrap().to_string(),
            bv: bvid,
            cid: cid.as_i64().unwrap().to_string(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct VideoInfo {
    title: String,
    bv: String,
    cid: String,
}

impl VideoInfo {
    pub async fn get_audio(&self) -> Result<()> {
        let audio_url = self.gte_audio_url().await?;

        let client = reqwest::Client::builder().user_agent(spoof_ua()).build()?;

        let mut headers = HeaderMap::new();
        headers.insert("Accept", "*/*".parse().unwrap());
        headers.insert("Accept-Language", "en-US,en;q=0.5".parse().unwrap());
        headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
        headers.insert("Range", "bytes=0-".parse().unwrap());
        headers.insert(
            "Referer",
            ("https://api.bilibili.com/x/web-interface/view?bvid=".to_string() + &self.bv.clone())
                .parse()
                .unwrap(),
        );
        headers.insert("Origin", "https://www.bilibili.com".parse().unwrap());
        headers.insert("Connection", "keep-alive".parse().unwrap());

        let bytes = client
            .get(audio_url)
            .headers(headers)
            .send()
            .await?
            .bytes()
            .await?;

        let title = self.title.replace("/", "").replace("\\", "");

        let save_path = env::current_exe()?
            .parent()
            .unwrap()
            .join("download")
            .join(title + ".mp3");

        println!("{:?}", save_path);

        let content = bytes.bytes().collect::<Result<Vec<_>, _>>()?;

        let mut file = match File::create(&save_path) {
            Err(why) => panic!("couldn't create {}", why),
            Ok(file) => file,
        };

        file.write_all(&content)?;

        Ok(())
    }

    pub async fn gte_audio_url(&self) -> Result<String> {
        let base_url = "http://api.bilibili.com/x/player/playurl?fnval=16&";

        let url = base_url.to_string() + "cid=" + &self.cid + "&bvid=" + &self.bv;

        let audio_res_text = reqwest::get(url).await?.text().await?;

        let audio_res_value: Value = serde_json::from_str(audio_res_text.as_str()).unwrap();

        let audio_url = audio_res_value["data"]["dash"]["audio"][0]["baseUrl"]
            .as_str()
            .unwrap();

        Ok(audio_url.to_string())
    }
}
