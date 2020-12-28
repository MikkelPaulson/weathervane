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
            kurbo::Rect::from_center_size((80., 60.), temp_current.size()).origin(),
        );

        let temp_high_low = piet_cairo::CairoText::new()
            .new_text_layout("-19° / -25°")
            .default_attribute(piet::TextAttribute::FontSize(20.))
            .build()
            .unwrap();
        ctx.draw_text(
            &temp_high_low,
            kurbo::Rect::from_center_size((80., 110.), temp_high_low.size()).origin(),
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
            kurbo::Rect::from_origin_size((145., 15.), (120., 120.)),
            piet::InterpolationMode::NearestNeighbor,
        );

        let mut decoder =
            gif::Decoder::new(&include_bytes!("../images/radar-test.gif")[..]).unwrap();
        let palette = decoder.palette().unwrap().to_vec();

        let frame = decoder.read_next_frame().unwrap().unwrap();
        let radar_map = ctx
            .make_image(
                frame.width as usize,
                frame.height as usize,
                &frame
                    .buffer
                    .iter()
                    .flat_map(|color: &u8| {
                        vec![
                            palette[*color as usize * 3],
                            palette[*color as usize * 3 + 1],
                            palette[*color as usize * 3 + 2],
                            0xFF,
                        ]
                    })
                    .collect::<Vec<u8>>()[..],
                piet::ImageFormat::RgbaPremul,
            )
            .unwrap();
        ctx.draw_image_area(
            &radar_map,
            kurbo::Rect::from_origin_size(
                kurbo::Point::ZERO,
                (frame.height as f64, frame.height as f64),
            ),
            kurbo::Rect::from_origin_size((10., 210.), (260., 260.)),
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
