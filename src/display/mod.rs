use std::fmt;

use dither::ditherer::Dither;
use piet::RenderContext;

pub mod waveshare;

pub trait Display {
    type Err: fmt::Debug;

    /// Initialize the display.
    fn on(&mut self) -> Result<(), Self::Err>;

    /// Clear the display and power it down.
    fn off(&mut self) -> Result<(), Self::Err>;

    /// Put the display in low-power mode. This may or may not be the same as `off`.
    fn sleep(&mut self) -> Result<(), Self::Err>;

    /// Draw an image on the display. The image is represented as bytes in the range
    /// `0..self.get_color_depth()`, with 0 being black, so the input should have length
    /// `display_width * display_height`.
    fn draw(&mut self, image: impl IntoIterator<Item = u8>) -> Result<(), Self::Err>;

    /// Get the dimensions of the display in pixels (width, height).
    fn get_dimensions(&self) -> (usize, usize);

    /// Get the number of colours supported by the display. Used for dithering.
    fn get_color_depth(&self) -> u8;

    /// Draw a 32-bit RGB image dithered to the available colour depth. The image is represented as
    /// RGB bytes, so the input should have length `display_width * display_height * 3`.
    fn draw_dithered<'a>(&mut self, image: impl IntoIterator<Item = u8>) -> Result<(), Self::Err> {
        let (display_width, _) = self.get_dimensions();
        let mut image_iter = image.into_iter();
        let color_depth = self.get_color_depth();

        self.draw(
            dither::ditherer::FLOYD_STEINBERG
                .dither(
                    dither::prelude::Img::new(
                        (0..)
                            .map(|_| {
                                // Map pixels to f64 in range 0.0..255.0
                                if let (Some(r), Some(g), Some(b)) =
                                    (image_iter.next(), image_iter.next(), image_iter.next())
                                {
                                    Some(
                                        dither::color::RGB(r as f64, g as f64, b as f64)
                                            .to_chroma_corrected_black_and_white(),
                                    )
                                } else {
                                    None
                                }
                            })
                            .take_while(|x| x.is_some())
                            .map(|x| x.unwrap()),
                        display_width as u32,
                    )
                    .unwrap(),
                    dither::create_quantize_n_bits_func(color_depth - 1).unwrap(),
                )
                .iter()
                .map(|x| (x / 255. * (color_depth - 1) as f64) as u8),
        )
    }

    fn draw_context<F: FnOnce(&mut piet_cairo::CairoRenderContext)>(
        &mut self,
        f: F,
    ) -> Result<(), Self::Err> {
        let (display_width, display_height) = self.get_dimensions();
        let mut device = piet_common::Device::new().unwrap();
        let mut bitmap_target = device
            .bitmap_target(display_width, display_height, 1.)
            .unwrap();

        let mut render_context = bitmap_target.render_context();
        render_context.clear(piet::Color::WHITE);
        f(&mut render_context);

        self.draw_dithered(
            bitmap_target
                .to_image_buf(piet_common::ImageFormat::RgbaPremul)
                .unwrap()
                .raw_pixels()
                .iter()
                .enumerate()
                .filter_map(|(index, pixel)| if index % 4 == 3 { None } else { Some(pixel) })
                .copied(),
        )
    }
}
