use std::io;
use std::io::Write;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BlurParams {
    radius: u32,
    iterations: u32
}

/// # Safety
/// - `rgba_ptr` must be valid for writes of `rgba_len` bytes.
/// - `rgba_len` must equal `width * height * 4`.
/// - `params_ptr` must be valid for reads of `params_len` bytes.
/// - `params_ptr` must point to a valid UTF-8 JSON buffer if JSON parsing expects that.
/// - The memory referenced by `rgba_ptr` and `params_ptr` must remain valid for the duration of the call.
/// - `rgba_ptr` must not alias any other mutable reference.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_ptr: *mut u8,
    rgba_len: usize,
    params_ptr: *const u8,
    params_len: usize) -> i32 {
    println!("Blur image {width} x {height}");

    if !validate_input(width, height, rgba_ptr, rgba_len, params_ptr, params_len) {
        return 1;
    }

    let rgba: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(rgba_ptr, rgba_len) };
    let params_bytes: &[u8] = unsafe { std::slice::from_raw_parts(params_ptr, params_len) };

    let params: BlurParams = match serde_json::from_slice(params_bytes) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid JSON params: {e}");
            return 2;
        }
    };

    println!("Blur params: {params:?}");
    blur_image(rgba, width as usize, height as usize, params.radius, params.iterations);
    0
}

fn validate_input(width: u32,
                  height: u32,
                  rgba_ptr: *mut u8,
                  rgba_len: usize,
                  params_ptr: *const u8,
                  params_len: usize) -> bool {

    if rgba_ptr.is_null() || params_ptr.is_null() || params_len == 0 {
        return false;
    }

    let expected_len = match (width as usize)
        .checked_mul(height as usize)
        .and_then(|v| v.checked_mul(4))
    {
        Some(v) => v,
        None => return false,
    };

    rgba_len == expected_len
}

fn blur_image(rgba: &mut [u8], width: usize, height: usize, radius: u32, iterations: u32) {
    if width == 0 || height == 0 || radius == 0 || iterations == 0 {
        return;
    }

    let radius = radius as isize;
    let iterations = iterations as usize;
    let width_i = width as isize;
    let height_i = height as isize;
    let radius_sq = radius * radius;

    let mut kernel: Vec<(isize, isize, f32)> = Vec::new();

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let dist_sq = dx * dx + dy * dy;
            if dist_sq > radius_sq {
                continue;
            }

            let distance = (dist_sq as f32).sqrt();
            let weight = 1.0 / (1.0 + distance);
            kernel.push((dx, dy, weight));
        }
    }

    let mut src = rgba.to_vec();
    let mut dst = vec![0u8; rgba.len()];

    let total_rows = iterations * height;
    let mut done_rows = 0usize;

    for _ in 0..iterations {
        for y in 0..height {

            let percent = ((done_rows + 1) * 100) / total_rows;
            print!("\rIterations processing...{:>3}%", percent);
            io::stdout().flush().unwrap();

            for x in 0..width {
                let mut sum_r = 0.0f32;
                let mut sum_g = 0.0f32;
                let mut sum_b = 0.0f32;
                let mut sum_a = 0.0f32;
                let mut weight_sum = 0.0f32;

                let x_i = x as isize;
                let y_i = y as isize;

                for &(dx, dy, weight) in &kernel {
                    let nx = x_i + dx;
                    let ny = y_i + dy;

                    if nx < 0 || ny < 0 || nx >= width_i || ny >= height_i {
                        continue;
                    }

                    let idx = (ny as usize * width + nx as usize) * 4;

                    sum_r += src[idx] as f32 * weight;
                    sum_g += src[idx + 1] as f32 * weight;
                    sum_b += src[idx + 2] as f32 * weight;
                    sum_a += src[idx + 3] as f32 * weight;
                    weight_sum += weight;
                }

                let dst_idx = (y * width + x) * 4;
                dst[dst_idx] = (sum_r / weight_sum).round().clamp(0.0, 255.0) as u8;
                dst[dst_idx + 1] = (sum_g / weight_sum).round().clamp(0.0, 255.0) as u8;
                dst[dst_idx + 2] = (sum_b / weight_sum).round().clamp(0.0, 255.0) as u8;
                dst[dst_idx + 3] = (sum_a / weight_sum).round().clamp(0.0, 255.0) as u8;
            }

            done_rows += 1;
        }

        std::mem::swap(&mut src, &mut dst);
    }

    println!();
    rgba.copy_from_slice(&src);
}
