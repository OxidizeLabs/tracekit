use std::time::Instant;

fn main() {
    let start = Instant::now();

    let mut sum: u64 = 0;
    for i in 0..5_000_000u64 {
        sum = sum.wrapping_add(i);
    }

    let elapsed = start.elapsed();
    println!("bench placeholder: sum={sum} elapsed={elapsed:?}");
}
