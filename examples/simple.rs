fn main() {
    let amount = 50;
    let start = std::time::Instant::now();

    for i in 0..amount {
        heimdall::log(format!("Log #{}", i));
    }

    let duration = start.elapsed();
    println!("Time taken to send 10,000 logs: {:?}", duration);
    println!("Average time per log: {:?}", duration / amount as u32);
}
