use std::env;
use std::path::PathBuf;
use std::fs;
fn main() {
    let current_dir = env::current_dir().unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut source_logo_file = current_dir.clone();
    source_logo_file.push("logo_toyunda.png");
    let mut target_logo_file = out_dir.clone();
    target_logo_file.push("../../../logo_toyunda.png");
    fs::copy(source_logo_file,target_logo_file);
}
