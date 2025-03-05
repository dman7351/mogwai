use libc::fork;
use std::process::exit;
//use libc::getpid;

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
