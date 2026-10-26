//! Bounds-checked canonical integer decoding for compact reason records.
use super::MindRecordError;
pub(super) struct Reader<'a> {
    pub bytes: &'a [u8],
    pub at: usize,
}
impl Reader<'_> {
    pub fn byte(&mut self) -> Result<u8, MindRecordError> {
        let value = self
            .bytes
            .get(self.at)
            .copied()
            .ok_or(MindRecordError::Invalid("truncated"))?;
        self.at += 1;
        Ok(value)
    }
    pub fn unsigned(&mut self) -> Result<u64, MindRecordError> {
        let mut value = 0;
        for index in 0..10 {
            let byte = self.byte()?;
            if index == 9 && byte > 1 {
                return Err(MindRecordError::Invalid("varint overflow"));
            }
            value |= u64::from(byte & 127) << (index * 7);
            if byte < 128 {
                if index > 0 && byte == 0 {
                    return Err(MindRecordError::Invalid("noncanonical varint"));
                }
                return Ok(value);
            }
        }
        Err(MindRecordError::Invalid("varint overflow"))
    }
    pub fn signed(&mut self) -> Result<i64, MindRecordError> {
        let raw = self.unsigned()?;
        let magnitude =
            i64::try_from(raw >> 1).map_err(|_| MindRecordError::Invalid("signed overflow"))?;
        Ok(if raw & 1 == 0 { magnitude } else { !magnitude })
    }
    pub fn optional(&mut self) -> Result<Option<u64>, MindRecordError> {
        match self.byte()? {
            0 => Ok(None),
            1 => Ok(Some(self.unsigned()?)),
            _ => Err(MindRecordError::Invalid("option tag")),
        }
    }
    pub fn slot(&mut self) -> Result<usize, MindRecordError> {
        usize::try_from(self.unsigned()?).map_err(|_| MindRecordError::Invalid("slot overflow"))
    }
}
