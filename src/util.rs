/*  util.rs
 *
 *
 *  Copyright (C) 2015 rustybot All Rights Reserved.
 *
 *  This file is part of rustybot.
 *
 *  rustybot is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  rustybot is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with rustybot. If not, see <http://www.gnu.org/licenses/>.
 *
 */

use std::error::Error;
use std::fs::{OpenOptions, File};
use std::path::Path;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::Cursor;
use time::{get_time, Timespec, Duration};
use byteorder::{LittleEndian, ReadBytesExt};

// Number of bytes in an unsigned 32-bit integer
pub const SIZE_U32: usize = 4;

// Number of bytes in an unsigned 64-bit integer
pub const SIZE_U64: usize = 8;

/* Returns true if timestamp t has timed out in respect to the provided timeout value */
pub fn timed_out(t: Timespec, timeout: i64) -> bool
{
    t + Duration::seconds(timeout) <= get_time()
}

/*
 * Attempts to open file with path_name and return the file.
 * If create is true the file will be force created.
 */
pub fn open_file<P: AsRef<Path>>(path: P, create: bool) -> Option<File>
{
    let mut options = OpenOptions::new();

    match options.read(true).create(create).append(true).open(&path) {
        Ok(fp) => return Some(fp),
        Err(e) => {
            println!("Failed to open file with error: {}", Error::description(&e));
            return None;
        },
    };
}

/* Saves an arbitrary byte vector to path_name. */
pub fn save_data(path_name: &str, data: &Vec<u8>) -> Result<usize, String>
{
    let path = Path::new(path_name);
    let display = path.display();
    let mut options = OpenOptions::new();

    let fp = try!(options.write(true).create(true).open(&path)
                         .map_err(|e| format!("Couldn't open file {}: {}", display, Error::description(&e))));

    let mut writer = BufWriter::new(&fp);

    let size = try!(writer.write(&data)
                          .map_err(|e| format!("Couldn't write to file {}: {}", display, Error::description(&e))));
    Ok(size)
}

/* Copies up to max_bytes bytes of a string into buf. If string is smaller than max_bytes, pads with zeroes. */
pub fn string_to_nbytes(s: &str, buf: &mut Vec<u8>, max_bytes: usize)
{
    for (i, byte) in s.as_bytes().iter().enumerate() {
        if i >= max_bytes {
            break;
        }

        buf.push(*byte);
    }

    let len = s.len();

    if len >= max_bytes {
        return;
    }

    let padding = max_bytes - len;

    for _ in 0..padding {
        buf.push(0);
    }
}

/* Converts a unsigned 32-bit integer into bytes in little-endian order and pushes them to buf */
pub fn u32_to_bytes_le(val: u32, buf: &mut Vec<u8>)
{
    buf.push(val as u8);
    buf.push((val >>  8) as u8);
    buf.push((val >> 16) as u8);
    buf.push((val >> 24) as u8);
}

/* Converts bytes in little-endian order to an unsigned 32-bit integer*/
pub fn bytes_le_to_u32(buf: &[u8]) -> u32
{
    let mut temp = Cursor::new(buf);
    temp.read_u32::<LittleEndian>().unwrap_or(0)
}

/* Converts a unsigned 64-bit integer into bytes in little-endian order and pushes them to buf vector */
pub fn u64_to_bytes_le(val: u64, buf: &mut Vec<u8>)
{
    buf.push(val as u8);
    buf.push((val >>  8) as u8);
    buf.push((val >> 16) as u8);
    buf.push((val >> 24) as u8);
    buf.push((val >> 32) as u8);
    buf.push((val >> 40) as u8);
    buf.push((val >> 48) as u8);
    buf.push((val >> 56) as u8);
}

/* Converts bytes in little-endian order to an unsigned 64-bit integer */
pub fn bytes_le_to_u64(buf: &[u8]) -> u64
{
    let mut temp = Cursor::new(buf);
    temp.read_u64::<LittleEndian>().unwrap_or(0)
}

/* Returns the count of a given character within a string */
pub fn char_count(s: &str, c: char) -> usize
{
    let mut count = 0;

    for ch in s.chars() {
        if ch == c {
            count += 1;
        }
    }

    count
}
