use crate::pdf::pdf_dictionary::PdfDictionary;
use crate::pdf::pdf_name::PdfName;
use crate::pdf::pdf_number::PdfNumber;
use crate::pdf::pdf_object::PdfObject;
use std::collections::HashMap;
pub struct PrStream {
    pub offset: u64,
    pub length: Option<u64>,
    pub obj_num: Option<u64>,
    pub obj_gen: Option<u64>,
    pub compressed: Option<bool>,
    pub stream_bytes: Option<u16>,
    pub dictionary_type: crate::pdf::pdf_name::PdfName,
    pub hash_map: HashMap<PdfName, PdfObject>,
}

impl PrStream {
    pub fn new_with_offset(offset: u64) -> Self {
        PrStream {
            offset,
            length: None,
            obj_num: None,
            obj_gen: None,
            compressed: None,
            stream_bytes: None,
            dictionary_type: PdfName::Other("".to_string()),
            hash_map: HashMap::new(),
        }
    }

    pub fn put_all(&mut self, dict: PdfDictionary) {
        for (key, value) in dict.hash_map.iter() {
            self.hash_map.insert(*key, *value);
        }
    }

    pub fn set_obj_num(&mut self, obj_num: u64, obj_gen: u64) {
        self.obj_num = Some(obj_num);
        self.obj_gen = Some(obj_gen);
    }

    pub fn set_length(&mut self, length: u64) {
        self.length = Some(length);
        self.hash_map.insert(
            PdfName::Length,
            PdfObject::Number(PdfNumber {
                value: length as f64,
                bytes: vec![],
            }),
        );
    }
}
