use super::FieldSlot;

#[test]
fn test_fieldslot_no_assign() {
    let mut option: Option<i32> = None;

    {
        let slot = FieldSlot::new(&mut option);
        // let it drop
        let _ = slot;
    }

    assert!(option.is_none());
}

#[test]
fn test_fieldslot_with_assign() {
    let mut option: Option<i32> = None;

    {
        let slot = FieldSlot::new(&mut option);
        slot.fill::<i32>(42);
    }

    assert_eq!(option, Some(42));
}

#[test]
#[should_panic(expected = "tried to assign")]
fn test_fieldslot_with_assign_mismatched_type() {
    let mut option: Option<String> = None;

    let slot = FieldSlot::new(&mut option);
    slot.fill::<i32>(42);
}
