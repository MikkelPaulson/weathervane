use std::collections::HashMap;

use piet::kurbo::{Affine, Circle, Line, Point, Rect};
use piet::{RenderContext, Text, TextLayout, TextLayoutBuilder};
use piet_cairo::{CairoRenderContext, CairoText};
use resvg;
use usvg;

use crate::weather::{
    AtmosphereType, OpenWeatherResponse, RainType, SnowType, ThunderstormType, WeatherCondition,
    WeatherState,
};

pub fn render(
    weather_report: Option<OpenWeatherResponse>,
    radar_map: Option<Vec<u8>>,
    ctx: &mut CairoRenderContext,
) {
    // Render the image upside down (since the device is mounted upside down).
    ctx.transform(Affine::translate((280., 480.)));
    ctx.transform(Affine::rotate(std::f64::consts::PI));

    // Flip the layout daily to mitigate burn-in. (Is burn-in a thing with e-paper?)
    let radar_on_top = time::OffsetDateTime::try_now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
        .day()
        % 2
        == 0;

    if let Some(weather_report) = weather_report {
        draw_current_conditions(
            ctx,
            &weather_report.current,
            Rect::from_origin_size((0., if radar_on_top { 265. } else { 95. }), (280., 120.)),
        );

        for (i, forecast) in weather_report
            .hourly
            .iter()
            .filter(|e| e.time > weather_report.current.time)
            .step_by(2)
            .take(5)
            .enumerate()
        {
            draw_forecast(
                ctx,
                forecast,
                Rect::from_origin_size(
                    (15. + 50. * i as f64, if radar_on_top { 390. } else { 10. }),
                    (50., 80.),
                ),
            );
        }
    }

    if let Some(radar_map) = radar_map {
        draw_weather_radar(
            ctx,
            radar_map,
            Rect::from_origin_size(
                if radar_on_top {
                    Point::ORIGIN
                } else {
                    (0., 220.).into()
                },
                (280., 260.),
            ),
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
                .new_text_layout(format!(
                    "{}{}",
                    if temp.celsius() > -10. { " " } else { "" },
                    temp,
                ))
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
    ctx.with_save(|ctx| {
        ctx.clip(position);

        ctx.stroke(
            Line::new(
                (position.width() / 2. - 5., position.height() / 2.),
                (position.width() / 2. + 5., position.height() / 2.),
            ),
            &piet::Color::rgb8(0xAA, 0xAA, 0xAA),
            2.,
        );

        ctx.stroke(
            Line::new(
                (position.width() / 2., position.height() / 2. - 5.),
                (position.width() / 2., position.height() / 2. + 5.),
            ),
            &piet::Color::rgb8(0xAA, 0xAA, 0xAA),
            1.5,
        );

        for i in (20..position.width() as usize / 2)
            .step_by(20)
            .map(|i| i as f64)
        {
            ctx.stroke(
                Circle::new((position.width() / 2., position.height() / 2.), i * 2.),
                &piet::Color::rgb8(0xAA, 0xAA, 0xAA),
                1.5,
            );
        }

        Ok(())
    })
    .unwrap();

    draw_gif(
        ctx,
        &include_bytes!("../images/radar-rivers.gif")[..],
        position,
        |image_palette, _frame, bg_color| {
            let mut palette = HashMap::new();

            for (index, color) in image_palette.iter().step_by(3).enumerate() {
                palette.insert(index as u8, [0x80, 0x80, 0x80, 0xFF - *color]);
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

                // Ignore the first few colours, including the black also used for
                // the US/Canada border.
                if scale.len() < 4 {
                    break;
                }
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
            Some(WeatherCondition::Thunderstorm(subtype)) => match subtype {
                ThunderstormType::ThunderstormWithLightRain
                | ThunderstormType::ThunderstormWithRain
                | ThunderstormType::ThunderstormWithHeavyRain => {
                    include_str!("../images/cute-weather/006-thunder.svg")
                }
                _ => include_str!("../images/cute-weather/021-thunderstorm.svg"),
            },
            Some(WeatherCondition::Drizzle(_)) if partly_cloudy => {
                include_str!("../images/cute-weather/024-sunny.svg")
            }
            Some(WeatherCondition::Drizzle(_)) => {
                include_str!("../images/cute-weather/004-rain.svg")
            }
            Some(WeatherCondition::Rain(subtype)) => match subtype {
                RainType::FreezingRain => include_str!("../images/cute-weather/027-sleet.svg"),
                _ => include_str!("../images/cute-weather/026-umbrella.svg"),
            },
            Some(WeatherCondition::Snow(subtype)) => match subtype {
                SnowType::Sleet | SnowType::LightShowerSleet | SnowType::ShowerSleet => {
                    include_str!("../images/cute-weather/014-hail.svg")
                }
                SnowType::LightRainAndSnow
                | SnowType::RainAndSnow
                | SnowType::LightShowerSnow
                | SnowType::ShowerSnow
                | SnowType::HeavyShowerSnow => include_str!("../images/cute-weather/027-sleet.svg"),
                SnowType::LightSnow => include_str!("../images/cute-weather/007-snow.svg"),
                _ => include_str!("../images/cute-weather/018-snowflake.svg"),
            },
            Some(WeatherCondition::Atmosphere(subtype)) => match subtype {
                AtmosphereType::Tornado => include_str!("../images/cute-weather/022-tornado.svg"),
                AtmosphereType::Squalls => include_str!("../images/cute-weather/012-windy.svg"),
                _ if daytime => include_str!("../images/cute-weather/019-fog.svg"),
                _ => include_str!("../images/cute-weather/028-fog.svg"),
            },
            Some(WeatherCondition::Clear) if daytime => {
                include_str!("../images/cute-weather/001-sunny.svg")
            }
            Some(WeatherCondition::Clear) => {
                include_str!("../images/cute-weather/023-crescent moon.svg")
            }
            Some(WeatherCondition::Clouds(_)) if partly_cloudy && daytime => {
                include_str!("../images/cute-weather/011-sunny.svg")
            }
            Some(WeatherCondition::Clouds(_)) if partly_cloudy => {
                include_str!("../images/cute-weather/025-crescent moon.svg")
            }
            Some(WeatherCondition::Clouds(_)) => {
                include_str!("../images/cute-weather/020-cloudy.svg")
            }
            _ => include_str!("../images/cute-weather/009-thermometer.svg"),
        },
        &usvg::Options::default(),
    )
    .unwrap()
}
