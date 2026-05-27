use ruhdl::viz;

fn main() {
    println!("\x1b[2J\x1b[H");

    viz::animate_adder4(5, 3, false, 1500);
    std::thread::sleep(std::time::Duration::from_millis(2000));

    viz::animate_adder4(11, 4, false, 1500);
    std::thread::sleep(std::time::Duration::from_millis(2000));

    viz::animate_adder4(15, 1, false, 1500);
    std::thread::sleep(std::time::Duration::from_millis(2000));

    viz::animate_adder4(7, 8, true, 1500);
    std::thread::sleep(std::time::Duration::from_millis(2000));

    viz::animate_adder4(0, 0, true, 1500);
    std::thread::sleep(std::time::Duration::from_millis(1000));

    println!("\x1b[2J\x1b[HAll examples done!");
}
