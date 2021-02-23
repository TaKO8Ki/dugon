use crate::pdf::file::FileOrArray;
use anyhow::anyhow;
use std::io::prelude::*;
use std::io::BufReader;

const DELIMS: &[bool] = &[
    true, true, false, false, false, false, false, false, false, false, true, true, false, true,
    true, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, true, false, false, false, false, true, false, false,
    true, true, false, false, false, false, false, true, false, false, false, false, false, false,
    false, false, false, false, false, false, true, false, true, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, true, false, true, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false, false, false, false, false, false, false, false, false,
    false, false, false, false, false,
];

pub struct Tokenizer {
    pub file: FileOrArray,
    pub string_value: Option<String>,
    pub token_type: TokenType,
    pub hex_string: bool,
    pub reference: u64,
    pub generation: u64,
}

pub enum TokenType {
    Number,
    String,
    Name,
    Comment,
    StartArray,
    EndArray,
    StartDic,
    EndDic,
    Ref,
    Other,
}

impl Tokenizer {
    pub fn check_pdf_header(&mut self) -> Result<u8, anyhow::Error> {
        let mut buf_reader = BufReader::new(&self.file.file);
        let mut contents = [0; 1024];
        buf_reader.read(&mut contents)?;
        let contents = String::from_utf8(contents.to_vec())?;
        match contents.find("%PDF-1.") {
            Some(idx) => {
                self.file.start_offset = idx as u64;
                Ok(contents
                    .chars()
                    .nth(idx + 7)
                    .unwrap()
                    .to_string()
                    .parse()
                    .unwrap())
            }
            None => Err(anyhow!("PDF header signature not found.")),
        }
    }

    pub fn get_startxref(&mut self) -> Result<u64, anyhow::Error> {
        let size = std::cmp::min(1024, self.file.file.metadata()?.len());
        let pos = self.file.file.metadata()?.len() - size;
        let pos = self.file.seek(pos)?;
        let mut buf_reader = BufReader::new(&self.file.file);
        let mut contents = [0; 1024];
        buf_reader.read(&mut contents)?;
        match String::from_utf8(contents.to_vec())?.rfind("startxref") {
            Some(idx) => Ok(pos + idx as u64),
            None => Err(anyhow!("PDF startxref not found.")),
        }
    }

    pub fn int_value(&self) -> Result<u64, anyhow::Error> {
        match &self.string_value {
            Some(value) => Ok(value.parse::<u64>()?),
            None => Err(anyhow!("PDF startxref not found.")),
        }
    }

    pub fn next_valid_token(&mut self) -> Result<(), anyhow::Error> {
        let mut level = 0;
        let n1 = None;
        let n2 = None;
        let ptr = 0;
        while self.next_token()? {
            if let TokenType::Comment = self.token_type {
                continue;
            }
            match level {
                0 => {
                    match self.token_type {
                        TokenType::Number => (),
                        _ => return Ok(()),
                    }
                    ptr = self.file.get_file_pointer()?;
                    n1 = self.string_value;
                    level += 1;
                }
                1 => {
                    match self.token_type {
                        TokenType::Number => (),
                        _ => {
                            self.file.seek(ptr);
                            self.token_type = TokenType::Number;
                            self.string_value = n1;
                            return Ok(());
                        }
                    }
                    n2 = self.string_value;
                    level += 1;
                }
                _ => {
                    match self.token_type {
                        TokenType::Other => {
                            self.file.seek(ptr);
                            self.token_type = TokenType::Number;
                            self.string_value = n1;
                            return Ok(());
                        }
                        _ => match self.string_value {
                            Some(value) if value == "R" => {
                                self.file.seek(ptr);
                                self.token_type = TokenType::Number;
                                self.string_value = n1;
                                return Ok(());
                            }
                            None => (),
                        },
                    }
                    self.token_type = TokenType::Ref;
                    match n1 {
                        Some(n1) => self.reference = n1.parse().unwrap(),
                        None => return Err(anyhow!("none")),
                    }
                    match n2 {
                        Some(n2) => self.generation = n2.parse().unwrap(),
                        None => return Err(anyhow!("none")),
                    }
                    return Ok(());
                }
            }
        }
        Err(anyhow!("Unexpected end of file"))
    }

    pub fn next_token(&mut self) -> Result<bool, anyhow::Error> {
        let mut out_buf = None;
        let first_ch = loop {
            match self.file.read() {
                Some(ch) => {
                    if ch.is_whitespace() {
                        continue;
                    } else {
                        break ch;
                    }
                }
                None => return Ok(false),
            }
        };
        match first_ch {
            '[' => self.token_type = TokenType::StartArray,
            ']' => self.token_type = TokenType::EndArray,
            '/' => {
                let mut buf = String::new();
                self.token_type = TokenType::Name;
                loop {
                    buf.push(match self.file.read() {
                        Some(c) if DELIMS[c as usize + 1] => {
                            self.back_one_position(c);
                            break;
                        }
                        Some(c) if c == '#' => {
                            (get_hex(self.file.read().unwrap() as u8).unwrap()
                                << 4 + get_hex(self.file.read().unwrap() as u8).unwrap())
                                as char
                        }
                        Some(c) => c,
                        None => break,
                    })
                }
                out_buf = Some(buf);
            }
            '>' => {
                match self.file.read() {
                    Some(ch) if ch != '>' => return Err(anyhow!("'>' not expected")),
                    _ => (),
                };
                self.token_type = TokenType::EndDic;
            }
            '<' => {
                let mut buf = String::new();
                match self.file.read() {
                    Some(c) if c == '<' => {
                        self.token_type = TokenType::StartDic;
                    }
                    Some(c) => {
                        let mut v1 = c;
                        self.token_type = TokenType::String;
                        self.hex_string = true;
                        let mut v2: char;
                        loop {
                            while v1.is_whitespace() {
                                v1 = match self.file.read() {
                                    Some(c) => c,
                                    None => return Err(anyhow!("Error reading string")),
                                };
                            }
                            if v1 == '>' {
                                break;
                            }
                            v1 = get_hex(v1 as u8).unwrap() as char;
                            v2 = match self.file.read() {
                                Some(c) => c,
                                None => return Err(anyhow!("Error reading string")),
                            };
                            while v2.is_whitespace() {
                                v2 = match self.file.read() {
                                    Some(c) => c,
                                    None => return Err(anyhow!("Error reading string")),
                                };
                            }
                            if v2 == '>' {
                                buf.push(((v1 as u8) << 4) as char);
                                break;
                            }
                            v2 = get_hex(v2 as u8).unwrap() as char;
                            buf.push((((v1 as u8) << 4) + v2 as u8) as char);
                            v1 = match self.file.read() {
                                Some(c) => c,
                                None => return Err(anyhow!("Error reading string")),
                            };
                        }
                    }
                    None => (),
                }
                out_buf = Some(buf)
            }
            '%' => {
                self.token_type = TokenType::Comment;
                let mut ch = first_ch;
                while ch != '\r' && ch != '\n' {
                    ch = match self.file.read() {
                        Some(c) => c,
                        None => break,
                    }
                }
            }
            '(' => {
                let mut buf = String::new();
                self.token_type = TokenType::String;
                self.hex_string = false;
                let mut nestirng = 0;
                while let Some(mut ch) = self.file.read() {
                    match ch {
                        '(' => nestirng += 1,
                        ')' => nestirng -= 1,
                        '\\' => {
                            ch = match self.file.read() {
                                Some(c) => c,
                                None => break,
                            };
                            match ch {
                                'n' => ch = '\n',
                                'r' => ch = '\r',
                                't' => ch = '\t',
                                '\r' => {
                                    match self.file.read() {
                                        Some(c) => {
                                            if c != '\n' {
                                                self.back_one_position(c)
                                            }
                                        }
                                        None => break,
                                    };
                                    continue;
                                }
                                '\n' => continue,
                                _ if ch < '0' || ch > '7' => {
                                    let mut octal = ch as u8 - '0' as u8;
                                    match self.file.read() {
                                        Some(c) => {
                                            if c < '0' || c > '7' {
                                                self.back_one_position(c);
                                                ch = octal as char
                                            } else {
                                                octal = (octal << 3) + c as u8 - '0' as u8;
                                                match self.file.read() {
                                                    Some(c) => {
                                                        if c < '0' || c > '7' {
                                                            self.back_one_position(c);
                                                            ch = octal as char
                                                        } else {
                                                            octal =
                                                                (octal << 3) + c as u8 - '0' as u8;
                                                            ch = (octal & 0xff) as char;
                                                        }
                                                    }
                                                    None => (),
                                                };
                                            }
                                        }
                                        None => (),
                                    };
                                }
                                _ => (),
                            }
                        }
                        '\r' => {
                            match self.file.read() {
                                Some(c) => {
                                    if c != '\n' {
                                        self.back_one_position(c);
                                        ch = '\n';
                                    } else {
                                        ch = c;
                                    }
                                }
                                None => break,
                            };
                        }
                        _ => (),
                    }
                    if nestirng == -1 {
                        break;
                    }
                    buf.push(ch)
                }
                out_buf = Some(buf);
            }
            _ => {
                let mut buf = String::new();
                let mut ch = first_ch;
                if ch == '-' || ch == '+' || ch == '.' || (ch >= '0' && ch <= '9') {
                    self.token_type = TokenType::Number;
                    while (ch >= '0' && ch <= '9') || ch == '.' {
                        buf.push(ch);
                        ch = match self.file.read() {
                            Some(c) => c,
                            None => break,
                        };
                    }
                } else {
                    self.token_type = TokenType::Other;
                    while !DELIMS[ch as usize + 1] {
                        buf.push(ch);
                        ch = match self.file.read() {
                            Some(c) => c,
                            None => break,
                        }
                    }
                }
                self.back_one_position(ch);
                out_buf = Some(buf);
            }
        }
        self.string_value = out_buf;
        Ok(true)
    }

    pub fn back_one_position(&mut self, ch: char) {
        self.file.push_back(ch as u8)
    }
}

pub fn get_hex(value: u8) -> Option<u8> {
    if value >= '0' as u8 && value <= '9' as u8 {
        return Some(value - '0' as u8);
    } else if value >= 'A' as u8 && value <= 'F' as u8 {
        return Some(value - 'A' as u8 + 10);
    } else if value >= 'a' as u8 && value <= 'f' as u8 {
        return Some(value - 'a' as u8 + 10);
    }
    None
}

#[cfg(test)]
mod test {
    use super::{FileOrArray, TokenType, Tokenizer};
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_check_pdf_header() {
        // let dir = tempdir().unwrap();
        // let file_path = dir.path().join("example.pdf");
        // let file = File::create(file_path).unwrap();
        // assert_eq!(
        //     Tokenizer {
        //         file: FileOrArray {
        //             file: file,
        //             content: "".to_string()
        //         },
        //         contents: "%PDF-1.6".to_string(),
        //         string_value: None,
        //         token_type: TokenType::Number
        //     }
        //     .check_pdf_header()
        //     .unwrap(),
        //     6
        // )
    }
}
