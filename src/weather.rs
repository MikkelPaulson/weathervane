use std::convert::{TryFrom, TryInto};
use std::env;
use std::fmt;

pub fn query() -> Result<(Option<OpenWeatherResponse>, Option<RadarMap>), &'static str> {
    Ok((
        Some(
            json::parse(mock_open_weather_api())
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

/*
async fn call_open_weather_api() {
    reqwest::get(&format!(
            "https://api.openweathermap.org/data/2.5/onecall?lat={}&lon={}&exclude=minutely,daily&appid={}",
            env::var("lat").unwrap_or_else(|_| "45.5".to_string()),
            env::var("lon").unwrap_or_else(|_| "-73.6".to_string()),
            env::var("api_key").expect("Missing required API key."),
            )).text().await;
}
*/

const fn mock_open_weather_api() -> &'static str {
    r#"{"lat":45,"lon":-73,"timezone":"America/New_York","timezone_offset":-18000,"current":{"dt":1609447197,"sunrise":1609417812,"sunset":1609449598,"temp":274.4,"feels_like":268.57,"pressure":1021,"humidity":47,"dew_point":265.31,"uvi":0,"clouds":90,"visibility":10000,"wind_speed":4.1,"wind_deg":290,"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04d"}]},"hourly":[{"dt":1609444800,"temp":274.4,"feels_like":267.92,"pressure":1021,"humidity":47,"dew_point":265.31,"uvi":0.22,"clouds":90,"visibility":10000,"wind_speed":5.03,"wind_deg":289,"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04d"}],"pop":0},{"dt":1609448400,"temp":273.1,"feels_like":267.78,"pressure":1021,"humidity":70,"dew_point":268.84,"uvi":0,"clouds":78,"visibility":10000,"wind_speed":3.89,"wind_deg":292,"weather":[{"id":803,"main":"Clouds","description":"broken clouds","icon":"04d"}],"pop":0},{"dt":1609452000,"temp":271.61,"feels_like":266.69,"pressure":1023,"humidity":84,"dew_point":269.53,"uvi":0,"clouds":58,"visibility":10000,"wind_speed":3.47,"wind_deg":292,"weather":[{"id":803,"main":"Clouds","description":"broken clouds","icon":"04n"}],"pop":0},{"dt":1609455600,"temp":270.82,"feels_like":265.85,"pressure":1024,"humidity":91,"dew_point":269.7,"uvi":0,"clouds":42,"visibility":10000,"wind_speed":3.6,"wind_deg":287,"weather":[{"id":802,"main":"Clouds","description":"scattered clouds","icon":"03n"}],"pop":0},{"dt":1609459200,"temp":270.36,"feels_like":265.52,"pressure":1026,"humidity":94,"dew_point":269.63,"uvi":0,"clouds":33,"visibility":10000,"wind_speed":3.4,"wind_deg":288,"weather":[{"id":802,"main":"Clouds","description":"scattered clouds","icon":"03n"}],"pop":0},{"dt":1609462800,"temp":270.27,"feels_like":265.81,"pressure":1027,"humidity":94,"dew_point":267.44,"uvi":0,"clouds":56,"visibility":10000,"wind_speed":2.84,"wind_deg":285,"weather":[{"id":803,"main":"Clouds","description":"broken clouds","icon":"04n"}],"pop":0},{"dt":1609466400,"temp":270.18,"feels_like":265.74,"pressure":1029,"humidity":94,"dew_point":267.35,"uvi":0,"clouds":54,"visibility":10000,"wind_speed":2.81,"wind_deg":290,"weather":[{"id":803,"main":"Clouds","description":"broken clouds","icon":"04n"}],"pop":0},{"dt":1609470000,"temp":269.69,"feels_like":265.5,"pressure":1030,"humidity":94,"dew_point":267.09,"uvi":0,"clouds":37,"visibility":10000,"wind_speed":2.37,"wind_deg":299,"weather":[{"id":802,"main":"Clouds","description":"scattered clouds","icon":"03n"}],"pop":0},{"dt":1609473600,"temp":269.22,"feels_like":265.17,"pressure":1030,"humidity":94,"dew_point":266.48,"uvi":0,"clouds":28,"visibility":10000,"wind_speed":2.1,"wind_deg":298,"weather":[{"id":802,"main":"Clouds","description":"scattered clouds","icon":"03n"}],"pop":0},{"dt":1609477200,"temp":268.78,"feels_like":264.98,"pressure":1031,"humidity":94,"dew_point":265.94,"uvi":0,"clouds":23,"visibility":10000,"wind_speed":1.67,"wind_deg":290,"weather":[{"id":801,"main":"Clouds","description":"few clouds","icon":"02n"}],"pop":0},{"dt":1609480800,"temp":268.48,"feels_like":265.07,"pressure":1031,"humidity":94,"dew_point":265.67,"uvi":0,"clouds":19,"visibility":10000,"wind_speed":1.07,"wind_deg":266,"weather":[{"id":801,"main":"Clouds","description":"few clouds","icon":"02n"}],"pop":0},{"dt":1609484400,"temp":268.27,"feels_like":265.15,"pressure":1032,"humidity":94,"dew_point":265.46,"uvi":0,"clouds":0,"visibility":10000,"wind_speed":0.63,"wind_deg":233,"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01n"}],"pop":0},{"dt":1609488000,"temp":268.14,"feels_like":264.97,"pressure":1032,"humidity":94,"dew_point":265.24,"uvi":0,"clouds":0,"visibility":10000,"wind_speed":0.68,"wind_deg":200,"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01n"}],"pop":0},{"dt":1609491600,"temp":268.01,"feels_like":264.53,"pressure":1032,"humidity":93,"dew_point":265.03,"uvi":0,"clouds":0,"visibility":10000,"wind_speed":1.08,"wind_deg":170,"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01n"}],"pop":0},{"dt":1609495200,"temp":267.94,"feels_like":264.23,"pressure":1032,"humidity":93,"dew_point":264.9,"uvi":0,"clouds":0,"visibility":10000,"wind_speed":1.41,"wind_deg":158,"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01n"}],"pop":0},{"dt":1609498800,"temp":267.93,"feels_like":264.17,"pressure":1033,"humidity":93,"dew_point":264.86,"uvi":0,"clouds":0,"visibility":10000,"wind_speed":1.48,"wind_deg":157,"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01n"}],"pop":0},{"dt":1609502400,"temp":267.94,"feels_like":264.17,"pressure":1033,"humidity":93,"dew_point":264.92,"uvi":0,"clouds":0,"visibility":10000,"wind_speed":1.49,"wind_deg":169,"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01n"}],"pop":0},{"dt":1609506000,"temp":268.08,"feels_like":264.36,"pressure":1034,"humidity":93,"dew_point":265.06,"uvi":0,"clouds":0,"visibility":10000,"wind_speed":1.43,"wind_deg":148,"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01d"}],"pop":0},{"dt":1609509600,"temp":269.09,"feels_like":265.43,"pressure":1034,"humidity":92,"dew_point":265.39,"uvi":0.31,"clouds":0,"visibility":10000,"wind_speed":1.48,"wind_deg":160,"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01d"}],"pop":0},{"dt":1609513200,"temp":270.17,"feels_like":266.61,"pressure":1034,"humidity":91,"dew_point":265.63,"uvi":0.65,"clouds":0,"visibility":10000,"wind_speed":1.48,"wind_deg":173,"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01d"}],"pop":0},{"dt":1609516800,"temp":271.19,"feels_like":267.44,"pressure":1033,"humidity":91,"dew_point":266.4,"uvi":0.95,"clouds":0,"visibility":10000,"wind_speed":1.91,"wind_deg":213,"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01d"}],"pop":0},{"dt":1609520400,"temp":271.86,"feels_like":268.79,"pressure":1032,"humidity":90,"dew_point":266.68,"uvi":1.04,"clouds":5,"visibility":10000,"wind_speed":1.03,"wind_deg":204,"weather":[{"id":800,"main":"Clear","description":"clear sky","icon":"01d"}],"pop":0},{"dt":1609524000,"temp":272.03,"feels_like":269.18,"pressure":1031,"humidity":90,"dew_point":266.6,"uvi":0.87,"clouds":21,"visibility":10000,"wind_speed":0.74,"wind_deg":134,"weather":[{"id":801,"main":"Clouds","description":"few clouds","icon":"02d"}],"pop":0},{"dt":1609527600,"temp":272.12,"feels_like":269.15,"pressure":1030,"humidity":89,"dew_point":266.59,"uvi":0.51,"clouds":84,"visibility":10000,"wind_speed":0.91,"wind_deg":131,"weather":[{"id":803,"main":"Clouds","description":"broken clouds","icon":"04d"}],"pop":0},{"dt":1609531200,"temp":272.03,"feels_like":269.03,"pressure":1030,"humidity":91,"dew_point":267.29,"uvi":0.21,"clouds":92,"visibility":10000,"wind_speed":0.98,"wind_deg":158,"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04d"}],"pop":0},{"dt":1609534800,"temp":271.02,"feels_like":268.16,"pressure":1029,"humidity":92,"dew_point":266.86,"uvi":0,"clouds":95,"visibility":10000,"wind_speed":0.64,"wind_deg":116,"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04d"}],"pop":0},{"dt":1609538400,"temp":270.75,"feels_like":267.67,"pressure":1028,"humidity":92,"dew_point":266.73,"uvi":0,"clouds":96,"visibility":10000,"wind_speed":0.9,"wind_deg":81,"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04n"}],"pop":0},{"dt":1609542000,"temp":270.63,"feels_like":267.08,"pressure":1028,"humidity":92,"dew_point":266.6,"uvi":0,"clouds":97,"visibility":10000,"wind_speed":1.56,"wind_deg":96,"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04n"}],"pop":0},{"dt":1609545600,"temp":271.04,"feels_like":267.44,"pressure":1027,"humidity":92,"dew_point":267.06,"uvi":0,"clouds":97,"visibility":10000,"wind_speed":1.7,"wind_deg":107,"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04n"}],"pop":0},{"dt":1609549200,"temp":271.07,"feels_like":267.32,"pressure":1026,"humidity":92,"dew_point":267.21,"uvi":0,"clouds":100,"visibility":10000,"wind_speed":1.92,"wind_deg":112,"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04n"}],"pop":0},{"dt":1609552800,"temp":271.05,"feels_like":267.11,"pressure":1025,"humidity":93,"dew_point":267.53,"uvi":0,"clouds":100,"visibility":7446,"wind_speed":2.21,"wind_deg":115,"weather":[{"id":804,"main":"Clouds","description":"overcast clouds","icon":"04n"}],"pop":0.04},{"dt":1609556400,"temp":270.78,"feels_like":266.44,"pressure":1024,"humidity":95,"dew_point":268.41,"uvi":0,"clouds":100,"visibility":4588,"wind_speed":2.79,"wind_deg":146,"weather":[{"id":600,"main":"Snow","description":"light snow","icon":"13n"}],"pop":0.28,"snow":{"1h":0.19}},{"dt":1609560000,"temp":270.62,"feels_like":266.72,"pressure":1023,"humidity":97,"dew_point":269.23,"uvi":0,"clouds":100,"visibility":2763,"wind_speed":2.18,"wind_deg":143,"weather":[{"id":600,"main":"Snow","description":"light snow","icon":"13n"}],"pop":0.52,"snow":{"1h":0.25}},{"dt":1609563600,"temp":270.74,"feels_like":266.39,"pressure":1022,"humidity":97,"dew_point":269.52,"uvi":0,"clouds":100,"visibility":1987,"wind_speed":2.84,"wind_deg":125,"weather":[{"id":600,"main":"Snow","description":"light snow","icon":"13n"}],"pop":0.8,"snow":{"1h":0.31}},{"dt":1609567200,"temp":271.05,"feels_like":266.71,"pressure":1020,"humidity":97,"dew_point":269.81,"uvi":0,"clouds":100,"visibility":10000,"wind_speed":2.88,"wind_deg":106,"weather":[{"id":600,"main":"Snow","description":"light snow","icon":"13n"}],"pop":1,"snow":{"1h":0.38}},{"dt":1609570800,"temp":271.27,"feels_like":266.66,"pressure":1019,"humidity":97,"dew_point":269.85,"uvi":0,"clouds":100,"visibility":983,"wind_speed":3.3,"wind_deg":106,"weather":[{"id":600,"main":"Snow","description":"light snow","icon":"13n"}],"pop":1,"snow":{"1h":0.13}},{"dt":1609574400,"temp":271.29,"feels_like":266.32,"pressure":1017,"humidity":98,"dew_point":270.49,"uvi":0,"clouds":100,"visibility":316,"wind_speed":3.84,"wind_deg":101,"weather":[{"id":601,"main":"Snow","description":"snow","icon":"13n"}],"pop":1,"snow":{"1h":0.69}},{"dt":1609578000,"temp":271.77,"feels_like":267.34,"pressure":1014,"humidity":98,"dew_point":271.07,"uvi":0,"clouds":100,"visibility":262,"wind_speed":3.17,"wind_deg":107,"weather":[{"id":601,"main":"Snow","description":"snow","icon":"13n"}],"pop":1,"snow":{"1h":0.56}},{"dt":1609581600,"temp":271.82,"feels_like":267.93,"pressure":1012,"humidity":99,"dew_point":271.34,"uvi":0,"clouds":100,"visibility":203,"wind_speed":2.43,"wind_deg":116,"weather":[{"id":601,"main":"Snow","description":"snow","icon":"13n"}],"pop":1,"snow":{"1h":0.88}},{"dt":1609585200,"temp":271.94,"feels_like":268.33,"pressure":1010,"humidity":98,"dew_point":271.37,"uvi":0,"clouds":100,"visibility":746,"wind_speed":2.03,"wind_deg":86,"weather":[{"id":601,"main":"Snow","description":"snow","icon":"13n"}],"pop":1,"snow":{"1h":0.88}},{"dt":1609588800,"temp":271.92,"feels_like":268.67,"pressure":1009,"humidity":99,"dew_point":271.64,"uvi":0,"clouds":100,"visibility":124,"wind_speed":1.54,"wind_deg":37,"weather":[{"id":601,"main":"Snow","description":"snow","icon":"13n"}],"pop":1,"snow":{"1h":2.88}},{"dt":1609592400,"temp":272.05,"feels_like":268.97,"pressure":1008,"humidity":99,"dew_point":271.76,"uvi":0,"clouds":100,"visibility":251,"wind_speed":1.32,"wind_deg":28,"weather":[{"id":601,"main":"Snow","description":"snow","icon":"13d"}],"pop":1,"snow":{"1h":2.81}},{"dt":1609596000,"temp":272.11,"feels_like":268.57,"pressure":1008,"humidity":99,"dew_point":271.85,"uvi":0.07,"clouds":100,"visibility":209,"wind_speed":1.99,"wind_deg":36,"weather":[{"id":601,"main":"Snow","description":"snow","icon":"13d"}],"pop":1,"snow":{"1h":1.5}},{"dt":1609599600,"temp":272.41,"feels_like":269.07,"pressure":1007,"humidity":99,"dew_point":272.08,"uvi":0.14,"clouds":100,"visibility":626,"wind_speed":1.76,"wind_deg":13,"weather":[{"id":601,"main":"Snow","description":"snow","icon":"13d"}],"pop":1,"snow":{"1h":0.69}},{"dt":1609603200,"temp":272.42,"feels_like":268.97,"pressure":1007,"humidity":99,"dew_point":272.18,"uvi":0.29,"clouds":100,"visibility":130,"wind_speed":1.91,"wind_deg":1,"weather":[{"id":601,"main":"Snow","description":"snow","icon":"13d"}],"pop":1,"snow":{"1h":0.81}},{"dt":1609606800,"temp":272.37,"feels_like":267.55,"pressure":1007,"humidity":99,"dew_point":272.11,"uvi":0.32,"clouds":100,"visibility":100,"wind_speed":3.86,"wind_deg":322,"weather":[{"id":601,"main":"Snow","description":"snow","icon":"13d"}],"pop":1,"snow":{"1h":1}},{"dt":1609610400,"temp":271.55,"feels_like":266.09,"pressure":1007,"humidity":99,"dew_point":271.16,"uvi":0.27,"clouds":100,"visibility":122,"wind_speed":4.62,"wind_deg":325,"weather":[{"id":601,"main":"Snow","description":"snow","icon":"13d"}],"pop":1,"snow":{"1h":0.81}},{"dt":1609614000,"temp":271.28,"feels_like":265.31,"pressure":1008,"humidity":98,"dew_point":270.65,"uvi":0.45,"clouds":100,"visibility":537,"wind_speed":5.28,"wind_deg":316,"weather":[{"id":600,"main":"Snow","description":"light snow","icon":"13d"}],"pop":0.64,"snow":{"1h":0.31}}]}"#
}
