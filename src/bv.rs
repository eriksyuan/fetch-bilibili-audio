use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{self, HeaderMap};
use serde_json::Value;
use std::{path::PathBuf, thread, time::Duration};
use tokio::fs;
use ua_generator::ua::spoof_ua;

use crate::utils::{save, save_with_cb};

pub async fn get_download_path() -> Result<PathBuf> {
    let path = PathBuf::new().join("download");
    if !path.is_dir() {
        fs::create_dir(&path).await?;
    }
    Ok(path)
}

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

        let data = reqwest::get(url).await?.text().await?;

        let json_data: Value = serde_json::from_str(data.as_str()).unwrap();

        let title = &json_data["data"]["title"];

        let cid = &json_data["data"]["pages"][p.parse::<usize>().unwrap() - 1]["cid"];

        let cover = &json_data["data"]["pic"].as_str().unwrap().to_string();
        Ok(VideoInfo {
            title: title.as_str().unwrap().to_string(),
            bv: bvid,
            cid: cid.as_i64().unwrap().to_string(),
            cover: cover.clone(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct VideoInfo {
    title: String,
    bv: String,
    cid: String,
    cover: String,
}

impl VideoInfo {
    pub async fn download_audio(
        &self,
        audio_url: reqwest::Url,
        part: Option<usize>,
        format: &String,
    ) -> Result<PathBuf> {
        let client = reqwest::Client::builder().user_agent(spoof_ua()).build()?;

        let mut headers = HeaderMap::new();
        headers.insert("Accept", "*/*".parse().unwrap());
        headers.insert("Accept-Language", "en-US,en;q=0.5".parse().unwrap());
        headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
        // headers.insert("Range", "bytes=0-".parse().unwrap());
        headers.insert(
            "Referer",
            ("https://api.bilibili.com/x/web-interface/view?bvid=".to_string() + &self.bv.clone())
                .parse()
                .unwrap(),
        );
        headers.insert("Origin", "https://www.bilibili.com".parse().unwrap());
        headers.insert("Connection", "keep-alive".parse().unwrap());

        let total_size = {
            let resp = client
                .head(audio_url.clone())
                .headers(headers.clone())
                .send()
                .await?;
            if resp.status().is_success() {
                resp.headers()
                    .get(header::CONTENT_LENGTH)
                    .and_then(|ct_len| ct_len.to_str().ok())
                    .and_then(|ct_len| ct_len.parse().ok())
                    .unwrap_or(0)
            } else {
                return Err(anyhow!(
                    "Couldn't download URL: {}. Error: {:?}",
                    audio_url,
                    resp.status(),
                ));
            }
        };

        let pb = ProgressBar::new(total_size);

        pb.set_style(ProgressStyle::default_bar()
                 .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                 .unwrap()
                 .progress_chars("#>-"));

        let mut request = client.get(audio_url.clone()).headers(headers.clone());

        let title = self.title.replace("/", "").replace("\\", "");

        let title = match part {
            None => format!("{}.{}", title, format),
            Some(part) => format!("{}_{}.{}", title, part, format),
        };

        let save_path = get_download_path().await?.join(&title);

        // 如果文件已存在
        if save_path.exists() {
            let size = save_path.metadata()?.len().saturating_sub(1);
            request = request.header(header::RANGE, format!("bytes={}-", size));
            pb.inc(size);
        }

        let mut response = request.send().await?;

        let cb = |size: usize| {
            pb.inc(size as u64);
        };

        save_with_cb(&save_path, &mut response, &cb).await?;

        println!("BV {} download finished", self.bv);
        // 等待1s;
        thread::sleep(Duration::from_secs(1));
        Ok(save_path)
    }
    pub async fn get_audios(&self, all: bool, format: &String) -> Result<Vec<PathBuf>> {
        let urls = self.get_audio_urls().await?;

        let urls = if all { urls } else { vec![urls[0].clone()] };

        self.download_cover().await?;

        let mut audio_paths = Vec::new();
        for i in 0..urls.len() {
            let url: reqwest::Url = urls[i].parse()?;
            // let index =
            let audio_path = self.download_audio(url, Some(i + 1), format).await?;

            audio_paths.push(audio_path);

            // 应用封面
        }

        Ok(audio_paths)
    }

    pub async fn get_audio_urls(&self) -> Result<Vec<String>> {
        let base_url = "http://api.bilibili.com/x/player/playurl?fnval=16&";

        let url = base_url.to_string() + "cid=" + &self.cid + "&bvid=" + &self.bv;

        let audio_res_text = reqwest::get(url).await?.text().await?;

        let audio_res_value: Value = serde_json::from_str(audio_res_text.as_str()).unwrap();

        let audio_urls = audio_res_value["data"]["dash"]["audio"].as_array().unwrap();

        let urls = audio_urls
            .iter()
            .map(|l| l["baseUrl"].as_str().unwrap().to_string())
            .collect::<Vec<String>>();

        Ok(urls)
    }

    pub async fn download_cover(&self) -> Result<PathBuf> {
        let url = self.cover.clone();

        let mut response = reqwest::get(url).await?;

        let file_path = get_download_path().await?.join(format!("{}.jpg", self.bv));

        save(&file_path, &mut response).await?;

        Ok(file_path)
    }
}
