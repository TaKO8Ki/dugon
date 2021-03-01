use anyhow::anyhow;
use std::fs::File;
use std::io::prelude::*;

pub struct FileOrArray {
    pub file: File,
    pub content: String,
    pub file_name: String,
    pub is_back: bool,
    pub back: u8,
    pub array_in: Vec<u8>,
    pub array_in_ptr: u64,
    pub start_offset: u64,
}

impl FileOrArray {
    pub fn push_back(&mut self, b: u8) {
        self.back = b;
        self.is_back = true
    }

    pub fn read(&mut self) -> Option<char> {
        if self.is_back {
            self.is_back = false;
            return Some((self.back & 0xff) as char);
        }
        if self.array_in.is_empty() {
            let mut contents = [0; 1];
            self.file.read(&mut contents);
            Some(contents[0] as char)
        } else {
            if self.array_in_ptr as usize >= self.array_in.len() {
                return None;
            }
            self.array_in_ptr += 1;
            Some((self.array_in[self.array_in_ptr as usize] & 0xff) as char)
        }
    }

    pub fn get_file_pointer(&mut self) -> Result<u64, anyhow::Error> {
        match self.file.seek(std::io::SeekFrom::Current(0)) {
            Ok(pos) => Ok(pos),
            Err(err) => Err(anyhow!(err)),
        }
    }

    pub fn seek(&mut self, pos: u64) -> Result<u64, anyhow::Error> {
        let pos = pos + self.start_offset;
        self.is_back = false;
        if self.array_in.is_empty() {
            // insureOpen();
            self.file.seek(std::io::SeekFrom::Start(pos))?;
        } else {
            self.array_in_ptr = pos;
        }
        Ok(pos)
    }

    // pub fn seek() -> u8 {
    // pos += startOffset;
    // isBack = false;
    // if (arrayIn == null) {
    //     insureOpen();
    //     rf.seek(pos);
    // }
    // else
    //     arrayInPtr = pos;
    // }
}
