use domain::{Nick, NickError};

#[test]
fn trims_whitespace() {
    assert_eq!(Nick::parse("  Wiktor \t").unwrap().as_str(), "Wiktor");
}

#[test]
fn rejects_empty_and_blank() {
    assert_eq!(Nick::parse(""), Err(NickError::Empty));
    assert_eq!(Nick::parse("   "), Err(NickError::Empty));
}

#[test]
fn counts_characters_not_bytes() {
    let polish = "ż".repeat(32);
    assert!(Nick::parse(&polish).is_ok());
    assert_eq!(Nick::parse(&"ż".repeat(33)), Err(NickError::TooLong));
    assert!(Nick::parse(&"a".repeat(32)).is_ok());
    assert_eq!(Nick::parse(&"a".repeat(33)), Err(NickError::TooLong));
}

#[test]
fn rejects_control_characters() {
    assert_eq!(Nick::parse("a\u{0}b"), Err(NickError::ControlCharacter));
    assert_eq!(Nick::parse("a\nb"), Err(NickError::ControlCharacter));
}

#[test]
fn key_is_case_insensitive_including_unicode() {
    assert_eq!(
        Nick::parse("Łukasz").unwrap().key(),
        Nick::parse("łUKASZ").unwrap().key()
    );
    assert_ne!(
        Nick::parse("Ola").unwrap().key(),
        Nick::parse("Ala").unwrap().key()
    );
}
