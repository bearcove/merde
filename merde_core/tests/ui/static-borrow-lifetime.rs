use merde_core::FieldSlot;

fn main() {
    let mut option: Option<i32> = None;
    let slot = FieldSlot::new(&mut option);

    #[allow(clippy::needless_lifetimes)]
    fn take_static_fieldslot<'s>(_f: FieldSlot<'s, 'static>) {}

    take_static_fieldslot(slot);
}
