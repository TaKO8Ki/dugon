use anyhow::anyhow;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

pub const TK_NUMBER: u8 = 1;

pub struct Tokenizer {
    pub file: File,
    pub contents: String,
    pub string_value: Option<String>,
    pub token_type: u8,
}

impl Tokenizer {
    pub fn header(&self) -> Result<u8, String> {
        match self.contents.find("%PDF-1.") {
            Some(idx) => Ok(self
                .contents
                .chars()
                .nth(idx + 7)
                .unwrap()
                .to_string()
                .parse()
                .unwrap()),
            None => Err("PDF header signature not found.".to_string()),
        }
    }

    pub fn get_startxref(&mut self) -> Result<u64, anyhow::Error> {
        let mut buf_reader = BufReader::new(&self.file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents)?;
        let size = std::cmp::min(1024, self.file.metadata()?.len());
        let pos = self.file.metadata()?.len() - size;
        self.file.seek(std::io::SeekFrom::Start(pos))?;
        match contents.rfind("startxref") {
            Some(idx) => Ok(pos + idx as u64),
            None => Err(anyhow!("PDF startxref not found.")),
        }
    }

    pub fn seek(&mut self, pos: u64) -> Result<(), anyhow::Error> {
        self.file.seek(std::io::SeekFrom::Start(pos))?;
        Ok(())
    }

    pub fn int_value(&self) -> Result<u64, anyhow::Error> {
        match &self.string_value {
            Some(value) => Ok(value.parse::<u64>()?),
            None => Err(anyhow!("PDF startxref not found.")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Tokenizer;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_header() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("example.pdf");
        let file = File::create(file_path).unwrap();
        assert_eq!(
            Tokenizer {
                file: file,
                contents: "%PDF-1.6".to_string(),
                string_value: None,
                token_type: 1
            }
            .header()
            .unwrap(),
            6
        )
    }
}
