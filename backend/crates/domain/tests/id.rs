use domain::{EventId, UserId};

#[test]
fn generated_ids_have_prefix_and_parse_back() {
    let event = EventId::generate();
    assert!(event.as_str().starts_with("e_"));
    assert_eq!(event.as_str().len(), 2 + 22);
    assert_eq!(EventId::parse(event.as_str()).unwrap(), event);

    let user = UserId::generate();
    assert!(user.as_str().starts_with("u_"));
    assert_eq!(UserId::parse(user.as_str()).unwrap(), user);
}

#[test]
fn generated_ids_are_unique() {
    assert_ne!(EventId::generate(), EventId::generate());
}

#[test]
fn rejects_wrong_prefix_length_or_alphabet() {
    let user = UserId::generate();
    assert!(EventId::parse(user.as_str()).is_err());
    assert!(EventId::parse("e_short").is_err());
    assert!(EventId::parse("e_AAAAAAAAAAAAAAAAAAAAA/").is_err());
    assert!(EventId::parse("").is_err());
}
