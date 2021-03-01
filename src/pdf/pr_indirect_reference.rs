use crate::pdf::reader::Reader;

#[derive(Clone)]
pub struct PrIndirectReference {
    pub number: u64,
    pub generation: u64,
}
