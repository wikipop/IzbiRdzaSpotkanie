use domain::{BitmapError, SlotBitmap, SlotId};

fn slot(s: &str) -> SlotId {
    s.parse().unwrap()
}

#[test]
fn insert_and_contains() {
    let mut bitmap = SlotBitmap::empty();
    assert!(bitmap.is_empty());
    bitmap.insert(slot("pn-00"));
    bitmap.insert(slot("sr-09"));
    bitmap.insert(slot("nd-23"));
    assert!(bitmap.contains(slot("pn-00")));
    assert!(bitmap.contains(slot("sr-09")));
    assert!(bitmap.contains(slot("nd-23")));
    assert!(!bitmap.contains(slot("sr-10")));
    assert_eq!(bitmap.len(), 3);
}

#[test]
fn bit_layout_is_lsb_first() {
    let bitmap: SlotBitmap = [slot("pn-00"), slot("pn-09"), slot("nd-23")]
        .into_iter()
        .collect();
    let bytes = bitmap.as_bytes();
    assert_eq!(bytes.len(), 21);
    assert_eq!(bytes[0], 0b0000_0001);
    assert_eq!(bytes[1], 0b0000_0010);
    assert_eq!(bytes[20], 0b1000_0000);
}

#[test]
fn iter_returns_slots_in_index_order() {
    let bitmap: SlotBitmap = [slot("wt-01"), slot("pn-05")].into_iter().collect();
    let slots: Vec<String> = bitmap.iter().map(|s| s.to_string()).collect();
    assert_eq!(slots, ["pn-05", "wt-01"]);
}

#[test]
fn base64url_roundtrip() {
    let bitmap: SlotBitmap = SlotId::all().filter(|s| s.hour() % 3 == 0).collect();
    let encoded = bitmap.to_base64url();
    assert_eq!(encoded.len(), 28);
    assert!(!encoded.contains(['+', '/', '=']));
    assert_eq!(SlotBitmap::from_base64url(&encoded).unwrap(), bitmap);
}

#[test]
fn full_bitmap_roundtrip() {
    let bitmap: SlotBitmap = SlotId::all().collect();
    assert_eq!(bitmap.len(), 168);
    assert_eq!(
        SlotBitmap::from_base64url(&bitmap.to_base64url()).unwrap(),
        bitmap
    );
}

#[test]
fn rejects_invalid_encodings() {
    assert_eq!(
        SlotBitmap::from_base64url("not base64!"),
        Err(BitmapError::InvalidBase64)
    );
    assert_eq!(
        SlotBitmap::from_base64url("AAAA"),
        Err(BitmapError::InvalidLength(3))
    );
    assert_eq!(
        SlotBitmap::from_bytes(&[0; 22]),
        Err(BitmapError::InvalidLength(22))
    );
}
