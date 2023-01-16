use anyhow::Result;
use duct::cmd;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

pub fn transform_format_code(input: &str) -> Result<()> {
    let output = input.replace(".m4s", ".mp3");

    let big_cmd = cmd!(
        "bin\\win\\ffmpeg.exe",
        "-i",
        input,
        "-aq",
        "0",
        "-acodec",
        "libmp3lame",
        &output
    );
    let reader = big_cmd.stderr_to_stdout().reader().unwrap();
    let lines = BufReader::new(reader).lines();

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(34));
    let spinner_style =
        ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg:40.cyan/blue}")
            .unwrap();
    pb.set_style(spinner_style);

    let file = Path::new(&output).file_name().unwrap().to_str().unwrap();
    pb.set_prefix(format!("{}", file));

    pb.set_message("正在转码...");
    for _ in lines {}
    pb.finish_with_message("转码完成");
    Ok(())
}
