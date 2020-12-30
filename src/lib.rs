use display::Display;

pub mod display;
pub mod image;
pub mod weather;

fn refresh() -> Result<(), &'static str> {
    let weather_report = weather::query();
    let mut display = display::waveshare::EPaper3_7in::new();

    display.on()?;
    display.draw_context(|ctx| {
        // image::render(weather_report, ctx);
    })?;
    display.sleep()?;

    Ok(())
}
