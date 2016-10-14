use std::io::Write;
use std::fs::OpenOptions;
use std::process::{Command,Output};

fn main() {
    let mut log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("build.log")
        .expect("Failed to open build.log");
    let _r = match Command::new("python").arg("build.py").output() {
        Ok(Output {
            status: exit_status,
            stdout,
            stderr
        }) => {
            let _r = writeln!(&mut log_file,"STDERR:\n{}\n\nSTDOUT:\n{}\n",
                            String::from_utf8_lossy(stderr.as_slice()),
                            String::from_utf8_lossy(stdout.as_slice()));
            if !exit_status.success() {
                if let Some(code) = exit_status.code() {
                    writeln!(&mut log_file,"error when running build.py : process returned code {}",code)
                } else {
                    writeln!(&mut log_file,"build.py was interrupted")
                    // it's an interruption, no need to print an error here
                }
            } else {
               writeln!(&mut log_file,"build.py terminated successfully")
            }
        },
        Err(e) => writeln!(&mut log_file,"An unknown error happened : {:?}",e)
    };
}
