//! LSB steganography in RGBA images

use image::{DynamicImage, ImageBuffer, Rgba};

use super::{CryptoError, Result};

pub struct Steganography;

impl Steganography {
    pub fn hide(image: &DynamicImage, data: &[u8]) -> Result<DynamicImage> {
        let mut rgba = image.to_rgba8();
        let capacity = (rgba.width() as usize) * (rgba.height() as usize) * 4 / 8;
        if data.len() + 4 > capacity {
            return Err(CryptoError::Stego(format!(
                "payload {} > capacity ~{}",
                data.len(),
                capacity
            )));
        }

        let mut payload = (data.len() as u32).to_be_bytes().to_vec();
        payload.extend_from_slice(data);
        let total_bits = payload.len() * 8;
        let mut bit_idx = 0usize;

        for pixel in rgba.pixels_mut() {
            for c in 0..4 {
                if bit_idx >= total_bits {
                    break;
                }
                let byte_i = bit_idx / 8;
                let bit_i = 7 - (bit_idx % 8);
                let bit = (payload[byte_i] >> bit_i) & 1;
                pixel.0[c] = (pixel.0[c] & 0xFE) | bit;
                bit_idx += 1;
            }
            if bit_idx >= total_bits {
                break;
            }
        }
        Ok(DynamicImage::ImageRgba8(rgba))
    }

    pub fn extract(image: &DynamicImage) -> Result<Vec<u8>> {
        let rgba = image.to_rgba8();
        let mut bits = Vec::with_capacity(rgba.len());
        for pixel in rgba.pixels() {
            for c in 0..4 {
                bits.push(pixel.0[c] & 1);
            }
        }
        if bits.len() < 32 {
            return Err(CryptoError::Stego("image too small".into()));
        }
        let mut len_bytes = [0u8; 4];
        for i in 0..32 {
            len_bytes[i / 8] |= bits[i] << (7 - (i % 8));
        }
        let len = u32::from_be_bytes(len_bytes) as usize;
        let need = 32 + len * 8;
        if bits.len() < need {
            return Err(CryptoError::Stego("truncated".into()));
        }
        let mut data = vec![0u8; len];
        for i in 0..(len * 8) {
            data[i / 8] |= bits[32 + i] << (7 - (i % 8));
        }
        Ok(data)
    }

    pub fn hide_in_blank(width: u32, height: u32, data: &[u8]) -> Result<DynamicImage> {
        let img: ImageBuffer<Rgba<u8>, _> =
            ImageBuffer::from_pixel(width, height, Rgba([100, 120, 140, 255]));
        Self::hide(&DynamicImage::ImageRgba8(img), data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hide_extract_roundtrip() {
        let secret = b"Liberty secret payload";
        let img = Steganography::hide_in_blank(64, 64, secret).unwrap();
        let out = Steganography::extract(&img).unwrap();
        assert_eq!(out, secret);
    }
}
