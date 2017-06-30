extern crate image;
use image::{ImageBuffer, ColorType, Pixel, Rgb};
use image::png::PNGEncoder;

extern crate libc;
use libc::{ENOENT, EISDIR};

extern crate fuse_mt;
use fuse_mt::*;

use std::io::{Write, stdout, stderr};
use std::env::{args_os, current_dir};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::exit;

// 82 bytes

struct ColorFs;

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

        let color_str = path.iter().nth(1).unwrap();
        let color = match color_from_str(color_str) {
            Ok(color) => color,
            Err(_) => {
                return Err(ENOENT);
            },
        };

        let image = ImageBuffer::from_pixel(1, 1, color);
        let mut buf: Vec<u8> = Vec::new();
        let encoder = PNGEncoder::new(&mut buf);
        encoder.encode(
            &image.into_vec(),
            1, 2,
            ColorType::RGB(16),
        );

        Ok(buf)
    }

    fn getattr(&self, _req: RequestInfo, path: &Path, _fh: Option<u64>) -> ResultEntry {
        unimplemented!();
    }
}

fn color_from_str(s: &OsStr) -> Result<Rgb<u8>, ()> {
    let s = s.to_string_lossy();

    if s.len() != 6 {
        return Err(());
    }

    let r = match s[0..2].parse::<u8>() {
        Ok(color) => color,
        Err(_) => {
            return Err(());
        },
    };

    let g = match s[2..4].parse::<u8>() {
        Ok(color) => color,
        Err(_) => {
            return Err(());
        },
    };

    let b = match s[4..6].parse::<u8>() {
        Ok(color) => color,
        Err(_) => {
            return Err(());
        },
    };

    Ok(Rgb::from_channels(r, g, b, 0))
}

fn main() {
    let mut mountpoint = match args_os().nth(1) {
        Some(arg) => PathBuf::from(arg),
        None => {
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

    let _ = mount(FuseMT::new(ColorFs, 0), &mountpoint, &[]);
}
