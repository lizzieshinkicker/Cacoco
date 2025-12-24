use crate::render::palette::DoomPalette;

pub fn decode_doom_patch(data: &[u8], palette: &DoomPalette) -> Option<(u32, u32, i16, i16, Vec<u8>)> {
    if data.len() < 8 { return None; }

    let width = u16::from_le_bytes(data[0..2].try_into().ok()?) as u32;
    let height = u16::from_le_bytes(data[2..4].try_into().ok()?) as u32;
    let left_offset = i16::from_le_bytes(data[4..6].try_into().ok()?);
    let top_offset = i16::from_le_bytes(data[6..8].try_into().ok()?);

    if width > 4096 || height > 4096 || width == 0 || height == 0 { return None; }

    let mut pixels = vec![0u8; (width * height * 4) as usize];

    let col_offsets_start = 8;
    let col_offsets_end = 8 + (width as usize * 4);
    if data.len() < col_offsets_end { return None; }

    for x in 0..width {
        let offset_pos = col_offsets_start + (x as usize * 4);
        let col_offset = u32::from_le_bytes(data[offset_pos..offset_pos + 4].try_into().ok()?) as usize;
        let mut cursor = col_offset;

        while cursor < data.len() {
            let row_start = data[cursor];
            if row_start == 255 { break; }
            if cursor + 2 >= data.len() { break; }
            let len = data[cursor + 1] as usize;
            cursor += 3;

            for i in 0..len {
                if cursor >= data.len() { break; }
                let color_idx = data[cursor];
                cursor += 1;

                let y = row_start as usize + i;
                if y < height as usize {
                    let pixel_idx = ((y * width as usize) + x as usize) * 4;
                    if pixel_idx + 3 < pixels.len() {
                        let color = palette.get(color_idx);
                        pixels[pixel_idx] = color.r();
                        pixels[pixel_idx + 1] = color.g();
                        pixels[pixel_idx + 2] = color.b();
                        pixels[pixel_idx + 3] = 255;
                    }
                }
            }
            cursor += 1;
        }
    }
    Some((width, height, left_offset, top_offset, pixels))
}

pub fn decode_doom_flat(data: &[u8], palette: &DoomPalette) -> Option<(u32, u32, Vec<u8>)> {
    if data.len() != 4096 { return None; }

    let width = 64;
    let height = 64;
    let mut pixels = vec![0u8; width * height * 4];

    for i in 0..4096 {
        let color = palette.get(data[i]);
        let pix_idx = i * 4;
        pixels[pix_idx] = color.r();
        pixels[pix_idx + 1] = color.g();
        pixels[pix_idx + 2] = color.b();
        pixels[pix_idx + 3] = 255;
    }

    Some((width as u32, height as u32, pixels))
}