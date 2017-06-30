extern crate image;
use image::{ImageBuffer, ColorType, Pixel, Rgb};
use image::png::PNGEncoder;

extern crate libc;
use libc::{ENOENT, EISDIR};

extern crate fuse_mt;
use fuse_mt::*;

extern crate time;
use time::Timespec;

use std::u8;
use std::io::{Write, stderr};
use std::env::{args_os, current_dir};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::exit;

struct ColorFs {
    init_time: Timespec,
    uid: u32,
    gid: u32,
}

impl ColorFs {
    fn new(uid: u32, gid: u32) -> Self {
        ColorFs {
            init_time: time::get_time(),
            uid: uid,
            gid: gid,
        }
    }
}

impl FilesystemMT for ColorFs {
    fn init(&self, _req: RequestInfo) -> ResultEmpty {
        Ok(())
    }

    fn read(&self, _req:RequestInfo, path:&Path, _fh:u64, offset:u64, size:u32) -> ResultData {
        if path == Path::new("/") {
            return Err(EISDIR);
        }

        if path.iter().count() != 2 {
            return Err(ENOENT);
        }

        if path.extension() != Some(OsStr::new("png")) {
            return Err(ENOENT);
        }

        let color_str = path.iter().nth(1).unwrap();
        let color = match color_from_str(color_str) {
            Ok(color) => color,
            Err(_) => {
                return Err(ENOENT);
            },
        };

        let image = ImageBuffer::from_pixel(1, 1, color);
        let mut buf = Vec::new();
        {
            let encoder = PNGEncoder::new(&mut buf);
            let _ = encoder.encode(
                &image.into_vec(),
                1, 1,
                ColorType::RGB(8),
            );
        }

        let len = buf.len();

        let end = {
            if (size as u64 + offset) as usize > len {
                len
            } else {
                size as usize
            }
        };

        Ok(buf[offset as usize..end].to_owned())
    }

    fn opendir(&self, _req: RequestInfo, path: &Path, _flags: u32) -> ResultOpen {
        if path == Path::new("/") {
            Ok((0, 0))
        } else {
            Err(ENOENT)
        }
    }

    fn getattr(&self, _req: RequestInfo, path: &Path, _fh: Option<u64>) -> ResultEntry {
        if path == Path::new("/") {
            return Ok((
                Timespec::new(1, 0),
                FileAttr {
                    size: 4096,
                    blocks: 8,
                    atime: self.init_time,
                    mtime: self.init_time,
                    ctime: self.init_time,
                    crtime: self.init_time,
                    kind: FileType::Directory,
                    perm: 0o700,
                    nlink: 2,
                    uid: self.uid,
                    gid: self.gid,
                    rdev: 0,
                    flags: 0,
                }
            ));
        }

        if path.iter().count() != 2 {
            return Err(ENOENT);
        }

        if path.extension() != Some(OsStr::new("png")) {
            return Err(ENOENT);
        }

        if let Ok(_color) = color_from_str(path.iter().nth(1).unwrap()) {
            return Ok((
                Timespec::new(1, 0),
                FileAttr {
                    size: 45,
                    blocks: 1,
                    atime: self.init_time,
                    mtime: self.init_time,
                    ctime: self.init_time,
                    crtime: self.init_time,
                    kind: FileType::RegularFile,
                    perm: 0o400,
                    nlink: 0,
                    uid: self.uid,
                    gid: self.gid,
                    rdev: 0,
                    flags: 0,
                }
            ));
        }

        return Err(ENOENT);
    }
}

fn color_from_str(s: &OsStr) -> Result<Rgb<u8>, ()> {
    let s = s.to_string_lossy();

    if s.len() != 10 {
        return Err(());
    }

    let r = match u8::from_str_radix(&s[0..2], 16) {
        Ok(color) => color,
        Err(_) => {
            return Err(());
        },
    };

    let b = match u8::from_str_radix(&s[2..4], 16) {
        Ok(color) => color,
        Err(_) => {
            return Err(());
        },
    };

    let g = match u8::from_str_radix(&s[4..6], 16) {
        Ok(color) => color,
        Err(_) => {
            return Err(());
        },
    };

    println!("{} {} {}", r, g, b);
    Ok(Rgb::from_channels(r, g, b, 0))
}

fn main() {
    let mut mountpoint = match args_os().nth(1) {
        Some(arg) => PathBuf::from(arg),
        None => {
            let _ = writeln!(stderr(), "Usage: colorfs MOUNTPOINT");
            exit(1);
        },
    };

    let mut current_directory = match current_dir() {
        Ok(dir) => dir,
        Err(_) => {
            let _ = writeln!(stderr(), "Failed to determine current directory; try again with an absolute path");
            exit(1);
        },
    };

    current_directory.push(mountpoint);
    mountpoint = current_directory;

    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };

    let _ = mount(FuseMT::new(ColorFs::new(uid, gid), 0), &mountpoint, &[]);
}
