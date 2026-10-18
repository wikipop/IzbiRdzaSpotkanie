use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use crate::SlotId;

/// Number of bytes needed for 168 bits.
const BYTES: usize = SlotId::COUNT / 8;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BitmapError {
    #[error("bitmap is not valid base64url")]
    InvalidBase64,
    #[error("bitmap has {0} bytes, expected 21")]
    InvalidLength(usize),
}

/// A set of week slots stored as 168 bits (21 bytes).
///
/// The slot with index `i` is bit `i % 8` (counting from the least
/// significant bit) of byte `i / 8`. The same layout is stored in the
/// gateway's SQLite and in the JWT `slots` claim (as unpadded base64url,
/// 28 characters).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SlotBitmap([u8; BYTES]);

impl SlotBitmap {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, slot: SlotId) {
        let index = slot.index();
        self.0[index / 8] |= 1 << (index % 8);
    }

    pub fn contains(&self, slot: SlotId) -> bool {
        let index = slot.index();
        self.0[index / 8] & (1 << (index % 8)) != 0
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|byte| *byte == 0)
    }

    pub fn len(&self) -> usize {
        self.0.iter().map(|byte| byte.count_ones() as usize).sum()
    }

    /// Slots that are set, in index order.
    pub fn iter(&self) -> impl Iterator<Item = SlotId> + '_ {
        SlotId::all().filter(|slot| self.contains(*slot))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BitmapError> {
        let bytes: [u8; BYTES] = bytes
            .try_into()
            .map_err(|_| BitmapError::InvalidLength(bytes.len()))?;
        Ok(SlotBitmap(bytes))
    }

    pub fn to_base64url(&self) -> String {
        URL_SAFE_NO_PAD.encode(self.0)
    }

    pub fn from_base64url(encoded: &str) -> Result<Self, BitmapError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| BitmapError::InvalidBase64)?;
        Self::from_bytes(&bytes)
    }
}

impl FromIterator<SlotId> for SlotBitmap {
    fn from_iter<I: IntoIterator<Item = SlotId>>(slots: I) -> Self {
        let mut bitmap = SlotBitmap::empty();
        for slot in slots {
            bitmap.insert(slot);
        }
        bitmap
    }
}
