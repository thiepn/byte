use crate::core::error::ByteError;
use std::{fs::File, io::Read, path::Path};

pub enum BoundedText {
    Missing,
    Present(String),
    Invalid,
}

pub fn read_bounded_text(path: &Path, max_bytes: u64) -> Result<BoundedText, ByteError> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(BoundedText::Missing);
        }
        Err(error) => return Err(error.into()),
    };

    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;

    if bytes.len() as u64 > max_bytes {
        return Ok(BoundedText::Invalid);
    }

    match String::from_utf8(bytes) {
        Ok(text) => Ok(BoundedText::Present(text)),
        Err(_) => Ok(BoundedText::Invalid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn rejects_oversized_and_invalid_utf8_without_unbounded_reads() {
        let temp = tempfile::tempdir().expect("temp");
        let large = temp.path().join("large.json");
        fs::write(&large, vec![b'x'; 33]).expect("write");
        assert!(matches!(
            read_bounded_text(&large, 32).expect("read"),
            BoundedText::Invalid
        ));

        let invalid = temp.path().join("invalid.json");
        fs::write(&invalid, [0xff, 0xfe]).expect("write");
        assert!(matches!(
            read_bounded_text(&invalid, 32).expect("read"),
            BoundedText::Invalid
        ));
    }

    #[test]
    fn distinguishes_missing_and_present_files() {
        let temp = tempfile::tempdir().expect("temp");
        assert!(matches!(
            read_bounded_text(&temp.path().join("missing"), 32).expect("read"),
            BoundedText::Missing
        ));

        let file = temp.path().join("ok");
        fs::write(&file, "{}").expect("write");
        assert!(matches!(
            read_bounded_text(&file, 32).expect("read"),
            BoundedText::Present(value) if value == "{}"
        ));
    }
}
