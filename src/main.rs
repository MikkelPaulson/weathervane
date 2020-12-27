use std::thread;
use std::time::Duration;

use resvg;
use usvg;

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

    println!("display.clear();");
    display.clear();
    println!("draw_sample();");
    draw_sample(&mut display);
    println!("sleeping");
    thread::sleep(Duration::from_secs(10));

    println!("display.sleep();");
    display.init();
    display.clear();
    display.sleep();
}

fn draw_sample(display: &mut Display) {
    let rust_logo =
        usvg::Tree::from_str(&include_str!("rust.svg"), &usvg::Options::default()).unwrap();

    let image_size = rust_logo.svg_node().size;
    let display_size = usvg::ScreenSize::new(
        Display::DISPLAY_WIDTH as u32,
        Display::DISPLAY_HEIGHT as u32,
    )
    .unwrap();

    let fit = if image_size.width()
        > display_size.width() as f64 / display_size.height() as f64 * image_size.height()
    {
        usvg::FitTo::Width(display_size.width())
    } else {
        usvg::FitTo::Height(display_size.height())
    };

    let image = resvg::render(&rust_logo, fit, None).unwrap();
    let image_data = image.data();

    let (channel1, channel2): (Vec<u8>, Vec<u8>) = image_data[..]
        .chunks(8 * 4) // 8 pixels @ RGBA
        .map(|chunk: &[u8]| {
            chunk
                .chunks_exact(4) // chunk by pixel (RGBA)
                .enumerate()
                .map(|(index, pixel)| {
                    // For now, we just render the alpha channel.
                    let color = (pixel[3] as f64 / 255. * 3.).round() as u8;
                    let result = (
                        if color & 0x01 == 0x01 {
                            0
                        } else {
                            0x80 >> index
                        },
                        if color & 0x02 == 0x02 {
                            0
                        } else {
                            0x80 >> index
                        },
                    );
                    result
                })
                .fold((0, 0), |a, b| (a.0 | b.0, a.1 | b.1))
        })
        .unzip();

    display.draw(&channel1, &channel2);
}
/*
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
        .to_image_buf(piet::ImageFormat::RgbaPremul)
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
                channel1_value |= 0x80 >> (index % 8);
            }

            if color & 0x01 == 0x01 {
                channel2_value |= 0x80 >> (index % 8);
            }

            if index % 8 == 7 {
                channel1.push(channel1_value);
                channel2.push(channel2_value);
            }
        }
    }

    display.draw(&channel1, &channel2);
}
*/
