#[derive(Clone)]
pub struct PdfString {
    pub hex_writing: bool,
    pub encoding: String,
    pub obj_num: u64,
    pub obj_gen: u64,
    pub value: Option<String>,
}

impl Default for PdfString {
    fn default() -> Self {
        PdfString {
            hex_writing: false,
            obj_num: 0,
            obj_gen: 0,
            encoding: "PDF".to_string(),
            value: None,
        }
    }
}
