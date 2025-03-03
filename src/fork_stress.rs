use libc::{fork, getpid, _exit};
use std::process::exit;

pub fn stress_fork() {
    loop {
        unsafe {
            let pid = fork();
            if pid < 0 {
                eprintln!("Fork failed");
                exit(1);
            }
        }
    }
}
