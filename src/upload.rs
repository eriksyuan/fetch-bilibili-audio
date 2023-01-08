use anyhow::{anyhow, Result};
use console::{style, Term};
use futures_util::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{multipart, Body, Client};
use serde_json::{self, Value};
use std::cmp::min;
use std::time;
use std::{thread, time::Duration};
use tokio::fs::File;
use tokio::io::BufReader;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::io::ReaderStream;
const NETEAST_BASE_URL: &str = "http://129.211.15.249:3000";

pub struct User {
    pub cookie: String,
}

impl User {
    pub async fn login() -> Result<Self> {
        match get_local_cookie().await {
            Ok(cookie) => {
                return Ok(Self { cookie });
            }
            Err(_) => {}
        };

        let key = generate_key().await?;
        let qr_str = generate_qr(&key).await?;

        qr2term::print_qr(qr_str)?;

        let term = Term::stdout();
        term.set_title("扫码");

        term.write_line("请使用网易云APP扫描二维码登录")?;
        term.write_line("")?;
        loop {
            let (status, message, cookie) = check_login(&key).await?;
            term.move_cursor_up(1)?;
            term.write_line(&format!("{}", style(message).cyan()))?;
            if status == 803 && cookie.is_some() {
                let cookie_txt = cookie.unwrap();
                save_local_cookie(&cookie_txt).await?;
                return Ok(Self { cookie: cookie_txt });
            } else if status == 800 {
                return Err(anyhow!("二维码已过期"));
            }
            thread::sleep(Duration::from_secs(1));
        }
    }

    pub async fn upload(&self, file_path: &str, file_name: String) -> Result<()> {
        let url = format!(
            "{}/cloud?cookie={}&timestramp={}",
            NETEAST_BASE_URL,
            self.cookie.clone(),
            get_timestamp()
        );

        let file = File::open(file_path).await?;

        let total_size = file.metadata().await.unwrap().len();

        let mut reader_stream = ReaderStream::new(file);

        let mut uploaded_size: u64 = Client::new().get(&url).send().await?.content_length().unwrap_or(0);

        let pb = ProgressBar::new(total_size);

        pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.red} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

        let async_stream = async_stream::stream! {
            while let Some(chunk) = reader_stream.next().await {
                if let Ok(byte) = &chunk {

                    let new = min(uploaded_size + (byte.len() as u64), total_size);
                    uploaded_size = new;
                    pb.inc(byte.len() as u64);
                    if(uploaded_size >= total_size){
                    //   println!("上传成功！！！")
                        pb.finish()
                    }
                }
                yield chunk;
            }
        };

        let file_body = Body::wrap_stream(async_stream);
        //make form part of file
        let some_file = multipart::Part::stream(file_body)
            .file_name(file_name)
            .mime_str("audio/x-flac")?;

        let form = multipart::Form::new().part("songFile", some_file);

         Client::new()
            .post(url)
            .header(
                "Range",
                "bytes=".to_owned() + &uploaded_size.to_string() + "-",
            )
            .multipart(form)
            .send()
            .await?;

        Ok(())
    }
}

pub fn get_timestamp() -> u64 {
    let now = time::SystemTime::now();
    let since_the_epoch = now
        .duration_since(time::UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs()
}
pub async fn generate_key() -> Result<String> {
    let url = format!(
        "{}/login/qr/key?timestamp={}",
        NETEAST_BASE_URL,
        get_timestamp()
    );

    let text_res = reqwest::get(url).await?.text().await?;
    let value: Value = serde_json::from_str(&text_res)?;

    let key = value["data"]["unikey"].as_str().unwrap().to_string();
    Ok(key)
}

pub async fn generate_qr(key: &String) -> Result<String> {
    let url = format!(
        "{}/login/qr/create?key={}&timestamp={}",
        NETEAST_BASE_URL,
        key,
        get_timestamp()
    );
    let text_res = reqwest::get(url).await?.text().await?;
    let value: Value = serde_json::from_str(&text_res)?;
    let qr = value["data"]["qrurl"].as_str().unwrap().to_string();
    Ok(qr)
}

pub async fn check_login(key: &String) -> Result<(i64, String, Option<String>)> {
    let url = format!(
        "{}/login/qr/check?key={}&timestamp={}",
        NETEAST_BASE_URL,
        key,
        get_timestamp()
    );
    let text_res = reqwest::get(url).await?.text().await?;
    let value: Value = serde_json::from_str(&text_res)?;

    let code = value["code"].as_i64().unwrap();
    let message = value["message"].as_str().unwrap().to_string();

    let cookie = value["cookie"].as_str().map(|s| s.to_string());

    Ok((code, message, cookie))
}

pub async fn get_local_cookie() -> Result<String> {
    let file = File::open("cookie.txt").await?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents).await?;
    Ok(contents)
}

pub async fn save_local_cookie(cookie: &String) -> Result<()> {
    let mut file = File::create("cookie.txt").await?;
    file.write_all(cookie.as_bytes()).await?;
    Ok(())
}
