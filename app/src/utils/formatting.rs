use std::{io::Cursor, mem::size_of_val};

use chrono::DateTime;
use image::ImageReader;
use redb::AccessGuard;
use size::Size;

use crate::prelude::Result;

pub fn trim(s: &[u8]) -> Vec<u8> {
    // Original from https://stackoverflow.com/a/67358195 just changed to be used on vectors
    let from = match s.iter().position(|c| !c.is_ascii_whitespace()) {
        Some(i) => i,
        None => return s[0..0].into(),
    };
    let to = s.iter().rposition(|c| !c.is_ascii_whitespace()).unwrap();
    s[from..=to].into()
}

// https://stackoverflow.com/a/38461750
pub fn truncate(s: String, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => s[..idx].into(),
    }
}

fn get_size(payload: Vec<u8>) -> Size {
    Size::from_bytes(size_of_val(&payload))
}

fn detect_image(payload: Vec<u8>) -> Result<Option<String>> {
    let Ok(image_reader) = ImageReader::new(Cursor::new(&payload)).with_guessed_format() else {
        return Ok(None);
    };
    let Some(format) = image_reader.format() else {
        return Ok(None);
    };
    let Ok((width, height)) = image_reader.into_dimensions() else {
        return Ok(None);
    };

    Ok(Some(format!(
        "[[ binary data {} {} {width}x{height} ]]",
        get_size(payload),
        format.to_mime_type(),
    )))
}

pub fn format_entry(
    (k, v): (AccessGuard<i64>, AccessGuard<Vec<u8>>),
    width: usize,
) -> (String, String) {
    let date = DateTime::from_timestamp_millis(k.value()).unwrap().format("%c").to_string();

    let data = if let Ok(Some(image)) = detect_image(v.value()) {
        image
    } else {
        let clip = String::from_utf8(trim(&v.value())).unwrap();
        if width == 0 {
            clip
        } else {
            truncate(clip, width)
        }
    };

    (date, data)
}

#[cfg(test)]
mod test {
    use image::{DynamicImage, Rgb, RgbImage};

    use super::*;

    #[test]
    fn it_previews_images() {
        let mut mock_image = RgbImage::new(32, 32);
        for x in 15..=17 {
            for y in 8..24 {
                mock_image.put_pixel(x, y, Rgb([255, 0, 0]));
                mock_image.put_pixel(y, x, Rgb([255, 0, 0]));
            }
        }

        let output = detect_image(mock_image.into_vec()).unwrap();

        println!("{:?}", output);
    }
}
