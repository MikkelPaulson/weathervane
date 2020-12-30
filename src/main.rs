use std::collections::HashMap;
use std::iter;
//use std::thread;
//use std::time::Duration;

use piet::kurbo;
use piet::{RenderContext, Text, TextLayout, TextLayoutBuilder};
use resvg;
use usvg;

use weathervane::display::waveshare::EPaper3_7in;
use weathervane::display::Display;

fn main() {
    let mut display = EPaper3_7in::new();
    println!("display.on();");
    display.on().unwrap();

    println!("Drawing mockup");
    draw_mockup(&mut display);

    println!("display.sleep();");
    display.sleep().unwrap();
}

fn draw_mockup(display: &mut impl Display) {
    display
        .draw_context(|ctx: &mut piet_cairo::CairoRenderContext| {
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
                gif::Decoder::new(&include_bytes!("../images/radar-rivers.gif")[..]).unwrap();
            let (palette, frame, frame_width, frame_height) = {
                let palette: Vec<u8> = decoder.palette().unwrap().iter().copied().collect();
                let frame = decoder.read_next_frame().unwrap().unwrap();

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
                            iter::repeat(0x55).take(3).chain(iter::once(
                                0xFF - palette.get((color * 3) as usize).unwrap_or(&0x00),
                            ))
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
                for i in (0x55..0xFF).step_by(0x2a) {
                    if let Some(index) = scale.pop() {
                        palette.insert(index, i);
                    }
                }
                while let Some(index) = scale.pop() {
                    palette.insert(index, 0xFF);
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
                            iter::repeat(0x00)
                                .take(3)
                                .chain(iter::once(palette.get(color).unwrap_or(&0x00)).copied())
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
        })
        .unwrap();
}
