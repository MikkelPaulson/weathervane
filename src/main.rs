use std::thread;
use std::time::Duration;

use weathervane::display::Display;

fn main() {
    let mut display = Display::new();
    println!("display.init();");
    display.init();
    println!("display.clear();");
    display.clear();

    println!("display.checkerboard();");
    display.checkerboard();

    println!("sleeping");
    thread::sleep(Duration::from_secs(10));

    println!("display.sleep();");
    display.init();
    display.clear();
    display.sleep();
}
