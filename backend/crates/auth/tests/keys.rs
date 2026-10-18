use auth::{KeyError, generate_keypair, parse_public_keys};

#[test]
fn generates_pem_keys() {
    let pair = generate_keypair();
    assert!(pair.private_pem.starts_with("-----BEGIN PRIVATE KEY-----"));
    assert!(pair.public_pem.starts_with("-----BEGIN PUBLIC KEY-----"));
}

#[test]
fn parses_multiple_keys_with_escaped_newlines() {
    let spec = "k1=-----BEGIN PUBLIC KEY-----\\nAAA=\\n-----END PUBLIC KEY-----, k2=B";
    let keys = parse_public_keys(spec).unwrap();
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0].0, "k1");
    assert_eq!(
        keys[0].1,
        "-----BEGIN PUBLIC KEY-----\nAAA=\n-----END PUBLIC KEY-----"
    );
    assert_eq!(keys[1], ("k2".to_owned(), "B".to_owned()));
}

#[test]
fn rejects_malformed_specs() {
    assert_eq!(parse_public_keys(""), Err(KeyError::Empty));
    assert!(matches!(
        parse_public_keys("no-kid"),
        Err(KeyError::MissingKid(_))
    ));
    assert!(matches!(
        parse_public_keys("=pem"),
        Err(KeyError::MissingKid(_))
    ));
    assert_eq!(
        parse_public_keys("a=x,a=y"),
        Err(KeyError::DuplicateKid("a".to_owned()))
    );
}
