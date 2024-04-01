use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::time::Duration;

use memmap2::Mmap;
use mio::{Events, Interest, Poll, Token};
use tokio::sync::mpsc::UnboundedSender;


// not sure what this meant to do
pub fn put_kernel_messages_into_queue(send_queue_tx: UnboundedSender<String>) {
    let file = File::open("/private/var/log/system.log").expect("Failed to syslog file");
    let mmap = unsafe { Mmap::map(&file).expect("Failed to map file") };

    let poll = Poll::new().expect("Failed to create poll");
    let token = Token(0);

    let raw_fd = file.as_raw_fd();
    if raw_fd < 0 {
        eprintln!("Invalid file descriptor");
        return;
    }

    poll.registry()
        .register(&mut EventedFile(&file), token, Interest::READABLE)
        .expect("Failed to register file");

    let mut events = Events::with_capacity(1024);
    let mut position = mmap.len() as usize;

    loop {
        poll.poll(&mut events, None).expect("Failed to poll");

        for event in &events {
            if event.is_readable() {
                let lastest_syslog_content = std::str::from_utf8(&mmap[position..])
                    .expect("Invalid UTF-8 content");
                send_queue_tx.send(lastest_syslog_content.to_owned());
                println!("{}", lastest_syslog_content);
                position = mmap.len();
            }
        }

        std::thread::sleep(Duration::from_millis(500));
    }
}

struct EventedFile<'a>(&'a File);

impl<'a> Read for EventedFile<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl<'a> Seek for EventedFile<'a> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.0.seek(pos)
    }
}

impl<'a> AsRawFd for EventedFile<'a> {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.0.as_raw_fd()
    }
}
