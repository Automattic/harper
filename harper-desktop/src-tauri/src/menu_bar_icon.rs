use tauri::image::Image;

fn service_status_color(is_running: bool) -> [u8; 4] {
    match is_running {
        true => [34, 197, 94, 255],
        false => [239, 68, 68, 255],
    }
}

pub fn menu_bar_icon(is_running: bool) -> tauri::Result<Image<'static>> {
    let icon = Image::from_bytes(include_bytes!("../icons/menu-bar-icon.png"))?;
    let width = icon.width();
    let height = icon.height();
    let mut rgba = icon.rgba().to_vec();

    draw_status_line(&mut rgba, width, height, service_status_color(is_running));

    Ok(Image::new_owned(rgba, width, height))
}

fn draw_status_line(rgba: &mut [u8], width: u32, height: u32, color: [u8; 4]) {
    let line_height = (height.min(width) / 14).max(3);
    let start_y = height.saturating_sub(line_height);

    for y in start_y..height {
        for x in 0..width {
            let index = ((y * width + x) * 4) as usize;
            rgba[index..index + 4].copy_from_slice(&color);
        }
    }
}
