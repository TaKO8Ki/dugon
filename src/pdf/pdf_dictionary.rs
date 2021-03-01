use crate::pdf::pdf_name::PdfName;
use crate::pdf::pdf_object::PdfObject;
use std::collections::HashMap;

#[derive(Clone)]
pub struct PdfDictionary {
    pub dictionary_type: crate::pdf::pdf_name::PdfName,
    pub hash_map: HashMap<PdfName, PdfObject>,
}

impl PdfDictionary {
    pub fn new() -> Self {
        PdfDictionary {
            dictionary_type: crate::pdf::pdf_name::PdfName::A,
            hash_map: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: PdfName, value: PdfObject) {
        self.hash_map.insert(key, value);
    }

    pub fn put_all(&mut self, hash_map: HashMap<PdfName, PdfObject>) {
        for (key, value) in hash_map.iter() {
            self.hash_map.insert(key.clone(), value.clone());
        }
    }
}
