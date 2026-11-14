use std::{env, fs, path::PathBuf};

fn ensure_windows_icon() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"));
    let icon_dir = manifest_dir.join("icons");
    let icon_path = icon_dir.join("icon.ico");

    if icon_path.exists() {
        return;
    }

    fs::create_dir_all(&icon_dir).expect("create Tauri icon directory");

    // Build a small transparent 32x32 Crescent mark as a Windows ICO.
    // Pixels are stored bottom-up in BGRA order, as required by BMP/ICO.
    let width = 32u32;
    let height = 32u32;
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in (0..height).rev() {
        for x in 0..width {
            let dx = x as f32 - 14.0;
            let dy = y as f32 - 16.0;
            let outer = dx * dx + dy * dy <= 12.5 * 12.5;
            let cut_x = x as f32 - 21.0;
            let cut_y = y as f32 - 11.0;
            let cutout = cut_x * cut_x + cut_y * cut_y <= 11.5 * 11.5;
            if outer && !cutout {
                pixels.extend_from_slice(&[235, 231, 220, 255]); // warm silver, BGRA
            } else {
                pixels.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    let mask = vec![0u8; ((width + 31) / 32 * 4 * height) as usize];
    let image_size = pixels.len() + mask.len();
    let mut dib = Vec::with_capacity(40 + image_size);

    dib.extend_from_slice(&40u32.to_le_bytes()); // BITMAPINFOHEADER size
    dib.extend_from_slice(&width.to_le_bytes());
    dib.extend_from_slice(&(height * 2).to_le_bytes()); // XOR + AND planes
    dib.extend_from_slice(&1u16.to_le_bytes());
    dib.extend_from_slice(&32u16.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    dib.extend_from_slice(&(image_size as u32).to_le_bytes());
    dib.extend_from_slice(&0i32.to_le_bytes());
    dib.extend_from_slice(&0i32.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());
    dib.extend_from_slice(&0u32.to_le_bytes());
    dib.extend_from_slice(&pixels);
    dib.extend_from_slice(&mask);

    let mut ico = Vec::with_capacity(22 + dib.len());
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&[width as u8, height as u8, 0, 0]);
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&32u16.to_le_bytes());
    ico.extend_from_slice(&(dib.len() as u32).to_le_bytes());
    ico.extend_from_slice(&22u32.to_le_bytes());
    ico.extend_from_slice(&dib);

    fs::write(icon_path, ico).expect("write generated Tauri Windows icon");
}

fn main() {
    ensure_windows_icon();
    tauri_build::build();
}
