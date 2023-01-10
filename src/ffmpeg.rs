use anyhow::Result;
use duct::cmd;
use std::io::prelude::*;
use std::io::BufReader;

pub fn transform_format_code(input: &str) -> Result<()> {
    println!("开始转码...");
    let output = input.replace(".m4s", ".mp3");
    let big_cmd = cmd!(
        "bin/mac/ffmpeg",
        "-i",
        input,
        "-acodec",
        "libmp3lame",
        output
    );
    let reader = big_cmd.stderr_to_stdout().reader().unwrap();
    let lines = BufReader::new(reader).lines();

    for x in lines {
        println!("{}", x.unwrap())
    }

    Ok(())
}
