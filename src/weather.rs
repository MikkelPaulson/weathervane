use std::convert::{TryFrom, TryInto};
use std::fmt;

pub fn query() -> Result<(Option<OpenWeatherResponse>, Option<RadarMap>), &'static str> {
    Ok((
        Some(
            json::parse(mock_weather_response())
                .map_err(|_| "Invalid JSON input.")?
                .try_into()?,
        ),
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
    pub clouds: u8,
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
            clouds: json
                .remove("clouds")
                .as_u8()
                .ok_or("Empty or invalid \"clouds\" value.")?,
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
}

impl Wind {
    pub fn km_h(&self) -> f32 {
        self.speed * 3.6
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

const fn mock_weather_response() -> &'static str {
    r#"{"lat":45,"lon":-73,"timezone":"America/New_York","timezone_offset":-18000,"current":{"dt":1609444246,"sunrise":1609417812,"sunset":1609449598,"temp":274.42,"feels_like":269.38,"pressure":1021,"humidity":51,"dew_point":266.27,"uvi":0.22,"clouds":90,"visibility":10000,"wind_speed":3.1,"wind_deg":280,"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04d"}]},"daily":[{"dt":1609430400,"sunrise":1609417812,"sunset":1609449598,"temp":{"day":273.92,"min":269.22,"max":275.35,"night":269.22,"eve":271.07,"morn":275.01},"feels_like":{"day":267.22,"night":265.17,"eve":266.23,"morn":269.44},"pressure":1017,"humidity":88,"dew_point":269.45,"wind_speed":6.54,"wind_deg":280,"weather":[{"id":616,"main":"Snow","description":"rain and snow","icon":"13d"}],"clouds":46,"pop":1,"rain":0.44,"snow":0.95,"uvi":0.95},{"dt":1609516800,"sunrise":1609504215,"sunset":1609536051,"temp":{"day":271.19,"min":267.93,"max":272.12,"night":270.62,"eve":270.75,"morn":267.94},"feels_like":{"day":267.44,"night":266.72,"eve":267.67,"morn":264.23},"pressure":1033,"humidity":91,"dew_point":266.4,"wind_speed":1.91,"wind_deg":213,"weather":[{"id":600,"main":"Snow","description":"light snow","icon":"13d"}],"clouds":0,"pop":0.52,"snow":0.44,"uvi":1.04},{"dt":1609603200,"sunrise":1609590615,"sunset":1609622506,"temp":{"day":272.42,"min":267.72,"max":272.42,"night":267.72,"eve":270.64,"morn":271.82},"feels_like":{"day":268.97,"night":264.32,"eve":265.92,"morn":267.93},"pressure":1007,"humidity":99,"dew_point":272.18,"wind_speed":1.91,"wind_deg":1,"weather":[{"id":601,"main":"Snow","description":"snow","icon":"13d"}],"clouds":100,"pop":1,"snow":14.9,"uvi":0.45},{"dt":1609689600,"sunrise":1609677012,"sunset":1609708962,"temp":{"day":270.42,"min":266.39,"max":272.78,"night":271.06,"eve":270.38,"morn":267.19},"feels_like":{"day":266.69,"night":267.79,"eve":267.58,"morn":263.74},"pressure":1020,"humidity":97,"dew_point":269.03,"wind_speed":1.9,"wind_deg":70,"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01d"}],"clouds":0,"pop":0.04,"uvi":1.23},{"dt":1609776000,"sunrise":1609763407,"sunset":1609795421,"temp":{"day":271.26,"min":268.03,"max":271.51,"night":268.64,"eve":271.18,"morn":271.43},"feels_like":{"day":267.57,"night":264.47,"eve":266.45,"morn":268.22},"pressure":1013,"humidity":98,"dew_point":270.5,"wind_speed":2.01,"wind_deg":343,"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04d"}],"clouds":100,"pop":0.28,"uvi":1.05},{"dt":1609862400,"sunrise":1609849799,"sunset":1609881881,"temp":{"day":269.76,"min":268.27,"max":271.86,"night":269.84,"eve":271.86,"morn":268.64},"feels_like":{"day":265.11,"night":265.34,"eve":266.81,"morn":264.46},"pressure":1012,"humidity":98,"dew_point":269.09,"wind_speed":3.13,"wind_deg":337,"weather":[{"id":600,"main":"Snow","description":"light snow","icon":"13d"}],"clouds":100,"pop":0.68,"snow":3.08,"uvi":2},{"dt":1609948800,"sunrise":1609936189,"sunset":1609968343,"temp":{"day":267.93,"min":264.26,"max":270.54,"night":266.24,"eve":268.74,"morn":265.2},"feels_like":{"day":264.68,"night":262.87,"eve":265.76,"morn":261.83},"pressure":1023,"humidity":97,"dew_point":266.85,"wind_speed":0.82,"wind_deg":265,"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01d"}],"clouds":2,"pop":0.2,"uvi":2},{"dt":1610035200,"sunrise":1610022576,"sunset":1610054807,"temp":{"day":270.7,"min":266.08,"max":272.95,"night":270.86,"eve":271.91,"morn":266.15},"feels_like":{"day":267.46,"night":267.89,"eve":268.76,"morn":262.65},"pressure":1023,"humidity":97,"dew_point":269.28,"wind_speed":1.24,"wind_deg":126,"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04d"}],"clouds":97,"pop":0,"uvi":2}]}"#
}
