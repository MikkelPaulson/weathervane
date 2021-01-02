use std::collections::HashMap;

use piet::kurbo::{Point, Rect};
use piet::{RenderContext, Text, TextLayout, TextLayoutBuilder};
use piet_cairo::{CairoRenderContext, CairoText};
use resvg;
use usvg;

use crate::weather::{
    AtmosphereType, CloudsType, OpenWeatherResponse, RainType, SnowType, WeatherCondition,
    WeatherState,
};

pub fn render(
    weather_report: Option<OpenWeatherResponse>,
    radar_map: Option<Vec<u8>>,
    ctx: &mut CairoRenderContext,
) {
    if let Some(weather_report) = weather_report {
        draw_current_conditions(
            ctx,
            &weather_report.current,
            Rect::from_origin_size((0., 270.), (280., 120.)),
        );

        for (i, forecast) in weather_report
            .hourly
            .iter()
            .filter(|e| e.time > weather_report.current.time)
            .step_by(2)
            .take(6)
            .enumerate()
        {
            draw_forecast(
                ctx,
                forecast,
                Rect::from_origin_size((280. / 6. * i as f64, 400.), (280. / 6., 80.)),
            );
        }
    }

    if let Some(radar_map) = radar_map {
        draw_weather_radar(
            ctx,
            radar_map,
            Rect::from_origin_size(Point::ORIGIN, (280., 260.)),
        );
    }
}

fn draw_current_conditions(ctx: &mut CairoRenderContext, state: &WeatherState, position: Rect) {
    ctx.with_save(|ctx| {
        ctx.clip(position);

        let icon_size = position.height() - 20.;
        let text_area_width = position.width() - icon_size;

        if let Some(temp) = &state.temp {
            let text = CairoText::new()
                .new_text_layout(format!(" {}", temp))
                .default_attribute(piet::TextAttribute::FontSize(position.height() / 3. * 2.))
                .build()
                .unwrap();
            ctx.draw_text(
                &text,
                ((text_area_width - text.size().width) / 2., position.y0),
            );
        }

        if let Some(wind) = &state.wind {
            let wind_speed = CairoText::new()
                .new_text_layout(
                    if wind.gust.is_some() && wind.gust.unwrap() > wind.speed + 3. {
                        format!(
                            "{}-{} km/h",
                            wind.speed_km_h().round(),
                            wind.gust_km_h().unwrap().round()
                        )
                    } else {
                        format!("{} km/h", wind.speed_km_h().round())
                    },
                )
                .default_attribute(piet::TextAttribute::FontSize(position.height() / 6.))
                .build()
                .unwrap();
            let wind_direction = CairoText::new()
                .new_text_layout(wind.arrow())
                .default_attribute(piet::TextAttribute::FontSize(position.height() / 4.))
                .build()
                .unwrap();

            ctx.draw_text(
                &wind_direction,
                (
                    (text_area_width - wind_direction.size().width - wind_speed.size().width) / 2.,
                    position.y1
                        - wind_speed.size().height * 1.5
                        - (wind_direction.size().height - wind_speed.size().height) / 2.,
                ),
            );
            ctx.draw_text(
                &wind_speed,
                (
                    (text_area_width + wind_direction.size().width - wind_speed.size().width) / 2.,
                    position.y1 - wind_speed.size().height * 1.5,
                ),
            );
        }

        {
            let icon = ctx
                .make_image(
                    icon_size as usize,
                    icon_size as usize,
                    &resvg::render(
                        &get_weather_icon(state),
                        usvg::FitTo::Height(icon_size as u32),
                        None,
                    )
                    .unwrap()
                    .data()[..],
                    piet::ImageFormat::RgbaPremul,
                )
                .unwrap();
            ctx.draw_image(
                &icon,
                Rect::from_origin_size(
                    (
                        position.x1 - icon_size - 10.,
                        position.y0 + (position.height() - icon_size) / 2.,
                    ),
                    (icon_size, icon_size),
                ),
                piet::InterpolationMode::NearestNeighbor,
            );
        }

        Ok(())
    })
    .unwrap()
}

fn draw_forecast(ctx: &mut CairoRenderContext, state: &WeatherState, position: Rect) {
    ctx.with_save(|ctx| {
        ctx.clip(position);

        let icon_size = position.height() / 2. - 10.;

        if let Some(temp) = &state.temp {
            let text = CairoText::new()
                .new_text_layout(format!(" {}", temp))
                .default_attribute(piet::TextAttribute::FontSize(position.width() / 5. * 2.))
                .build()
                .unwrap();
            ctx.draw_text(
                &text,
                (
                    position.x0 + (position.width() - text.size().width) / 2.,
                    position.y0,
                ),
            );
        }

        {
            let icon = ctx
                .make_image(
                    icon_size as usize,
                    icon_size as usize,
                    &resvg::render(
                        &get_weather_icon(state),
                        usvg::FitTo::Height(icon_size as u32),
                        None,
                    )
                    .unwrap()
                    .data()[..],
                    piet::ImageFormat::RgbaPremul,
                )
                .unwrap();
            ctx.draw_image(
                &icon,
                Rect::from_origin_size(
                    (
                        position.x0 + (position.width() - icon_size) / 2.,
                        position.y0 + (position.height() - icon_size) / 2.,
                    ),
                    (icon_size, icon_size),
                ),
                piet::InterpolationMode::NearestNeighbor,
            );
        }

        {
            let text = CairoText::new()
                .new_text_layout(format!("{}h", state.time.hour()))
                .default_attribute(piet::TextAttribute::FontSize(position.width() / 3.))
                .build()
                .unwrap();
            ctx.draw_text(
                &text,
                (
                    position.x0 + (position.width() - text.size().width) / 2.,
                    position.y1 - text.size().height,
                ),
            );
        }

        Ok(())
    })
    .unwrap();
}

fn draw_weather_radar(ctx: &mut CairoRenderContext, radar_map: Vec<u8>, position: Rect) {
    draw_gif(
        ctx,
        &include_bytes!("../images/radar-rivers.gif")[..],
        position,
        |image_palette, _frame, bg_color| {
            let mut palette = HashMap::new();

            for (index, color) in image_palette.iter().step_by(3).enumerate() {
                palette.insert(index as u8, [0x80, 0x80, 0x80, *color]);
            }

            if let Some(index) = bg_color {
                palette.insert(index as u8, [0x00; 4]);
            }

            palette
        },
    );

    draw_gif(
        ctx,
        &radar_map[..],
        position,
        |_image_palette, frame, _bg_color| {
            let mut scale: Vec<&u8> = Vec::new();

            frame
                .buffer
                .iter()
                .skip(524)
                .step_by(frame.width as usize)
                .for_each(|pixel| {
                    if !scale.contains(&pixel) {
                        scale.push(pixel);
                    }
                });

            let mut palette = HashMap::new();
            for i in (0x55..0xFF).step_by(0x2a) {
                if let Some(&index) = scale.pop() {
                    palette.insert(index, [0x00, 0x00, 0x00, i]);
                }
            }
            while let Some(&index) = scale.pop() {
                palette.insert(index, [0x00, 0x00, 0x00, 0xFF]);
            }

            palette
        },
    );

    draw_gif(
        ctx,
        &include_bytes!("../images/radar-towns.gif")[..],
        position,
        |image_palette, _frame, bg_color| {
            let mut palette = HashMap::new();

            for (index, color) in image_palette.chunks_exact(3).enumerate() {
                palette.insert(index as u8, [color[0], color[1], color[2], 0xFF]);
            }

            if let Some(index) = bg_color {
                palette.insert(index as u8, [0x00; 4]);
            }

            palette
        },
    );
}

fn draw_gif<F: Fn(&[u8], &gif::Frame, Option<u8>) -> HashMap<u8, [u8; 4]>>(
    ctx: &mut CairoRenderContext,
    image: &[u8],
    position: Rect,
    palette_callback: F,
) {
    ctx.with_save(|ctx| {
        ctx.clip(position);

        let mut decoder = gif::Decoder::new(image).unwrap();
        let (width, height) = (decoder.width() as usize, decoder.height() as usize);
        let global_palette: Vec<u8> = decoder.global_palette().unwrap().iter().copied().collect();
        let bg_color = decoder.bg_color().map(|i| i as u8);
        let frame = decoder.read_next_frame().unwrap().unwrap();

        let palette = palette_callback(&global_palette[..], &frame, bg_color);

        let mut buffer: Vec<u8> = Vec::with_capacity(width * height * 4);

        for index in frame.buffer.iter() {
            buffer.extend_from_slice(&palette.get(index).unwrap_or(&[0x00; 4])[..]);
        }

        let ctx_image = ctx
            .make_image(
                width as usize,
                height as usize,
                &buffer,
                piet::ImageFormat::RgbaPremul,
            )
            .unwrap();

        ctx.draw_image_area(
            &ctx_image,
            Rect::from_center_size((height as f64 / 2., height as f64 / 2.), position.size()),
            position,
            piet::InterpolationMode::Bilinear,
        );

        Ok(())
    })
    .unwrap();
}

fn get_weather_icon(state: &WeatherState) -> usvg::Tree {
    let daytime = if let (Some(sunrise), Some(sunset)) = (state.sunrise, state.sunset) {
        state.time > sunrise && state.time < sunset
    } else {
        true
    };

    let partly_cloudy = state.clouds.map_or(false, |clouds| clouds <= 50);

    usvg::Tree::from_str(
        match &state.condition {
            Some(WeatherCondition::Thunderstorm(_)) if partly_cloudy => {
                if daytime {
                    include_str!("../images/weather/022-storm-1.svg")
                } else {
                    include_str!("../images/weather/042-storm-2.svg")
                }
            }
            Some(WeatherCondition::Thunderstorm(_)) => {
                include_str!("../images/weather/043-rain-1.svg")
            }
            Some(WeatherCondition::Drizzle(_)) => include_str!("../images/weather/046-drizzle.svg"),
            Some(WeatherCondition::Rain(subtype)) => match subtype {
                RainType::FreezingRain => include_str!("../images/weather/014-icicles.svg"),
                _ if partly_cloudy => {
                    if daytime {
                        include_str!("../images/weather/011-cloudy.svg")
                    } else {
                        include_str!("../images/weather/013-night-1.svg")
                    }
                }
                _ => {
                    include_str!("../images/weather/004-rainy.svg")
                }
            },
            Some(WeatherCondition::Snow(subtype)) => match subtype {
                SnowType::Sleet | SnowType::LightShowerSleet | SnowType::ShowerSleet => {
                    include_str!("../images/weather/031-hail.svg")
                }
                SnowType::LightRainAndSnow
                | SnowType::RainAndSnow
                | SnowType::LightShowerSnow
                | SnowType::ShowerSnow
                | SnowType::HeavyShowerSnow => include_str!("../images/weather/024-snowy.svg"),
                _ if partly_cloudy => {
                    if daytime {
                        include_str!("../images/weather/032-snowy-1.svg")
                    } else {
                        include_str!("../images/weather/002-night.svg")
                    }
                }
                _ => include_str!("../images/weather/041-snowy-2.svg"),
            },
            Some(WeatherCondition::Atmosphere(subtype)) => match subtype {
                AtmosphereType::Tornado => include_str!("../images/weather/016-tornado-1.svg"),
                AtmosphereType::Squalls => include_str!("../images/weather/015-windy-1.svg"),
                _ => include_str!("../images/weather/045-fog.svg"),
            },
            Some(WeatherCondition::Clear) if daytime => {
                include_str!("../images/weather/044-sun.svg")
            }
            Some(WeatherCondition::Clear) => include_str!("../images/weather/034-night-3.svg"),
            Some(WeatherCondition::Clouds(subtype)) => match subtype {
                CloudsType::FewClouds | CloudsType::ScatteredClouds if daytime => {
                    include_str!("../images/weather/033-cloudy-3.svg")
                }
                CloudsType::FewClouds => {
                    include_str!("../images/weather/023-night-2.svg")
                }
                CloudsType::ScatteredClouds => {
                    include_str!("../images/weather/028-cloudy-2.svg")
                }
                _ => include_str!("../images/weather/035-cloudy-4.svg"),
            },
            _ => include_str!("../images/weather/019-weathercock.svg"),
        },
        &usvg::Options::default(),
    )
    .unwrap()
}
