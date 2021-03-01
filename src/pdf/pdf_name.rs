#[derive(Hash, Eq, PartialEq, Clone)]
pub enum PdfName {
    A,
    Aa,
    Absolutecalorimetric,
    Ac,
    Length,
    Size,
    Index,
    W,
    Prev,
    Xref,
    XrefStm,
    Type,
    Other(String),
}

impl PdfName {
    fn new<'a>(self) -> Vec<u8> {
        let name = match self {
            PdfName::A => "A".to_string(),
            PdfName::Aa => "AA".to_string(),
            PdfName::Other(name) => name,
            _ => "b".to_string(),
        };
        let length = name.len();
        let mut pdf_name = String::new();
        pdf_name.push('/');
        let mut character: char;
        for ch in name.chars() {
            character = ((ch as u8) & 0xff) as char;
            match character {
                ' ' | '%' | '(' | ')' | '<' | '>' | '[' | ']' | '{' | '}' | '/' => (),
                '#' => {
                    pdf_name.push('#');
                    pdf_name.push(character);
                    break;
                }
                _ => {
                    if (character as u8) > 126 || (character as u8) < 32 {
                        pdf_name.push('#');
                        if (character as u8) < 16 {
                            pdf_name.push('0');
                        }
                        pdf_name.push(character);
                    } else {
                        pdf_name.push(character);
                    }
                }
            }
        }
        pdf_name.as_bytes().to_vec()
    }
}
