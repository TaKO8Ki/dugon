use anyhow::anyhow;

use super::tokenizer;
pub struct Reader {
    pub tokens: crate::pdf::tokenizer::Tokenizer,
}

impl Reader {
    pub fn read_pdf(&self) {
        let pdf_version = self.tokens.header().unwrap();
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
        let startxref = self.tokens.int_value();
        let lastXref = startxref;
        // eofPos = self.tokens.getFilePointer();
        // try {
        //     if (readXRefStream(startxref)) {
        //         newXrefType = true;
        //         return;
        //     }
        // }
        // catch (Exception e) {}
        // xref = null;
        // self.tokens.seek(startxref);
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
}
