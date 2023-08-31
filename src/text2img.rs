use anyhow::{anyhow, Result};
use image::{
    codecs::png::{CompressionType, FilterType, PngEncoder},
    ImageBuffer, Rgb, RgbImage,
};
use rusttype::{Font, Scale};

pub const CHAR_WIDTH: usize = 25;
pub const CHAR_HEIGHT: usize = 25;

pub fn text_to_image(text: &str) -> Result<Vec<u8>> {
    let lines = text.lines().collect::<Vec<_>>();
    let (image_width, image_height) = (
        CHAR_WIDTH * lines[0].chars().count(),
        CHAR_HEIGHT * lines.len(),
    );

    // Create a new RGB image buffer
    let mut image: RgbImage = ImageBuffer::new(image_width as u32, image_height as u32);

    // Set the background color to white
    // let background_color = Rgb([255, 255, 255]);
    // for pixel in image.pixels_mut() {
    //     *pixel = background_color;
    // }

    // Set the text color
    let text_color = Rgb([255, 255, 255]);

    // Set font
    let font_size = 20.0;
    let font_data: &[u8] = include_bytes!("../font/Roboto.ttf");
    let font: Font<'static> =
        Font::try_from_bytes(font_data).ok_or(anyhow!("Failed to load braille font"))?;

    // Add the text to the image
    for (line_id, line) in lines.iter().enumerate() {
        for (cid, ch) in line.chars().enumerate() {
            imageproc::drawing::draw_text_mut(
                &mut image,
                text_color,
                (cid * CHAR_WIDTH) as i32,
                (line_id * CHAR_HEIGHT) as i32,
                Scale::uniform(font_size),
                &font,
                &ch.to_string(),
            );
        }
    }

    // output the image datas
    let mut img_bytes: Vec<u8> = Vec::new();
    let encoder =
        PngEncoder::new_with_quality(&mut img_bytes, CompressionType::Best, FilterType::NoFilter);
    image.write_with_encoder(encoder)?;

    Ok(img_bytes)
}
