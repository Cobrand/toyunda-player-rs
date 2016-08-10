use std::env;
use std::path::PathBuf;
use std::process::Command;
fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("../../..");

    let _ = Command::new("cp")
                    .arg("logo_toyunda.png")
                    .arg(out_dir.with_file_name("logo_toyunda.png"))
                    .output()
                    .expect("failed to execute process");

    let _ = Command::new("cp")
                    .arg("-R")
                    .arg("web/")
                    .arg(out_dir)
                    .output()
                    .expect("failed to execute process");
}
