use merde_core::FieldSlot;

fn main() {
    let mut option: Option<i32> = None;
    let slot = FieldSlot::new(&mut option);

    #[allow(clippy::needless_lifetimes)]
    fn take_static_fieldslot<'borrow>(_f: FieldSlot<'static, 'borrow>) {}

    take_static_fieldslot(slot);
}
