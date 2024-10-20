use merde_core::FieldSlot;

fn main() {
    let mut option: Option<i32> = None;
    let slot = FieldSlot::new(&mut option);

    fn prove_invariance<'long, 'short: 'long>(
        long: FieldSlot<'long, 'long>,
    ) -> FieldSlot<'short, 'short> {
        long // Error: mismatched types
    }

    assert!(option.is_none());
}
