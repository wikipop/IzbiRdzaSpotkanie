use domain::State;

#[test]
fn roundtrip() {
    for state in [State::Yes, State::Maybe] {
        assert_eq!(state.as_str().parse::<State>().unwrap(), state);
    }
}

#[test]
fn rejects_unknown() {
    assert!("no".parse::<State>().is_err());
    assert!("YES".parse::<State>().is_err());
}
