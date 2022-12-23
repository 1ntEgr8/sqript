pub mod error;

use error:: SrError;
use clap::{crate_name, crate_authors, Parser};
use std::io;
use std::os::unix::io::AsRawFd;
use std::mem::MaybeUninit;
use std::ops::Drop;

use nix::sys::termios::{self, tcsetattr, tcgetattr, cfmakeraw, Termios};
use nix::libc;

#[derive(Parser)]
#[command(name = crate_name!())]
#[command(author = crate_authors!())]
struct Cli {
    #[arg(short, long, default_value = "typescript")]
    output_path: String,
}

struct Session {
    og_attrs: Termios,
    og_winsize: libc::winsize,
}

impl Session {
    pub fn create() -> Result<Self, SrError> {
        let stdin = io::stdin().as_raw_fd();
        let mut attrs = tcgetattr(stdin)?;
        let og_attrs = attrs.clone();

        // enter raw mode
        attrs.local_flags &= !(termios::LocalFlags::ECHO);
        cfmakeraw(&mut attrs);
        tcsetattr(stdin, termios::SetArg::TCSANOW, &attrs)?;

        // get terminal window size
        let winsize = MaybeUninit::<libc::winsize>::uninit();
        unsafe {
            libc::ioctl(stdin, libc::TIOCGWINSZ, &winsize);
        }
        let og_winsize = unsafe { winsize.assume_init() };

        Ok(Session {
            og_attrs,
            og_winsize
        })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let stdin = io::stdin().as_raw_fd();
        tcsetattr(stdin, termios::SetArg::TCSANOW, &self.og_attrs).unwrap();
    }
}

fn record_session(output_path: &str) {
    let session = Session::create().unwrap();
}

fn main() {
    let cli = Cli::parse();
    
    record_session(&cli.output_path);
}
