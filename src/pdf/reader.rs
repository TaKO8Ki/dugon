use anyhow::anyhow;

use super::tokenizer;
pub struct Reader {
    pub tokens: crate::pdf::tokenizer::Tokenizer,
    pub new_xref_type: bool,
    pub xref: Option<u64>,
}

impl Reader {
    pub fn read_pdf(&self) {
        let pdf_version = self.tokens.check_pdf_header().unwrap();
    }

    pub fn read_xref(&mut self) -> Result<(), anyhow::Error> {
        let hybrid_xref = false;
        let new_xref_type = false;
        let startxref = self.tokens.get_startxref()?;
        self.tokens.seek(startxref)?;
        // self.tokens.nextToken();
        if &self.tokens.string_value.clone().unwrap() != "startxref" {
            return Err(anyhow!("startxref not found."));
        }
        // self.tokens.nextToken();
        if self.tokens.token_type != tokenizer::TK_NUMBER {
            return Err(anyhow!("startxref is not followed by a number."));
        }
        let start_xref = self.tokens.int_value()?;
        let last_xref = start_xref;
        let eof_pos = self.tokens.get_file_pointer()?;
        if self.read_x_ref_stream(start_xref) {
            self.new_xref_type = true;
            return Ok(());
        }
        self.xref = None;
        self.tokens.seek(startxref)?;
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

    pub fn read_x_ref_stream(&self, start_xref: u64) -> bool {
        true
    }
}
