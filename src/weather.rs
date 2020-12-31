use std::convert::TryFrom;

pub fn query() -> Result<(Option<OpenWeatherResponse>, Option<RadarMap>), &'static str> {
    Ok((
        Some(OpenWeatherResponse {
            current: WeatherState {
                time: time::OffsetDateTime::now_utc().to_offset(time::UtcOffset::seconds(-18000)),
                sunrise: time::OffsetDateTime::now_utc()
                    .to_offset(time::UtcOffset::seconds(-18000))
                    - time::Duration::hour(),
                sunset: time::OffsetDateTime::now_utc().to_offset(time::UtcOffset::seconds(-18000))
                    + time::Duration::hour(),
                temp: 253.15.into(),
                wind: Wind {
                    speed: 3.6,
                    direction: 180,
                },
                condition: 616.into(),
            },
            hourly: vec![
                WeatherState {
                    time: time::OffsetDateTime::now_utc()
                        .to_offset(time::UtcOffset::seconds(-18000))
                        + time::Duration::hours(1),
                    sunrise: time::OffsetDateTime::now_utc()
                        .to_offset(time::UtcOffset::seconds(-18000))
                        - time::Duration::hour(),
                    sunset: time::OffsetDateTime::now_utc()
                        .to_offset(time::UtcOffset::seconds(-18000))
                        + time::Duration::hour(),
                    temp: 253.15.into(),
                    wind: Wind {
                        speed: 3.6,
                        direction: 180,
                    },
                    condition: 616.into(),
                },
                WeatherState {
                    time: time::OffsetDateTime::now_utc()
                        .to_offset(time::UtcOffset::seconds(-18000))
                        + time::Duration::hours(2),
                    sunrise: time::OffsetDateTime::now_utc()
                        .to_offset(time::UtcOffset::seconds(-18000))
                        - time::Duration::hour(),
                    sunset: time::OffsetDateTime::now_utc()
                        .to_offset(time::UtcOffset::seconds(-18000))
                        + time::Duration::hour(),
                    temp: 253.15.into(),
                    wind: Wind {
                        speed: 3.6,
                        direction: 180,
                    },
                    condition: 616.into(),
                },
                WeatherState {
                    time: time::OffsetDateTime::now_utc()
                        .to_offset(time::UtcOffset::seconds(-18000))
                        + time::Duration::hours(3),
                    sunrise: time::OffsetDateTime::now_utc()
                        .to_offset(time::UtcOffset::seconds(-18000))
                        - time::Duration::hour(),
                    sunset: time::OffsetDateTime::now_utc()
                        .to_offset(time::UtcOffset::seconds(-18000))
                        + time::Duration::hour(),
                    temp: 253.15.into(),
                    wind: Wind {
                        speed: 3.6,
                        direction: 180,
                    },
                    condition: 616.into(),
                },
                WeatherState {
                    time: time::OffsetDateTime::now_utc()
                        .to_offset(time::UtcOffset::seconds(-18000))
                        + time::Duration::hours(4),
                    sunrise: time::OffsetDateTime::now_utc()
                        .to_offset(time::UtcOffset::seconds(-18000))
                        - time::Duration::hour(),
                    sunset: time::OffsetDateTime::now_utc()
                        .to_offset(time::UtcOffset::seconds(-18000))
                        + time::Duration::hour(),
                    temp: 253.15.into(),
                    wind: Wind {
                        speed: 3.6,
                        direction: 180,
                    },
                    condition: 616.into(),
                },
            ],
        }),
        Some(RadarMap::from_static(include_bytes!(
            "../images/radar-test.gif"
        ))),
    ))
}

type RadarMap = bytes::Bytes;

pub struct OpenWeatherResponse {
    pub current: WeatherState,
    pub hourly: Vec<WeatherState>,
}

impl TryFrom<json::JsonValue> for OpenWeatherResponse {
    type Error = &'static str;

    fn try_from(mut json: json::JsonValue) -> Result<Self, Self::Error> {
        Ok(Self {
            current: WeatherState::try_from(json.remove("current"))?,
            hourly: json
                .remove("hourly")
                .members_mut()
                .map(|e| WeatherState::try_from(e.take()))
                .collect::<Result<_, _>>()?,
        })
    }
}

/// ```json
/// {
///     "dt": 1595243443,
///     "sunrise": 1608124431,
///     "sunset": 1608160224,
///     "temp": 274.75,
///     "feels_like": 270.4,
///     "pressure": 1017,
///     "humidity": 96,
///     "dew_point": 274.18,
///     "uvi": 0,
///     "clouds": 90,
///     "visibility": 6437,
///     "wind_speed": 3.6,
///     "wind_deg": 320,
///     "weather": [{
///         "id": 701,
///         "main": "Mist",
///         "description": "mist",
///         "icon": "50n"
///     }]
/// }
/// ```
pub struct WeatherState {
    pub time: time::OffsetDateTime,
    pub sunrise: time::OffsetDateTime,
    pub sunset: time::OffsetDateTime,
    pub temp: Temperature,
    pub wind: Wind,
    pub condition: WeatherCondition,
}

impl TryFrom<json::JsonValue> for WeatherState {
    type Error = &'static str;

    fn try_from(mut json: json::JsonValue) -> Result<Self, Self::Error> {
        Ok(Self {
            time: time::OffsetDateTime::from_unix_timestamp(
                json.remove("dt")
                    .as_i64()
                    .ok_or("Missing or invalid \"dt\" value.")?,
            ),
            sunrise: time::OffsetDateTime::from_unix_timestamp(
                json.remove("sunrise")
                    .as_i64()
                    .ok_or("Missing or invalid \"sunrise\" value.")?,
            ),
            sunset: time::OffsetDateTime::from_unix_timestamp(
                json.remove("sunset")
                    .as_i64()
                    .ok_or("Missing or invalid \"sunset\" value.")?,
            ),
            temp: json
                .remove("temp")
                .as_f32()
                .ok_or("Missing or invalid \"temp\" value.")?
                .into(),
            wind: Wind {
                speed: json
                    .remove("wind_speed")
                    .as_f32()
                    .ok_or("Missing or invalid \"wind_speed\" value.")?,
                direction: json
                    .remove("wind_deg")
                    .as_u16()
                    .ok_or("Missing or invalid \"wind_deg\" value.")?,
            },
            condition: json
                .remove("weather")
                .members_mut()
                .next()
                .ok_or("Empty \"weather\" value.")?
                .remove("id")
                .as_u16()
                .ok_or("Empty or invalid \"weather.id\" value.")?
                .into(),
        })
    }
}

pub struct Temperature(f32);

impl Temperature {
    pub const fn kelvin(kelvin: f32) -> Self {
        Self(kelvin)
    }

    pub fn celsius(&self) -> f32 {
        self.0 - 273.15
    }
}

impl From<f32> for Temperature {
    fn from(input: f32) -> Self {
        Self::kelvin(input)
    }
}

pub struct Wind {
    speed: f32,
    direction: u16,
}

impl Wind {}

pub enum WeatherCondition {
    Thunderstorm(ThunderstormType),
    Drizzle(DrizzleType),
    Rain(RainType),
    Snow(SnowType),
    Atmosphere(AtmosphereType),
    Clear,
    Clouds(CloudsType),
    Unknown(u16),
}

impl From<u16> for WeatherCondition {
    fn from(data: u16) -> Self {
        match data {
            200..=299 => Self::Thunderstorm(data.into()),
            300..=399 => Self::Drizzle(data.into()),
            500..=599 => Self::Rain(data.into()),
            600..=699 => Self::Snow(data.into()),
            700..=799 => Self::Atmosphere(data.into()),
            800 => Self::Clear,
            801..=899 => Self::Clouds(data.into()),
            _ => Self::Unknown(data),
        }
    }
}

pub enum ThunderstormType {
    ThunderstormWithLightRain,
    ThunderstormWithRain,
    ThunderstormWithHeavyRain,
    LightThunderstorm,
    Thunderstorm,
    HeavyThunderstorm,
    RaggedThunderstorm,
    ThunderstormWithLightDrizzle,
    ThunderstormWithDrizzle,
    ThunderstormWithHeavyDrizzle,
    Unknown(u16),
}

impl From<u16> for ThunderstormType {
    fn from(data: u16) -> Self {
        match data {
            200 => Self::ThunderstormWithLightRain,
            201 => Self::ThunderstormWithRain,
            202 => Self::ThunderstormWithHeavyRain,
            210 => Self::LightThunderstorm,
            211 => Self::Thunderstorm,
            212 => Self::HeavyThunderstorm,
            221 => Self::RaggedThunderstorm,
            230 => Self::ThunderstormWithLightDrizzle,
            231 => Self::ThunderstormWithDrizzle,
            232 => Self::ThunderstormWithHeavyDrizzle,
            _ => Self::Unknown(data),
        }
    }
}

pub enum DrizzleType {
    LightIntensityDrizzle,
    Drizzle,
    HeavyIntensityDrizzle,
    LightIntensityDrizzleRain,
    DrizzleRain,
    HeavyIntensityDrizzleRain,
    ShowerRainAndDrizzle,
    HeavyShowerRainAndDrizzle,
    ShowerDrizzle,
    Unknown(u16),
}

impl From<u16> for DrizzleType {
    fn from(data: u16) -> Self {
        match data {
            300 => Self::LightIntensityDrizzle,
            301 => Self::Drizzle,
            302 => Self::HeavyIntensityDrizzle,
            310 => Self::LightIntensityDrizzleRain,
            311 => Self::DrizzleRain,
            312 => Self::HeavyIntensityDrizzleRain,
            313 => Self::ShowerRainAndDrizzle,
            314 => Self::HeavyShowerRainAndDrizzle,
            321 => Self::ShowerDrizzle,
            _ => Self::Unknown(data),
        }
    }
}

pub enum RainType {
    LightRain,
    ModerateRain,
    HeavyIntensityRain,
    VeryHeavyRain,
    ExtremeRain,
    FreezingRain,
    LightIntensityShowerRain,
    ShowerRain,
    HeavyIntensityShowerRain,
    RaggedShowerRain,
    Unknown(u16),
}

impl From<u16> for RainType {
    fn from(data: u16) -> Self {
        match data {
            500 => Self::LightRain,
            501 => Self::ModerateRain,
            502 => Self::HeavyIntensityRain,
            503 => Self::VeryHeavyRain,
            504 => Self::ExtremeRain,
            511 => Self::FreezingRain,
            520 => Self::LightIntensityShowerRain,
            521 => Self::ShowerRain,
            522 => Self::HeavyIntensityShowerRain,
            531 => Self::RaggedShowerRain,
            _ => Self::Unknown(data),
        }
    }
}

pub enum SnowType {
    LightSnow,
    Snow,
    HeavySnow,
    Sleet,
    LightShowerSleet,
    ShowerSleet,
    LightRainAndSnow,
    RainAndSnow,
    LightShowerSnow,
    ShowerSnow,
    HeavyShowerSnow,
    Unknown(u16),
}

impl From<u16> for SnowType {
    fn from(data: u16) -> Self {
        match data {
            600 => Self::LightSnow,
            601 => Self::Snow,
            602 => Self::HeavySnow,
            611 => Self::Sleet,
            612 => Self::LightShowerSleet,
            613 => Self::ShowerSleet,
            615 => Self::LightRainAndSnow,
            616 => Self::RainAndSnow,
            620 => Self::LightShowerSnow,
            621 => Self::ShowerSnow,
            622 => Self::HeavyShowerSnow,
            _ => Self::Unknown(data),
        }
    }
}

pub enum AtmosphereType {
    Mist,
    Smoke,
    Haze,
    SandDustWhirls,
    Fog,
    Sand,
    Dust,
    VolcanicAsh,
    Squalls,
    Tornado,
    Unknown(u16),
}

impl From<u16> for AtmosphereType {
    fn from(data: u16) -> Self {
        match data {
            701 => Self::Mist,
            711 => Self::Smoke,
            721 => Self::Haze,
            731 => Self::SandDustWhirls,
            741 => Self::Fog,
            751 => Self::Sand,
            761 => Self::Dust,
            762 => Self::VolcanicAsh,
            771 => Self::Squalls,
            781 => Self::Tornado,
            _ => Self::Unknown(data),
        }
    }
}

pub enum CloudsType {
    FewClouds,
    ScatteredClouds,
    BrokenClouds,
    OvercastClouds,
    Unknown(u16),
}

impl From<u16> for CloudsType {
    fn from(data: u16) -> Self {
        match data {
            801 => Self::FewClouds,
            802 => Self::ScatteredClouds,
            803 => Self::BrokenClouds,
            804 => Self::OvercastClouds,
            _ => Self::Unknown(data),
        }
    }
}
