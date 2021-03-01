use crate::pdf::{
    array::Array, pdf_dictionary::PdfDictionary, pdf_literal::PdfLiteral, pdf_name::PdfName,
    pdf_number::PdfNumber, pdf_string::PdfString, pr_indirect_reference::PrIndirectReference,
    pr_stream::PrStream,
};

#[derive(Clone)]
pub enum PdfObject {
    Boolean,
    Number(PdfNumber),
    String(PdfString),
    Name(PdfName),
    Array(Array),
    XArray(Vec<u64>),
    Dictionary(PdfDictionary),
    PrStream(PrStream),
    Null,
    MNull,
    Indirect(PrIndirectReference),
    Literal(PdfLiteral),
    Nothing(String),
    TextPdfdocencoding(String),
    TextUnicode(String),
}
