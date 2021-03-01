#[derive(Clone)]
pub struct PdfNumber {
    pub value: f64,
    pub bytes: Vec<u8>,
}

impl PdfNumber {
    pub fn new(content: String) -> Self {
        match content.parse() {
            Ok(value) => PdfNumber {
                value,
                bytes: vec![],
            },
            Err(err) => panic!(err),
        }
    }
}
