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
use std::collections::HashMap;
pub struct Reader {
    pub tokens: crate::pdf::tokenizer::Tokenizer,
    pub new_xref_type: bool,
    pub xref: Option<u64>,
    pub obj_num: u64,
    pub obj_gen: u64,
    pub strings: Vec<crate::pdf::pdf_object::PdfObject>,
    pub visited: Vec<bool>,
    pub new_hits: HashMap<u64, u64>,
    pub trailer: Option<PdfDictionary>,
}

impl Reader {
    pub fn read_pdf(&mut self) {
        let pdf_version = self.tokens.check_pdf_header().unwrap();
    }

    pub fn read_xref(&mut self) -> Result<(), anyhow::Error> {
        let hybrid_xref = false;
        let new_xref_type = false;
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
        let last_xref = start_xref;
        let eof_pos = self.tokens.file.get_file_pointer()?;
        if self.read_x_ref_stream(start_xref)? {
            self.new_xref_type = true;
            return Ok(());
        }
        self.xref = None;
        self.tokens.file.seek(startxref)?;
        // trailer = readXrefSection();
        // PdfDictionary trailer2 = trailer;
        // while (true) {
        //     PdfNumber prev = (PdfNumber)trailer2.get(PdfName.PREV);
        //     if (prev == null)
        //         break;
        //     self.tokens.seek(prev.intValue());
        //     trailer2 = readXrefSection();
        // }
        Ok(())
    }

    pub fn read_x_ref_stream(&self, ptr: u64) -> Result<bool, anyhow::Error> {
        self.tokens.file.seek(ptr)?;
        let this_stream = 0;
        if !self.tokens.next_token()? {
            return Ok(false);
        }
        match self.tokens.token_type {
            TokenType::Number => (),
            _ => return Ok(false),
        }
        this_stream = self.tokens.int_value()?;
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
        match self.tokens.string_value {
            Some(value) if value != "obj" => return Ok(false),
            _ => (),
        }
        let object = self.read_pr_object()?;
        let stm = match object {
            PdfObject::PrStream(stm) => stm,
            _ => return Err(anyhow!("not stream")),
        };
        let index = if self.trailer.is_none() {
            let dic = PdfDictionary::new();
            dic.put_all(stm.hash_map);
            self.trailer = Some(dic);
        };
        match stm.hash_map.get(&PdfName::Length).unwrap() {
            PdfObject::Number(number) => {
                stm.set_length(number.value as u64);
            }
            _ => (),
        }
        let size = stm.hash_map.get(&PdfName::Size).unwrap();
        let obj = stm.hash_map.get(&PdfName::Index);
        let index = obj.is_none() {
            let idx = Array {array_list: vec![]}
        }
        Ok(true)
    }

    pub fn read_pr_object(&self) -> Result<PdfObject, anyhow::Error> {
        self.tokens.next_valid_token()?;
        let token_type = self.tokens.token_type;
        match self.tokens.token_type {
            TokenType::StartDic => {
                let dic = PdfDictionary::new();
                let pos = self.tokens.file.get_file_pointer()?;
                match self.tokens.string_value {
                    Some(value) if self.tokens.next_token()? && value == "stream" => {
                        let ch = self.tokens.file.read();
                        while let Some(ch) = self.tokens.file.read() {
                            if ch == '\n' {
                                break;
                            }
                        }
                        let stream =
                            PrStream::new_with_offset(self.tokens.file.get_file_pointer()?);
                        stream.put_all(dic);
                        stream.set_obj_num(self.obj_num, self.obj_gen);
                        return Ok(PdfObject::PrStream(stream));
                    }
                    _ => {
                        self.tokens.file.seek(pos);
                        return Ok(PdfObject::Dictionary(dic));
                    }
                }
            }
            TokenType::StartArray => return Ok(PdfObject::Array(self.read_array()?)),
            TokenType::Number => {
                return Ok(PdfObject::Number(PdfNumber::new(
                    self.tokens.string_value.unwrap(),
                )))
            }
            TokenType::String => {
                let mut str = PdfString::default();
                str.value = self.tokens.string_value;
                str.obj_gen = self.obj_gen;
                str.obj_num = self.obj_num;
                if !self.strings.is_empty() {
                    self.strings.push(PdfObject::String(str))
                }
                return Ok(PdfObject::String(str));
            }
            TokenType::Name => {
                return Ok(PdfObject::Name(PdfName::Other(
                    self.tokens.string_value.unwrap(),
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
                    self.tokens.string_value.unwrap(),
                )))
            }
        }
    }

    pub fn read_dictionary(&self) -> Result<PdfDictionary, anyhow::Error> {
        let dic = PdfDictionary::new();
        loop {
            self.tokens.next_valid_token();
            match self.tokens.token_type {
                TokenType::EndDic => break,
                TokenType::Name => (),
                _ => return Err(anyhow!("Dictionary key is not a name.")),
            }
            let name = crate::pdf::pdf_name::PdfName::Other(self.tokens.string_value.unwrap());
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

    pub fn read_array(&self) -> Result<Array, anyhow::Error> {
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
}
