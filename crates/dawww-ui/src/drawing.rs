use std::io::{self, Write};
use ab_glyph::{Font, FontRef, ScaleFont};
use base64::Engine as _;
use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};
use nix::sys::mman::{shm_open, shm_unlink};
use nix::sys::stat::Mode;
use nix::unistd::{ftruncate, close};
use nix::fcntl::OFlag;
use rand::Rng;
use std::fs::File;
use std::path::PathBuf;
use std::env;
use std::io::Cursor;
use png::{ColorType, Encoder};

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
    pub cols: usize,
    pub rows: usize,
}

pub fn get_terminal_dimensions() -> io::Result<Dimensions> {
    let mut sz: winsize = unsafe { std::mem::zeroed() };
    if unsafe { ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut sz) } == -1 {
        return Err(io::Error::last_os_error());
    }

    let width = sz.ws_xpixel as usize;
    let height = sz.ws_ypixel as usize;
    let cols = sz.ws_col as usize;
    let rows = sz.ws_row as usize;

    Ok(Dimensions { width, height, cols, rows })
}

pub fn draw_line(
    image_data: &mut [u8],
    width: usize,
    height: usize,
    start: Position,
    end: Position,
    thickness: usize,
    color: (u8, u8, u8, u8),
) {
    let dx = (end.x as i32 - start.x as i32).abs();
    let dy = (end.y as i32 - start.y as i32).abs();
    let sx = if start.x < end.x { 1 } else { -1 };
    let sy = if start.y < end.y { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = start.x as i32;
    let mut y = start.y as i32;

    while x != end.x as i32 || y != end.y as i32 {
        for i in -(thickness as i32)..=thickness as i32 {
            for j in -(thickness as i32)..=thickness as i32 {
                let px = (x + i) as usize;
                let py = (y + j) as usize;
                if px < width && py < height {
                    let idx = (py * width + px) * 4;
                    if idx + 3 < image_data.len() {
                        image_data[idx] = color.0;
                        image_data[idx + 1] = color.1;
                        image_data[idx + 2] = color.2;
                        image_data[idx + 3] = color.3;
                    }
                }
            }
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

pub fn draw_box(
    image_data: &mut [u8],
    width: usize,
    height: usize,
    start: Position,
    end: Position,
    thickness: usize,
    color: (u8, u8, u8, u8),
) {
    // Draw horizontal lines
    draw_line(
        image_data,
        width,
        height,
        start,
        Position { x: end.x, y: start.y },
        thickness,
        color,
    );
    draw_line(
        image_data,
        width,
        height,
        Position { x: start.x, y: end.y },
        end,
        thickness,
        color,
    );

    // Draw vertical lines
    draw_line(
        image_data,
        width,
        height,
        start,
        Position { x: start.x, y: end.y },
        thickness,
        color,
    );
    draw_line(
        image_data,
        width,
        height,
        Position { x: end.x, y: start.y },
        end,
        thickness,
        color,
    );
}

pub fn draw_label(
    image_data: &mut [u8],
    width: usize,
    height: usize,
    text: &str,
    position: Position,
    color: (u8, u8, u8, u8),
    font_data: &[u8],
    font_size: f32,
) -> io::Result<()> {
    let font = FontRef::try_from_slice(font_data)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let scale = ab_glyph::PxScale::from(font_size);
    let scaled_font = font.as_scaled(scale);
    let mut x = position.x as f32;
    let y = position.y as f32;

    for c in text.chars() {
        let glyph = scaled_font.scaled_glyph(c);
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|dx, dy, alpha| {
                let px = (x + dx as f32) as usize;
                let py = (y + dy as f32 + bounds.min.y) as usize;
                if px < width && py < height {
                    let idx = (py * width + px) * 4;
                    if idx + 3 < image_data.len() {
                        let alpha = (alpha * 255.0) as u8;
                        image_data[idx] = ((color.0 as f32 * alpha as f32 / 255.0) as u8).max(image_data[idx]);
                        image_data[idx + 1] = ((color.1 as f32 * alpha as f32 / 255.0) as u8).max(image_data[idx + 1]);
                        image_data[idx + 2] = ((color.2 as f32 * alpha as f32 / 255.0) as u8).max(image_data[idx + 2]);
                        image_data[idx + 3] = alpha.max(image_data[idx + 3]);
                    }
                }
            });
            x += bounds.width();
        }
    }

    Ok(())
}

pub fn send_image_data(image_data: &[u8], width: usize, height: usize) -> io::Result<()> {
    let mut stdout = io::stdout();
    
    if crate::DEBUG {
        eprintln!("Sending image data: {}x{} ({} bytes)", width, height, image_data.len());
    }
    
    // Convert RGBA data to PNG
    let mut png_data = Vec::new();
    {
        let mut encoder = Encoder::new(Cursor::new(&mut png_data), width as u32, height as u32);
        encoder.set_color(ColorType::Rgba);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(image_data)?;
    }
    
    if crate::DEBUG {
        eprintln!("PNG data size: {} bytes", png_data.len());
    }
    
    // Base64 encode the PNG data
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&png_data);
    
    // Send the image data using the kitty graphics protocol with chunking
    let chunk_size = 4096;
    let mut pos = 0;
    let mut is_first = true;
    
    while pos < base64_data.len() {
        // Ensure we don't split in the middle of a base64 character
        let mut end_pos = (pos + chunk_size).min(base64_data.len());
        if end_pos < base64_data.len() {
            // Round down to the nearest multiple of 4
            end_pos = (end_pos / 4) * 4;
        }
        
        let chunk = &base64_data[pos..end_pos];
        
        let cmd = if is_first {
            format!("\x1b_Ga=T,f=100,m={};{}", 
                if end_pos < base64_data.len() { "1" } else { "0" },
                chunk
            )
        } else {
            format!("\x1b_Gm={};{}", 
                if end_pos < base64_data.len() { "1" } else { "0" },
                chunk
            )
        };
        
        // Write the command and flush
        stdout.write_all((cmd + "\x1b\\").as_bytes())?;
        stdout.flush()?;
        
        pos = end_pos;
        is_first = false;
    }
    
    if crate::DEBUG {
        eprintln!("Finished sending image data");
    }
    
    Ok(())
}