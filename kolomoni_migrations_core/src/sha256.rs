use std::fmt::Display;

use sha2::{Digest, Sha256};


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sha256Hash {
    data: [u8; 32],
}

impl Sha256Hash {
    pub(crate) fn try_from_vec(vec: Vec<u8>) -> Result<Self, ()> {
        Ok(Self {
            data: vec.try_into().map_err(|_| ())?,
        })
    }

    /// Calculate SHA-256 (SHA-2, not SHA-3) of the provided byte slice.
    pub fn calculate(bytes: &[u8]) -> Self {
        let raw_data = <Sha256 as Digest>::digest(bytes);

        Self {
            data: raw_data.into(),
        }
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { data: bytes }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl Display for Sha256Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.data {
            write!(f, "{:X}", byte)?;
        }

        Ok(())
    }
}
