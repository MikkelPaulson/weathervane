use std::collections::HashMap;
//use std::thread;
//use std::time::Duration;

use piet::kurbo;
use piet::{RenderContext, Text, TextLayout, TextLayoutBuilder};
use resvg;
use usvg;

use weathervane::display::Display;

fn main() {
    let mut display = Display::new();
    //let mut display = Display::dummy();
    println!("display.init();");
    display.init().unwrap();
    //display.clear().unwrap();

    println!("Drawing mockup");
    draw_mockup(&mut display);
    //thread::sleep(Duration::from_secs(5));

    /*
    for i in 0..=14 {
        println!("sample {}", i);
        draw_sample(&mut display, i);
        thread::sleep(Duration::from_secs(5));
    }
    */

    /*
    println!("display.clear();");
    display.clear().unwrap();
    println!("draw_rust_logo();");
    draw_rust_logo(&mut display);
    println!("sleeping");
    thread::sleep(Duration::from_secs(10));
    */

    println!("display.sleep();");
    display.sleep().unwrap();
}

fn draw_mockup(display: &mut Display) {
    display.render(|ctx: &mut piet_cairo::CairoRenderContext| {
        let temp_current = piet_cairo::CairoText::new()
            .new_text_layout("-23°")
            .default_attribute(piet::TextAttribute::FontSize(60.))
            .build()
            .unwrap();
        ctx.draw_text(
            &temp_current,
            kurbo::Rect::from_center_size((80., 340.), temp_current.size()).origin(),
        );

        let temp_high_low = piet_cairo::CairoText::new()
            .new_text_layout("-19° / -25°")
            .default_attribute(piet::TextAttribute::FontSize(20.))
            .build()
            .unwrap();
        ctx.draw_text(
            &temp_high_low,
            kurbo::Rect::from_center_size((80., 390.), temp_high_low.size()).origin(),
        );

        let weather_icon = ctx
            .make_image(
                120,
                120,
                &resvg::render(
                    &usvg::Tree::from_str(
                        &include_str!("../images/weather/024-snowy.svg"),
                        &usvg::Options::default(),
                    )
                    .unwrap(),
                    usvg::FitTo::Width(120),
                    None,
                )
                .unwrap()
                .data()[..],
                piet::ImageFormat::RgbaPremul,
            )
            .unwrap();
        ctx.draw_image(
            &weather_icon,
            kurbo::Rect::from_origin_size((145., 295.), (120., 120.)),
            piet::InterpolationMode::NearestNeighbor,
        );

        let mut decoder =
            gif::Decoder::new(&include_bytes!("../images/radar-test.gif")[..]).unwrap();
        let (palette, frame, frame_width, frame_height) = {
            let frame = decoder.read_next_frame().unwrap().unwrap();

            let mut scale: Vec<u8> = Vec::new();

            frame
                .buffer
                .iter()
                .skip(524)
                .step_by(frame.width as usize)
                .for_each(|pixel| {
                    if !scale.contains(pixel) {
                        scale.push(*pixel);
                    }
                });

            let mut palette: HashMap<u8, u8> = HashMap::new();
            for i in (0x00..0xff).step_by(0x55).rev() {
                // 0xaa, 0x55, 0x00
                if let Some(index) = scale.pop() {
                    palette.insert(index, i + 0x2a);
                }
                if let Some(index) = scale.pop() {
                    palette.insert(index, i);
                }
            }
            while let Some(index) = scale.pop() {
                palette.insert(index, 0x00);
            }

            (palette, frame, frame.width as usize, frame.height as usize)
        };

        let radar_map = ctx
            .make_image(
                frame_width,
                frame_height,
                &frame
                    .buffer
                    .iter()
                    .flat_map(|color: &u8| {
                        vec![palette[color], palette[color], palette[color], 0xFF]
                    })
                    .collect::<Vec<u8>>()[..],
                piet::ImageFormat::RgbaPremul,
            )
            .unwrap();
        ctx.draw_image_area(
            &radar_map,
            kurbo::Rect::from_center_size(
                (frame_height as f64 / 2., frame_height as f64 / 2.),
                (280., 280.),
            ),
            kurbo::Rect::from_origin_size(kurbo::Point::ORIGIN, (280., 280.)),
            piet::InterpolationMode::Bilinear,
        );
    });
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
    let rust_logo = usvg::Tree::from_str(
        &include_str!("../images/rust.svg"),
        &usvg::Options::default(),
    )
    .unwrap();

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
