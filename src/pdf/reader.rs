use crate::pdf::array::Array;
use crate::pdf::pdf_dictionary::PdfDictionary;
use crate::pdf::pdf_literal::PdfLiteral;
use crate::pdf::pdf_name::PdfName;
use crate::pdf::pdf_number::PdfNumber;
use crate::pdf::pdf_object::PdfObject;
use crate::pdf::pdf_string::PdfString;
use crate::pdf::pr_indirect_reference::PrIndirectReference;
use crate::pdf::pr_stream::PrStream;
use crate::pdf::tokenizer::{self, TokenType};
use anyhow::anyhow;
use std::{collections::HashMap, vec};
pub struct Reader {
    pub tokens: crate::pdf::tokenizer::Tokenizer,
    pub new_xref_type: bool,
    pub obj_num: u64,
    pub obj_gen: u64,
    pub strings: Vec<crate::pdf::pdf_object::PdfObject>,
    pub visited: Vec<bool>,
    pub new_hits: HashMap<u64, u64>,
    pub trailer: Option<PdfDictionary>,
    pub xref: Vec<i64>,
    pub obj_stm_mark: Option<HashMap<u64, u64>>,
    pub obj_stm_to_offset: Option<HashMap<u64, u64>>,
    pub partial: bool,
    pub hybrid_xref: bool,
    pub eof_pos: u64,
    pub last_xref: u64,
}

impl Reader {
    pub fn read_pdf(&mut self) {
        let pdf_version = self.tokens.check_pdf_header().unwrap();
    }

    pub fn read_xref(&mut self) -> Result<(), anyhow::Error> {
        self.hybrid_xref = false;
        self.new_xref_type = false;
        let startxref = self.tokens.get_startxref()?;
        self.tokens.file.seek(startxref)?;
        self.tokens.next_token()?;
        if &self.tokens.string_value.clone().unwrap() != "startxref" {
            return Err(anyhow!("startxref not found."));
        }
        self.tokens.next_token()?;
        match self.tokens.token_type {
            tokenizer::TokenType::Number => {
                return Err(anyhow!("startxref is not followed by a number."))
            }
            _ => (),
        }
        let start_xref = self.tokens.int_value()?;
        self.last_xref = start_xref;
        self.eof_pos = self.tokens.file.get_file_pointer()?;
        if self.read_x_ref_stream(start_xref)? {
            self.new_xref_type = true;
            return Ok(());
        }
        self.xref = vec![];
        self.tokens.file.seek(startxref)?;
        self.trailer = Some(self.read_xref_section()?);
        let mut trailer2 = self.trailer.clone().unwrap();
        loop {
            let prev = match trailer2.hash_map.get(&PdfName::Prev) {
                Some(PdfObject::Number(number)) => number,
                _ => break,
            };
            self.tokens.file.seek(prev.value as u64)?;
            trailer2 = self.read_xref_section()?;
        }
        Ok(())
    }

    pub fn read_xref_section(&mut self) -> Result<PdfDictionary, anyhow::Error> {
        self.tokens.next_valid_token()?;
        match self.tokens.string_value.clone() {
            Some(value) if value == "xref" => return Err(anyhow!("xref subsection not found")),
            _ => (),
        }
        let mut start: u64;
        let mut end: u64;
        let mut pos: u64;
        let mut gen: u64;
        loop {
            match &self.tokens.string_value {
                Some(value) if value == "trailer" => break,
                _ => (),
            }
            match self.tokens.token_type.clone() {
                TokenType::Number => (),
                _ => {
                    return Err(anyhow!(
                        "Object number of the first object in this xref subsection not found"
                    ))
                }
            }
            start = self.tokens.int_value()?;
            self.tokens.next_valid_token()?;
            match self.tokens.token_type {
                TokenType::Number => (),
                _ => {
                    return Err(anyhow!(
                        "Number of entries in this xref subsection not found"
                    ))
                }
            }
            end = self.tokens.int_value()? + start;
            if start == 1 {
                let back = self.tokens.file.get_file_pointer()?;
                self.tokens.next_valid_token()?;
                pos = self.tokens.int_value()?;
                self.tokens.next_valid_token()?;
                gen = self.tokens.int_value()?;
                if pos == 0 && gen == 65535 {
                    start -= 1;
                    end -= 1;
                }
                self.tokens.file.seek(back)?;
            }
            self.ensure_xref_size(end as usize * 2);
            for k in start..end {
                self.tokens.next_valid_token()?;
                pos = self.tokens.int_value()?;
                self.tokens.next_valid_token()?;
                gen = self.tokens.int_value()?;
                self.tokens.next_valid_token()?;
                let p = k * 2;
                match self.tokens.string_value.clone() {
                    Some(value) if value == "n" => {
                        if self.xref[p as usize] == 0 && self.xref[p as usize + 1] == 0 {
                            self.xref[p as usize] = pos as i64;
                        }
                    }
                    Some(value) if value == "f" => {
                        if self.xref[p as usize] == 0 && self.xref[p as usize + 1] == 0 {
                            self.xref[p as usize] = -1;
                        }
                    }
                    _ => {
                        return Err(anyhow!(
                            "Invalid cross-reference entry in this xref subsection"
                        ))
                    }
                }
            }
        }
        let trailer = match self.read_pr_object()? {
            PdfObject::Dictionary(dictionary) => dictionary,
            _ => return Err(anyhow!("dictionary doesn't exist")),
        };
        let xref_size = match trailer.hash_map.get(&PdfName::Size) {
            Some(PdfObject::Number(number)) => number,
            _ => return Err(anyhow!("size doesn't exist")),
        };
        self.ensure_xref_size(xref_size.value as usize * 2);
        match trailer.hash_map.get(&PdfName::XrefStm) {
            Some(PdfObject::Number(number)) => {
                let loc = number.value as u64;
                match self.read_x_ref_stream(loc) {
                    Ok(_) => {
                        self.new_xref_type = true;
                        self.hybrid_xref = true
                    }
                    Err(err) => {
                        self.xref = vec![];
                        return Err(err);
                    }
                }
            }
            _ => return Err(anyhow!("size doesn't exist")),
        };
        Ok(trailer)
    }

    pub fn read_x_ref_stream(&mut self, ptr: u64) -> Result<bool, anyhow::Error> {
        self.tokens.file.seek(ptr)?;
        if !self.tokens.next_token()? {
            return Ok(false);
        }
        match self.tokens.token_type {
            TokenType::Number => (),
            _ => return Ok(false),
        }
        let mut this_stream = self.tokens.int_value()?;
        if !self.tokens.next_token()? {
            return Ok(false);
        }
        match self.tokens.token_type {
            TokenType::Number => (),
            _ => return Ok(false),
        }
        if !self.tokens.next_token()? {
            return Ok(false);
        }
        match self.tokens.string_value.clone() {
            Some(value) if value != "obj" => return Ok(false),
            _ => (),
        }
        let object = self.read_pr_object()?;
        let mut stm = match object {
            PdfObject::PrStream(stm) => {
                let token_type = match stm.hash_map.get(&PdfName::Type) {
                    Some(value) => value,
                    None => return Err(anyhow!("type doesn't exist")),
                };
                if let PdfObject::Name(PdfName::Xref) = token_type {
                    return Ok(false);
                }
                stm
            }
            _ => return Err(anyhow!("not stream")),
        };
        let index = if self.trailer.is_none() {
            let mut dic = PdfDictionary::new();
            dic.put_all(stm.hash_map.clone());
            self.trailer = Some(dic);
        };
        match stm.hash_map.get(&PdfName::Length).unwrap() {
            PdfObject::Number(number) => {
                stm.set_length(number.value as u64);
            }
            _ => (),
        }
        let size = match stm.clone().hash_map.get(&PdfName::Size) {
            Some(PdfObject::Number(number)) => number.value as usize,
            _ => return Err(anyhow!("number doesn't exist")),
        };
        let stm = stm.clone();
        let obj = stm.hash_map.get(&PdfName::Index);
        let index = if let Some(PdfObject::Array(array)) = obj {
            array.clone()
        } else {
            let mut idx = Array { array_list: vec![] };
            idx.array_list
                .push(PdfObject::XArray(Vec::with_capacity(size)));
            idx
        };
        let stm = stm.clone();
        let w = match stm.hash_map.get(&PdfName::W) {
            Some(PdfObject::Array(array)) => array,
            _ => return Err(anyhow!("array doesn't exist")),
        };
        let obj = stm.hash_map.get(&PdfName::Prev);
        let prev = if let Some(PdfObject::Number(number)) = obj {
            number.value as i64
        } else {
            -1
        };
        self.ensure_xref_size(size * 2);
        if self.obj_stm_mark.is_none() && !self.partial {
            self.obj_stm_mark = Some(HashMap::new());
        }
        if self.obj_stm_to_offset.is_none() && self.partial {
            self.obj_stm_to_offset = Some(HashMap::new());
        }
        let b = Vec::<u8>::new();
        let mut bptr = 0;
        let wa = w.array_list.clone();
        let mut wc = Vec::with_capacity(3);
        for k in 0..2 {
            wc[k] = match wa[k].clone() {
                PdfObject::Number(number) => number.value as u64,
                _ => return Err(anyhow!("number doesn't exist")),
            }
        }
        let sections = index.array_list;
        for (index, sec) in sections.iter().step_by(2).enumerate() {
            let mut start = match sec {
                PdfObject::Number(number) => number.value as u64,
                _ => return Err(anyhow!("number doesn't exist")),
            };
            let mut length = match sections[index + 1].clone() {
                PdfObject::Number(number) => number.value as u64,
                _ => return Err(anyhow!("number doesn't exist")),
            };
            self.ensure_xref_size((start + length) as usize * 2);
            while length > 0 {
                length -= 1;
                let token_type = if wc[0] > 0 {
                    let mut r#type = 0;
                    for k in 0..wc[0] {
                        bptr += 1;
                        r#type = (r#type << 8) + (b[bptr] & 0xff) as u64
                    }
                    r#type
                } else {
                    1
                };
                let mut field2 = 0;
                for k in 0..wc[1] {
                    bptr += 1;
                    field2 = (field2 << 8) + (b[bptr] & 0xff) as u64;
                }
                let mut field3 = 0;
                for k in 0..wc[2] {
                    bptr += 1;
                    field3 = (field3 << 8) + (b[bptr] & 0xff) as u64;
                }
                let base = start * 2;
                if self.xref[base as usize] == 0 && self.xref[base as usize + 1] == 0 {
                    match token_type {
                        0 => self.xref[base as usize] = -1,
                        1 => self.xref[base as usize] = field2 as i64,
                        2 => {
                            self.xref[base as usize] = field3 as i64;
                            self.xref[base as usize + 1] = field2 as i64;
                            if self.partial {
                                match &mut self.obj_stm_to_offset {
                                    Some(hashmap) => hashmap.insert(field2, 0),
                                    None => return Err(anyhow!("error")),
                                };
                            } else {
                                // let on = field2.clone();
                                // match self.obj_stm_mark {
                                //     Some(obj_stm_mark) => match obj_stm_mark.get(&on) {
                                //         Some(value) => value
                                //     },
                                //     None => return Err(anyhow!("error")),
                                // };
                            }
                        }
                        _ => (),
                    }
                }
                start += 1
            }
        }
        this_stream *= 2;
        if (this_stream as usize) < self.xref.len() {
            self.xref[this_stream as usize] = -1;
        }
        if prev == -1 {
            return Ok(true);
        }
        Ok(self.read_x_ref_stream(prev as u64)?)
    }

    pub fn read_pr_object(&mut self) -> Result<PdfObject, anyhow::Error> {
        self.tokens.next_valid_token()?;
        let token_type = self.tokens.token_type.clone();
        match self.tokens.token_type {
            TokenType::StartDic => {
                let dic = PdfDictionary::new();
                let pos = self.tokens.file.get_file_pointer()?;
                match self.tokens.string_value.clone() {
                    Some(value) if self.tokens.next_token()? && value == "stream" => {
                        let ch = self.tokens.file.read();
                        while let Some(ch) = self.tokens.file.read() {
                            if ch == '\n' {
                                break;
                            }
                        }
                        let mut stream =
                            PrStream::new_with_offset(self.tokens.file.get_file_pointer()?);
                        stream.put_all(dic);
                        stream.set_obj_num(self.obj_num, self.obj_gen);
                        return Ok(PdfObject::PrStream(stream));
                    }
                    _ => {
                        self.tokens.file.seek(pos)?;
                        return Ok(PdfObject::Dictionary(dic));
                    }
                }
            }
            TokenType::StartArray => return Ok(PdfObject::Array(self.read_array()?)),
            TokenType::Number => {
                return Ok(PdfObject::Number(PdfNumber::new(
                    self.tokens.string_value.clone().unwrap(),
                )))
            }
            TokenType::String => {
                let mut str = PdfString::default();
                str.value = self.tokens.string_value.clone();
                str.obj_gen = self.obj_gen;
                str.obj_num = self.obj_num;
                if !self.strings.is_empty() {
                    self.strings.push(PdfObject::String(str.clone()))
                }
                return Ok(PdfObject::String(str));
            }
            TokenType::Name => {
                return Ok(PdfObject::Name(PdfName::Other(
                    self.tokens.string_value.clone().unwrap(),
                )))
            }
            TokenType::Ref => {
                let num = self.tokens.reference;
                let reference = PrIndirectReference {
                    number: num,
                    generation: self.tokens.generation,
                };
                if !self.visited.is_empty() && !self.visited[num as usize] {
                    self.visited[num as usize] = true;
                    self.new_hits.insert(num, 1);
                }
                return Ok(PdfObject::Indirect(reference));
            }
            _ => {
                return Ok(PdfObject::Literal(PdfLiteral::new(
                    token_type,
                    self.tokens.string_value.clone().unwrap(),
                )))
            }
        }
    }

    pub fn read_dictionary(&mut self) -> Result<PdfDictionary, anyhow::Error> {
        let mut dic = PdfDictionary::new();
        loop {
            self.tokens.next_valid_token()?;
            match self.tokens.token_type {
                TokenType::EndDic => break,
                TokenType::Name => (),
                _ => return Err(anyhow!("Dictionary key is not a name.")),
            }
            let name =
                crate::pdf::pdf_name::PdfName::Other(self.tokens.string_value.clone().unwrap());
            let obj = self.read_pr_object()?;
            // int type = obj.type();
            // if (-type == PRTokeniser.TK_END_DIC)
            //     tokens.throwError("Unexpected '>>'");
            // if (-type == PRTokeniser.TK_END_ARRAY)
            //     tokens.throwError("Unexpected ']'");
            dic.put(name, obj);
        }
        return Ok(dic);
    }

    pub fn read_array(&mut self) -> Result<Array, anyhow::Error> {
        let mut array = Array { array_list: vec![] };
        loop {
            let obj = self.read_pr_object()?;
            match obj {
                PdfObject::Literal(literal) => match literal.token_type {
                    TokenType::EndArray => break,
                    TokenType::EndDic => return Err(anyhow!("Unexpected '>>'")),
                    _ => return Err(anyhow!("Unexpected '>>'")),
                },
                _ => (),
            }
            array.array_list.push(obj)
        }
        Ok(array)
    }

    pub fn ensure_xref_size(&mut self, size: usize) {
        if size == 0 {
            return;
        }
        if self.xref.is_empty() {
            self.xref = Vec::with_capacity(size)
        }
    }
}
