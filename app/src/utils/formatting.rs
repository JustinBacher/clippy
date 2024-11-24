use std::{io::Cursor, mem::size_of_val};

use image::ImageReader;
use size::Size;

use crate::database::ClipEntry;

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
pub fn truncate(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => s[..idx].into(),
        None => s.to_string(),
    }
}

fn detect_image(payload: &[u8]) -> Option<String> {
    let Ok(image_reader) = ImageReader::new(Cursor::new(&payload)).with_guessed_format() else {
        return None;
    };
    let mime_type = image_reader.format()?.to_mime_type();
    let Ok((width, height)) = image_reader.into_dimensions() else {
        return None;
    };
    let size = Size::from_bytes(size_of_val(payload));

    let output = format!("[[ binary data {size} {mime_type} {width}x{height} ]]");

    Some(output)
}

pub fn format_entry(entry: &ClipEntry, width: usize, include_dates: bool) -> String {
    let payload = match detect_image(&entry.payload) {
        Some(image) => image,
        None => match width {
            0 => entry.text().unwrap(),
            _ => truncate(&entry.text().unwrap(), width),
        },
    };

    let date = entry.epoch.0.format("%c").to_string();
    match include_dates {
        true => format!("{}:\t{}", date, payload),
        false => payload,
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use image::{ImageFormat::Png, Rgb, RgbImage};
    use pretty_assertions::assert_eq;

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

        let mut buf = Box::new(Cursor::new(Vec::new()));
        mock_image.write_to(buf.as_mut(), Png).unwrap();

        let output = detect_image(buf.into_inner().as_slice()).unwrap();

        assert_eq!(output, "[[ binary data 233 bytes image/png 32x32 ]]")
    }
}
