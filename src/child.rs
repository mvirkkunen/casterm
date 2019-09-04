use nix;
use nix::ioctl_write_ptr_bad;
use nix::pty::{forkpty, Winsize};
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{read, close, execv, Pid, ForkResult};
use std::ffi::CString;
use std::io::{self, prelude::*};
use std::os::unix::io::RawFd;
use std::process::exit;
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct Child {
    master: RawFd,
    pid: Pid,
}

fn winsize(rows: u16, cols: u16) -> Winsize {
    Winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    }
}

impl Child {
    pub fn spawn<'a, I>(path: &'a str, argv: I, rows: u16, cols: u16) -> nix::Result<Child>
    where
        I: IntoIterator<Item=&'a str>
    {
        let fork = forkpty(Some(&winsize(rows, cols)), None)?;

        match fork.fork_result {
            ForkResult::Child => {
                let path = CString::new(path).unwrap();
                let mut argv: Vec<_> = argv.into_iter().map(|a| CString::new(a).unwrap()).collect();
                argv.insert(0, path.clone());

                match execv(&path, &argv) {
                    Ok(_) => unreachable!(),
                    Err(err) => {
                        eprintln!("Failed to execute child: {}", err);
                        exit(1);
                    }
                }
            },
            ForkResult::Parent { child: pid } => {
                Ok(Child {
                    master: fork.master,
                    pid,
                })
            }
        }
    }

    pub fn is_running(&self) -> bool {
        match waitpid(self.pid, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(..)) | Ok(WaitStatus::Signaled(..)) => false,
            _ => true,
        }
    }

    pub fn set_window_size(&mut self, rows: u16, cols: u16) {
        ioctl_write_ptr_bad!(tiocswinsz, nix::libc::TIOCSWINSZ, Winsize);

        unsafe { tiocswinsz(self.master, &winsize(rows, cols) as *const Winsize).ok(); }
    }

    pub fn reader(&mut self) -> Reader {
        Reader(self.master)
    }
}

impl Drop for Child {
    fn drop(&mut self) {
        close(self.master).ok();

        if kill(self.pid, Signal::SIGTERM).is_ok() {
            thread::sleep(Duration::from_millis(1));

            for _ in 0..5 {
                if !self.is_running() {
                    return;
                }

                thread::sleep(Duration::from_secs(1));
            }
        }

        kill(self.pid, Signal::SIGKILL).ok();
    }
}

pub struct Reader(RawFd);

impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        read(self.0, buf).map_err(|err| err.as_errno().unwrap().into())
    }
}
