use std::collections::HashMap;
use std::iter;

use piet::kurbo;
use piet::{RenderContext, Text, TextLayout, TextLayoutBuilder};
use resvg;
use usvg;

use crate::weather::{
    AtmosphereType, CloudsType, OpenWeatherResponse, RainType, SnowType, WeatherCondition,
    WeatherState,
};

pub fn render(
    weather_report: Option<OpenWeatherResponse>,
    radar_map: Option<bytes::Bytes>,
    ctx: &mut piet_cairo::CairoRenderContext,
) {
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
                &get_weather_icon(weather_report.unwrap().current),
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

    let mut decoder = gif::Decoder::new(&include_bytes!("../images/radar-rivers.gif")[..]).unwrap();
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

    let mut decoder = gif::Decoder::new(&include_bytes!("../images/radar-test.gif")[..]).unwrap();
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
}

fn get_weather_icon(state: WeatherState) -> usvg::Tree {
    let daytime = state.time > state.sunrise && state.time < state.sunset;

    usvg::Tree::from_str(
        &match state.condition {
            WeatherCondition::Thunderstorm(_) => include_str!("../images/weather/010-rain.svg"),
            WeatherCondition::Drizzle(_) => include_str!("../images/weather/046-drizzle.svg"),
            WeatherCondition::Rain(subtype) => match subtype {
                RainType::FreezingRain => include_str!("../images/weather/014-icicles.svg"),
                _ => include_str!("../images/weather/004-rainy.svg"),
            },
            WeatherCondition::Snow(subtype) => match subtype {
                SnowType::Sleet | SnowType::LightShowerSleet | SnowType::ShowerSleet => {
                    include_str!("../images/weather/031-hail.svg")
                }
                SnowType::LightRainAndSnow
                | SnowType::RainAndSnow
                | SnowType::LightShowerSnow
                | SnowType::ShowerSnow
                | SnowType::HeavyShowerSnow => include_str!("../images/weather/024-snowy.svg"),
                _ => include_str!("../images/weather/032-snowy-1.svg"),
            },
            WeatherCondition::Atmosphere(subtype) => match subtype {
                AtmosphereType::Tornado => include_str!("../images/weather/006-tornado.svg"),
                AtmosphereType::Squalls => include_str!("../images/weather/003-windy.svg"),
                _ => include_str!("../images/weather/045-fog.svg"),
            },
            WeatherCondition::Clear if daytime => include_str!("../images/weather/044-sun.svg"),
            WeatherCondition::Clear => include_str!("../images/weather/002-night.svg"),
            WeatherCondition::Clouds(subtype) => match subtype {
                CloudsType::FewClouds | CloudsType::ScatteredClouds if daytime => {
                    include_str!("../images/weather/021-cloudy-1.svg")
                }
                CloudsType::FewClouds | CloudsType::ScatteredClouds => {
                    include_str!("../images/weather/028-cloudy-2.svg")
                }
                _ => include_str!("../images/weather/011-cloudy.svg"),
            },
            WeatherCondition::Unknown(_) => include_str!("../images/weather/019-weathercock.svg"),
        },
        &usvg::Options::default(),
    )
    .unwrap()
}
