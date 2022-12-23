pub mod error;
pub mod macros;

use error::SrError;
use macros::cstr;

use clap::{crate_authors, crate_name, Parser};
use std::env;
use std::io;
use std::mem::MaybeUninit;
use std::ops::Drop;
use std::os::unix::io::{RawFd, FromRawFd, AsRawFd};
use std::fs::File;
use std::io::Write;

use nix::errno::Errno;
use nix::libc;
use nix::pty::{forkpty, Winsize};
use nix::sys::termios::{self, cfmakeraw, tcgetattr, tcsetattr, Termios};
use nix::unistd::{read, write, execv, ForkResult};

use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};

use chrono::prelude::*;

#[derive(Parser)]
#[command(name = crate_name!())]
#[command(author = crate_authors!())]
struct Cli {
    #[arg(short, long, default_value = "typescript")]
    output_path: String,
}

fn write_all(fd: RawFd, buf: &[u8]) -> Result<(), Errno> {
    let mut count = buf.len();
    loop {
        let sz = write(fd, buf)?;
        count -= sz;

        if count <= 0 {
            break;
        }
    }
    Ok(())
}

struct Session {
    og_attrs: Termios,
    og_winsize: Winsize,

    max_events: usize,
}

impl Session {
    pub fn create(max_events: usize) -> Result<Self, SrError> {
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
        let winsize = unsafe { winsize.assume_init() };
        let og_winsize = Winsize {
            ws_row: winsize.ws_row,
            ws_col: winsize.ws_col,
            ws_xpixel: winsize.ws_xpixel,
            ws_ypixel: winsize.ws_ypixel,
        };

        Ok(Session {
            og_attrs,
            og_winsize,
            max_events,
        })
    }

    pub fn record(&mut self, output_path: &str) -> Result<(), SrError> {
        print!("sr recording started, output file is {}\r\n", output_path);
        let result = unsafe { forkpty(&self.og_winsize, &self.og_attrs) };
        match result {
            Ok(result) => {
                match result.fork_result {
                    ForkResult::Parent { .. } => {
                        let master = unsafe { File::from_raw_fd(result.master) };
                        let stdin = io::stdin();
                        let stdout = io::stdout();
                        let mut output = File::create(output_path).unwrap();

                        // set up event loop

                        let mut poll = Poll::new()?;
                        let mut events = Events::with_capacity(self.max_events);

                        let master_token = Token(0);
                        poll.registry().register(
                            &mut SourceFd(&master.as_raw_fd()),
                            master_token,
                            Interest::READABLE,
                        )?;

                        let stdin_token = Token(1);
                        poll.registry().register(
                            &mut SourceFd(&stdin.as_raw_fd()),
                            stdin_token,
                            Interest::READABLE,
                        )?;

                        // start event loop

                        loop {
                            let mut should_end = false;
                            poll.poll(&mut events, None)?;
                            
                            let mut buf = [0; 0x1000];
                            for event in events.iter() {
                                if event.is_read_closed() || event.is_error() {
                                    should_end = true;
                                } else {
                                    let tok = event.token();
                                    if event.is_readable() && tok == master_token {
                                        let sz = read(master.as_raw_fd(), &mut buf).unwrap();
                                        if sz == 0 {
                                            should_end = true;
                                        } else {
                                            write_all(stdout.as_raw_fd(), &buf[..sz]).unwrap();
                                            write_all(output.as_raw_fd(), &buf[..sz]).unwrap();
                                        }
                                    }
                                    if event.is_readable() && tok == stdin_token {
                                        let sz = read(stdin.as_raw_fd(), &mut buf).unwrap();
                                        if sz == 0 {
                                            should_end = true;
                                        } else {
                                            let result = write_all(master.as_raw_fd(), &buf[..sz]);
                                            match result {
                                                Err(Errno::EIO) => should_end = true,
                                                Err(..) => panic!("uh oh :("),
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                            if should_end {
                                break;
                            }
                        }
                        write!(output, "\nsr recording done on {}", Local::now().to_rfc2822()).unwrap();
                        print!("sr recording done, output file is {}\r\n", output_path);

                    }
                    ForkResult::Child => {
                        let shell = cstr!(env::var("SHELL").unwrap());
                        let args = [cstr!("-i")];
                        execv(shell, &args).unwrap();
                    }
                }
                Ok(())
            }
            Err(err) => {
                eprintln!("forkpty failed");
                Err(err.into())
            }
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let stdin = io::stdin().as_raw_fd();
        tcsetattr(stdin, termios::SetArg::TCSANOW, &self.og_attrs).unwrap();
    }
}

fn record_session(output_path: &str) {
    let max_events = 32;
    let mut session = Session::create(max_events).unwrap();
    session.record(output_path).unwrap();
}

fn main() {
    let cli = Cli::parse();

    record_session(&cli.output_path);
}
