use duct::cmd;
use std::io::prelude::*;
use std::io::BufReader;

fn main() {
    let big_cmd = cmd!(
        "bin/mac/ffmpeg",
        "-i",
        "download/demo.m4s",
        "-acodec",
        "libmp3lame",
        "output.mp3"
    );
    let reader = big_cmd.stderr_to_stdout().reader().unwrap();
    let  lines = BufReader::new(reader).lines();

    for x in  lines {
       println!("{}",x.unwrap())  
    }
    
    println!("转码完成")
}
