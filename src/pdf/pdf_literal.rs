use super::tokenizer;

#[derive(Clone)]
pub struct PdfLiteral {
    pub token_type: crate::pdf::tokenizer::TokenType,
    pub bytes: Vec<u8>,
}

impl PdfLiteral {
    pub fn new(token_type: crate::pdf::tokenizer::TokenType, content: String) -> Self {
        PdfLiteral {
            token_type,
            bytes: content.as_bytes().to_vec(),
        }
    }
}
