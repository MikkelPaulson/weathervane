use display::Display;

pub mod display;
pub mod image;
pub mod weather;

pub async fn refresh() -> Result<(), &'static str> {
    let (weather_report, weather_radar) = weather::query().await.unwrap();
    let mut display = display::waveshare::EPaper3_7in::new();

    display.on()?;
    display.draw_context(|ctx| {
        image::render(weather_report, weather_radar, ctx);
    })?;
    display.sleep()?;

    Ok(())
}
