use std::{format, io::Cursor, println, vec};

use anyhow::{anyhow, Result};
use image::{io::Reader as ImageReader, DynamicImage, GenericImage, GenericImageView, Rgba};
use rayon::{prelude::ParallelIterator, str::ParallelString};

use crate::text2img::{text_to_image, CHAR_HEIGHT, CHAR_WIDTH};

pub struct Image(pub DynamicImage);

impl TryFrom<String> for Image {
    type Error = anyhow::Error;

    fn try_from(path: String) -> std::result::Result<Self, Self::Error> {
        Ok(Image(ImageReader::open(path)?.decode()?))
    }
}

pub trait ImageBlur {
    fn blur(&self, sigma: Option<usize>, round: Option<usize>) -> DynamicImage;
    fn unblur_text(&self) -> Result<String>;
}

const ALPHABET: &str = r#"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 !.:()@#$%^&*_+=-{}[]<>,/?'"\|"#;
impl ImageBlur for Image {
    fn blur(&self, box_size: Option<usize>, round: Option<usize>) -> DynamicImage
    where
        Self: std::marker::Sized,
    {
        let box_size = box_size.map_or(15, |s| match s % 2 == 0 {
            true => (s + 1).max(3),
            false => s.max(3),
        });
        let radius = (box_size as u32 - 1) / 2;

        let mut dynamic_image = self.0.to_owned();
        let (width, height) = (dynamic_image.width(), dynamic_image.height());

        for _ in 0..round.unwrap_or(1) {
            for y in 0..height {
                for x in 0..width {
                    let mut matrix_pos = vec![];
                    for x_neighbord in
                        (if x < radius { 0 } else { x - radius })..=(x + radius).min(width - 1)
                    {
                        for y_neighbord in
                            (if y < radius { 0 } else { y - radius })..=(y + radius).min(height - 1)
                        {
                            matrix_pos.push(dynamic_image.get_pixel(x_neighbord, y_neighbord))
                        }
                    }

                    let matrix_len = matrix_pos.len();
                    let matrix_avg = matrix_pos
                        .into_iter()
                        .fold(
                            [0, 0, 0, 0] as [usize; 4],
                            |[avg_r, avg_g, avg_b, avg_a], Rgba([r, g, b, a])| {
                                [
                                    avg_r + r as usize,
                                    avg_g + g as usize,
                                    avg_b + b as usize,
                                    avg_a + a as usize,
                                ]
                            },
                        )
                        .map(|rgb_val| (rgb_val / matrix_len) as u8);
                    dynamic_image.put_pixel(x, y, Rgba(matrix_avg));
                }
            }
        }

        dynamic_image
    }

    fn unblur_text(&self) -> Result<String> {
        let img_chars = ALPHABET
            .par_chars()
            .map(|ch| {
                let char_bytes = text_to_image(&ch.to_string())?;
                ImageReader::new(Cursor::new(char_bytes))
                    .with_guessed_format()
                    .map_err(|_| anyhow!(""))?
                    .decode()
                    .map_err(|_| anyhow!(""))
            })
            .collect::<Result<Vec<_>, anyhow::Error>>()?;

        let dynimage = self.0.to_owned();
        let mut result = String::new();

        let mut sigma = None;
        let (chars_count, lines_count) = (
            dynimage.width() as usize / CHAR_WIDTH,
            dynimage.height() as usize / CHAR_HEIGHT,
        );
        for lid in 0..lines_count {
            for cid in 0..chars_count {
                let result_char = match sigma {
                    Some(sigma) => find_character(lid, cid, &dynimage, &img_chars, sigma, false),
                    None => {
                        let mut search_sigma = 0.0;
                        loop {
                            if let Some(found_char) =
                                find_character(0, 0, &dynimage, &img_chars, search_sigma, true)
                            {
                                sigma =
                                    Some(format!("{:.2}", search_sigma).parse::<f64>().unwrap());
                                println!("{sigma:?}");
                                break Some(found_char);
                            }

                            search_sigma += 0.1;
                            if search_sigma > 5.0 {
                                break None;
                            }
                        }
                    }
                };
                if let Some(found_char) = result_char {
                    result.push(found_char);
                }
            }
        }

        Ok(result)
    }
}

fn find_character(
    lid: usize,
    cid: usize,
    dynimage: &DynamicImage,
    img_chars: &[DynamicImage],
    sigma: f64,
    strict: bool,
) -> Option<char> {
    let mut dynimage_cid_pixels = vec![];
    for x in (cid * CHAR_WIDTH)..(CHAR_WIDTH * (cid + 1)) {
        let mut pixel_raw = vec![];
        for y in (lid * CHAR_HEIGHT)..(CHAR_HEIGHT * (lid + 1)) {
            let pixel = dynimage.get_pixel(x as u32, y as u32);
            pixel_raw.push(pixel);
        }

        if pixel_raw.iter().all(|rgba| rgba == &Rgba([0, 0, 0, 255])) {
            break;
        }
        dynimage_cid_pixels.append(&mut pixel_raw);
    }

    let mut guess = vec![];
    for (ch_id, ch_img) in img_chars.iter().enumerate() {
        let blurred_ch = ch_img.blur(sigma as f32);

        let mut blurred_cid_pixels = vec![];
        for x in 0..CHAR_WIDTH {
            let mut pixel_raw = vec![];
            for y in 0..CHAR_HEIGHT {
                let pixel = blurred_ch.get_pixel(x as u32, y as u32);
                pixel_raw.push(pixel);
            }

            if pixel_raw.iter().all(|rgba| rgba == &Rgba([0, 0, 0, 255])) {
                break;
            }
            blurred_cid_pixels.append(&mut pixel_raw);
        }

        if blurred_cid_pixels.is_empty() && dynimage_cid_pixels.is_empty() {
            return Some(' ');
        }
        if blurred_cid_pixels.len() == dynimage_cid_pixels.len() {
            let mut commun_pixel = 0;
            for i in 0..blurred_cid_pixels.len() {
                let (a, b) = (blurred_cid_pixels[i], dynimage_cid_pixels[i]);
                let diff = ((a[0] as isize - b[0] as isize)
                    + (a[1] as isize - b[1] as isize)
                    + (a[2] as isize - b[2] as isize)
                    + (a[3] as isize - b[3] as isize))
                    .abs();

                if (strict && diff <= 5) || (!strict && diff <= 10) {
                    commun_pixel += 1;
                }
            }
            let match_percent = (commun_pixel * 100) / blurred_cid_pixels.len();
            let letter = ALPHABET.chars().nth(ch_id).unwrap();
            guess.push((letter, match_percent));
        }
    }

    // println!("{sigma:.2} - {guess:?}");
    let possible_guess = guess.into_iter().max_by(|(_, a), (_, b)| a.cmp(b));
    if let Some((letter, match_percent)) = possible_guess {
        if (strict && match_percent >= 100) || (!strict && match_percent >= 70) {
            return Some(letter);
        }
    }
    None
}
