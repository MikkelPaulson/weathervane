use std::thread;
use std::time::Duration;

use resvg;
use usvg;

use weathervane::display::Display;

fn main() {
    let mut display = Display::new();
    //let mut display = Display::dummy();
    println!("display.init();");
    display.init().unwrap();
    display.clear().unwrap();

    for i in 0..=14 {
        println!("sample {}", i);
        draw_sample(&mut display, i);
        thread::sleep(Duration::from_secs(15));
    }

    /*
    println!("display.clear();");
    display.clear().unwrap();
    println!("draw_rust_logo();");
    draw_rust_logo(&mut display);
    println!("sleeping");
    thread::sleep(Duration::from_secs(10));
    */

    println!("display.sleep();");
    display.init().unwrap();
    display.clear().unwrap();
    display.sleep().unwrap();
}

fn draw_sample(display: &mut Display, sample_num: usize) {
    display.render(|ctx: &mut piet_cairo::CairoRenderContext| {
        piet::samples::get::<piet_cairo::CairoRenderContext>(sample_num)
            .unwrap()
            .draw(ctx)
            .unwrap();
    });
}

fn draw_rust_logo(display: &mut Display) {
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

    display.draw(&channel1, &channel2).unwrap();
}
