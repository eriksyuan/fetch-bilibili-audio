use std::{env, fs};

use anyhow::Result;

use crate::bv::Bv;

#[derive(Clone, Copy)]
pub struct ConfigFile {}

impl ConfigFile {
    pub fn read(file: &str) -> Result<Vec<Bv>> {
        let mut bv_list = Vec::new();

        let file_path = env::current_exe()?.parent().unwrap().join(file);

        let content = fs::read_to_string(file_path)?;
        for line in content.lines() {
            bv_list.push(Bv::new(line.to_string()));
        }
        Ok(bv_list)
    }
}
