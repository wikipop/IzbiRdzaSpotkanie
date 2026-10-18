use domain::{Day, SlotError, SlotId};

#[test]
fn parses_and_formats_slot() {
    let slot: SlotId = "sr-09".parse().unwrap();
    assert_eq!(slot.day(), Day::Sr);
    assert_eq!(slot.hour(), 9);
    assert_eq!(slot.to_string(), "sr-09");
}

#[test]
fn index_follows_day_times_24_plus_hour() {
    assert_eq!("pn-00".parse::<SlotId>().unwrap().index(), 0);
    assert_eq!("pn-23".parse::<SlotId>().unwrap().index(), 23);
    assert_eq!("wt-00".parse::<SlotId>().unwrap().index(), 24);
    assert_eq!("nd-23".parse::<SlotId>().unwrap().index(), 167);
}

#[test]
fn index_roundtrip_covers_all_168_slots() {
    let all: Vec<SlotId> = SlotId::all().collect();
    assert_eq!(all.len(), 168);
    for (index, slot) in all.iter().enumerate() {
        assert_eq!(slot.index(), index);
        assert_eq!(slot.to_string().parse::<SlotId>().unwrap(), *slot);
    }
}

#[test]
fn rejects_invalid_slots() {
    for input in [
        "", "pn", "pn-", "pn-9", "pn-009", "pn-24", "xx-01", "PN-01", "pn_01", "pn-+1", "pn- 1",
    ] {
        assert!(
            input.parse::<SlotId>().is_err(),
            "`{input}` should be rejected"
        );
    }
}

#[test]
fn rejects_index_out_of_range() {
    assert_eq!(
        SlotId::from_index(168),
        Err(SlotError::IndexOutOfRange(168))
    );
}
