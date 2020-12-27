use std::thread;
use std::time::Duration;

use weathervane::display::Display;

fn main() {
    let mut display = Display::new();
    println!("display.init();");
    display.init();

    println!("display.clear();");
    display.clear();
    println!("draw_sample();");
    draw_sample(&mut display);
    println!("sleeping");
    thread::sleep(Duration::from_secs(10));

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

fn draw_sample(display: &mut Display) {
    let mut device = piet_common::Device::new().unwrap();
    let mut target = device
        .bitmap_target(Display::DISPLAY_WIDTH, Display::DISPLAY_HEIGHT, 1.0)
        .unwrap();

    piet::samples::get(0)
        .unwrap()
        .draw(&mut target.render_context())
        .unwrap();

    let (mut channel1, mut channel2) = (Vec::new(), Vec::new());

    for row in target
        .to_image_buf(piet::ImageFormat::Rgb)
        .unwrap()
        .pixel_colors()
    {
        let (mut channel1_value, mut channel2_value) = (0, 0);
        for (index, pixel) in row.enumerate() {
            if index % 8 == 0 {
                channel1_value = 0;
                channel2_value = 0;
            }

            let (r, g, b, a) = pixel.as_rgba();
            let color = ((r + g + b) * a) as u8;

            if color & 0x02 == 0x02 {
                channel1_value |= 1 << index;
            }

            if color & 0x01 == 0x01 {
                channel2_value |= 1 << index;
            }

            if index % 8 == 7 {
                channel1.push(channel1_value);
                channel2.push(channel2_value);
            }
        }
    }

    display.draw(&channel1, &channel2);
}
