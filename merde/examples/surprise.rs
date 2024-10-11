use std::thread::Thread;

use merde::Value;

struct Person {
    first_name: String,
    last_name: String,
}

merde::derive! {
    impl (Deserialize) for struct Person { first_name, last_name }
}

fn main() {
    let jh = std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let cool_factor = 100_000;

            let first_half = "[".repeat(cool_factor);
            let second_half = "]".repeat(cool_factor);
            let input = format!("{first_half}{second_half}");

            let value: Value<'_> = merde::json::from_str(&input[..]).unwrap();

            let mut current_value = &value;
            let mut count = 0;
            loop {
                if let Value::Array(arr) = &current_value {
                    if arr.len() == 0 {
                        break;
                    } else {
                        current_value = &arr[0];
                        count += 1;
                    }
                }
            }
            println!("final count {count}");

            // at this point `value` is a bomb — if we try to drop it, it _will_
            // overflow the stack. the only way out of this is to mem::forget it
            std::mem::forget(value);
        })
        .unwrap();

    jh.join().unwrap();
}
