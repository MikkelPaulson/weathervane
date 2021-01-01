use std::convert::{TryFrom, TryInto};
use std::env;
use std::fmt;

pub async fn query() -> Result<(Option<OpenWeatherResponse>, Option<Vec<u8>>), &'static str> {
    let (open_weather, radar_map) = tokio::join!(call_open_weather_api(), get_weather_radar());

    Ok((
        open_weather
            .ok()
            .and_then(|s| json::parse(&s).ok())
            .and_then(|j| j.try_into().ok()),
        radar_map.ok(),
    ))
}

pub struct OpenWeatherResponse {
    pub current: WeatherState,
    pub minutely: Vec<WeatherState>,
    pub hourly: Vec<WeatherState>,
    pub daily: Vec<WeatherState>,
}

impl TryFrom<json::JsonValue> for OpenWeatherResponse {
    type Error = &'static str;

    fn try_from(mut json: json::JsonValue) -> Result<Self, Self::Error> {
        let tz_offset = json
            .remove("timezone_offset")
            .as_i32()
            .map_or(time::UtcOffset::UTC, |i| time::UtcOffset::seconds(i));

        let mut current = WeatherState::try_from(json.remove("current"))?;
        current.time = current.time.to_offset(tz_offset);
        let (sunrise, sunset) = (
            current.sunrise.map(|t| t.to_offset(tz_offset)),
            current.sunset.map(|t| t.to_offset(tz_offset)),
        );

        Ok(Self {
            current,
            minutely: json
                .remove("minutely")
                .members_mut()
                .map(|j| {
                    WeatherState::try_from(j.take()).map(|mut state| {
                        state.time = state.time.to_offset(tz_offset);
                        state
                    })
                })
                .collect::<Result<_, _>>()?,
            hourly: json
                .remove("hourly")
                .members_mut()
                .map(|j| {
                    WeatherState::try_from(j.take()).map(|mut state| {
                        state.time = state.time.to_offset(tz_offset);
                        state.sunrise = sunrise;
                        state.sunset = sunset;
                        state
                    })
                })
                .collect::<Result<_, _>>()?,
            daily: json
                .remove("daily")
                .members_mut()
                .map(|j| {
                    WeatherState::try_from(j.take()).map(|mut state| {
                        state.time = state.time.to_offset(tz_offset);
                        state.sunrise.map(|t| t.to_offset(tz_offset));
                        state.sunset.map(|t| t.to_offset(tz_offset));
                        state
                    })
                })
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
    pub sunrise: Option<time::OffsetDateTime>,
    pub sunset: Option<time::OffsetDateTime>,
    pub temp: Option<Temperature>,
    pub wind: Option<Wind>,
    pub clouds: Option<u8>,
    pub condition: Option<WeatherCondition>,
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
            sunrise: json
                .remove("sunrise")
                .as_i64()
                .map(|sunrise| time::OffsetDateTime::from_unix_timestamp(sunrise)),
            sunset: json
                .remove("sunset")
                .as_i64()
                .map(|sunset| time::OffsetDateTime::from_unix_timestamp(sunset)),
            temp: json.remove("temp").as_f32().map(|temp| temp.into()),
            wind: json
                .remove("wind_speed")
                .as_f32()
                .zip(json.remove("wind_deg").as_u16())
                .map(|(speed, direction)| Wind {
                    speed,
                    direction,
                    gust: json.remove("wind_gust").as_f32(),
                }),
            clouds: json.remove("clouds").as_u8(),
            condition: json
                .remove("weather")
                .members_mut()
                .next()
                .map(|weather| weather.remove("id").as_u16())
                .flatten()
                .map(|id| id.into()),
        })
    }
}

pub struct Temperature(f32);

impl Temperature {
    pub const fn from_kelvin(kelvin: f32) -> Self {
        Self(kelvin)
    }

    pub fn celsius(&self) -> f32 {
        self.0 - 273.15
    }
}

impl From<f32> for Temperature {
    fn from(input: f32) -> Self {
        Self::from_kelvin(input)
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}°", self.celsius().round())
    }
}

pub struct Wind {
    pub speed: f32,
    pub direction: u16,
    pub gust: Option<f32>,
}

impl Wind {
    pub fn speed_km_h(&self) -> f32 {
        self.speed * 3.6
    }

    pub fn gust_km_h(&self) -> Option<f32> {
        self.gust.map(|i| i * 3.6)
    }

    pub fn arrow(&self) -> &'static str {
        match self.direction {
            23..=67 => "⇗",
            68..=112 => "⇒",
            113..=157 => "⇘",
            158..=202 => "⇓",
            203..=247 => "⇙",
            248..=292 => "⇐",
            293..=337 => "⇖",
            _ => "⇑",
        }
    }
}

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

async fn get_weather_radar() -> Result<Vec<u8>, &'static str> {
    let page = reqwest::get(&format!(
        "https://weather.gc.ca/radar/index_e.html?id={}",
        env::var("ENVIRONMENT_CANADA_RADAR_ID").unwrap_or_else(|_| "wmn".to_string())
    ))
    .await
    .map_err(|_| "Failed to load radar page.")?
    .text()
    .await
    .map_err(|_| "Failed to load radar page content.")?;

    if let Some(start) = page.find("/data/radar/temp_image") {
        if let Some(end) = page[start..].find('"') {
            let image_url = &page[start..start + end];

            return Ok(reqwest::get(image_url)
                .await
                .map_err(|_| "Failed to load radar image.")?
                .bytes()
                .await
                .map_err(|_| "Failed to load radar image content.")?
                .iter()
                .copied()
                .collect());
        }
    }
    Err("Failed to parse radar image URL.")
}

async fn call_open_weather_api() -> reqwest::Result<String> {
    reqwest::get(&format!(
        "https://api.openweathermap.org/data/2.5/onecall?lat={}&lon={}&exclude=minutely,daily&appid={}",
        env::var("OPEN_WEATHER_LAT").unwrap_or_else(|_| "45.5".to_string()),
        env::var("OPEN_WEATHER_LON").unwrap_or_else(|_| "-73.6".to_string()),
        env::var("OPEN_WEATHER_API_KEY").expect("Missing required API key."),
    )).await?.text().await
}
