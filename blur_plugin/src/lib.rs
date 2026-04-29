use std::ffi::c_char;
use std::io;
use std::io::ErrorKind::InvalidData;
use std::io::Write;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BlurParams {
    radius: u32,
    iterations: u32
}

/// # Safety
/// - `width` and `height` must be greater than 0.
/// - `rgba_ptr` must be valid for writes of `rgba_len` bytes.
/// - `rgba_len` must equal `width * height * 4`.
/// - `params_ptr` must be valid for reads of `params_len` bytes.
/// - `params_ptr` must point to a valid C string with params at JSON structure.
/// - The memory referenced by `rgba_ptr` and `params_ptr` must remain valid for the duration of the call.
/// - `rgba_ptr` must not alias any other mutable reference.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_ptr: *mut u8,
    rgba_len: usize,
    params_ptr: *const c_char,
    params_len: usize) -> i32 {
    println!("Blur image {width} x {height}");

    if !validate_input(width, height, rgba_ptr, rgba_len, params_ptr, params_len) {
        return 1;
    }

    let rgba: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(rgba_ptr, rgba_len) };
    let params_bytes = unsafe { std::slice::from_raw_parts(params_ptr as *const u8, params_len) };

    let params: BlurParams = match serde_json::from_slice(params_bytes) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid JSON params: {e}");
            return 2;
        }
    };

    let radius: isize = match params.radius.try_into() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid radius: {e}");
            return 3;
        }
    };

    println!("Blur params: {params:?}");
    if let Err(err) = blur_image(rgba, width as usize, height as usize, radius, params.iterations) {
        eprintln!("Blur image error: {err}");
        return 4;
    }
    0
}

fn validate_input(width: u32,
                  height: u32,
                  rgba_ptr: *mut u8,
                  rgba_len: usize,
                  params_ptr: *const c_char,
                  params_len: usize) -> bool {

    if width == 0 ||
        height == 0 ||
        rgba_ptr.is_null() ||
        params_ptr.is_null() ||
        params_len == 0 {
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

fn blur_image(rgba: &mut [u8], width: usize, height: usize, radius: isize, iterations: u32) -> io::Result<()> {
    if radius == 0 || iterations == 0 {
        return Err(io::Error::new(InvalidData, "radius and iterations must be greater than 0"));
    }

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
            io::stdout().flush()?;

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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Создаем тестовое изображение 3x3 пикселя (черный фон, белый пиксель в центре)
    // 3 x 3 x 4 байта = 36 байт
    fn create_test_image() -> Vec<u8> {
        let mut img = vec![0; 3 * 3 * 4]; // Заполняем нулями (черный цвет, прозрачный)

        // Устанавливаем центральный пиксель (x=1, y=1) в белый непрозрачный цвет
        let center_idx = (1 * 3 + 1) * 4;
        img[center_idx] = 255;     // R
        img[center_idx + 1] = 255; // G
        img[center_idx + 2] = 255; // B
        img[center_idx + 3] = 255; // A

        img
    }

    #[test]
    fn test_blur_zero_radius_or_iterations() {
        let mut img = create_test_image();
        let original = img.clone();

        // Если радиус или итерации равны 0, изображение не должно измениться
        blur_image(&mut img, 3, 3, 0, 1);
        assert_eq!(img, original, "Изображение изменилось при radius=0");

        blur_image(&mut img, 3, 3, 1, 0);
        assert_eq!(img, original, "Изображение изменилось при iterations=0");
    }

    #[test]
    fn test_blur_spreads_pixels() {
        let mut img = create_test_image();
        let center_idx = (1 * 3 + 1) * 4;

        // Применяем размытие
        blur_image(&mut img, 3, 3, 2, 1);

        // После размытия центральный пиксель должен "отдать" часть цвета соседям,
        // поэтому его яркость должна стать меньше 255
        assert!(img[center_idx] < 255, "Центральный пиксель не потускнел");
        assert!(img[center_idx] > 0, "Центральный пиксель стал полностью черным");

        // Соседний пиксель (например, левый верхний: x=0, y=0) должен перестать быть черным
        let top_left_idx = 0;
        assert!(img[top_left_idx] > 0, "Цвет не распространился на соседние пиксели");
    }

    #[test]
    fn test_process_image_success() {
        let mut img = create_test_image();
        let params = c"{\"radius\": 1, \"iterations\": 1}";

        let result = unsafe {
            process_image(
                3, 3,
                img.as_mut_ptr(), img.len(),
                params.as_ptr(), params.count_bytes()
            )
        };

        assert_eq!(result, 0);

        // Убеждаемся, что размытие действительно применилось
        let center_idx = (1 * 3 + 1) * 4;
        assert!(img[center_idx] < 255);
    }

    #[test]
    fn test_process_image_validation_error_dimensions() {
        let mut img = create_test_image();
        let params = c"{\"radius\": 1, \"iterations\": 1}";

        let result = unsafe {
            process_image(
                0, 3,
                img.as_mut_ptr(), img.len(),
                params.as_ptr(), params.count_bytes()
            )
        };

        assert_eq!(result, 1);
    }

    #[test]
    fn test_process_image_validation_error_buffer_len() {
        let mut img = create_test_image();
        let params = c"{\"radius\": 1, \"iterations\": 1}";

        let result = unsafe {
            process_image(
                3, 3,
                img.as_mut_ptr(), 35, // Ожидается 36 (3*3*4), передаем 35
                params.as_ptr(), params.count_bytes()
            )
        };

        assert_eq!(result, 1);
    }

    #[test]
    fn test_process_image_json_error_missing_fields() {
        let mut img = create_test_image();
        let params = c"{\"radius\": 1}";

        let result = unsafe {
            process_image(
                3, 3,
                img.as_mut_ptr(), img.len(),
                params.as_ptr(), params.count_bytes()
            )
        };

        assert_eq!(result, 2);
    }

    #[test]
    fn test_process_image_null_pointer() {
        let params = c"{\"radius\": 1, \"iterations\": 1}";

        let result = unsafe {
            process_image(
                3, 3,
                std::ptr::null_mut(), 36,
                params.as_ptr(), params.count_bytes()
            )
        };

        assert_eq!(result, 1);
    }
}