use std::ffi::c_char;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MirrorParams {
    horizontal: bool,
    vertical: bool
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
    println!("Mirroring image {width} x {height}");

    if !validate_input(width, height, rgba_ptr, rgba_len, params_ptr, params_len) {
        return 1;
    }

    let rgba: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(rgba_ptr, rgba_len) };
    let params_bytes = unsafe { std::slice::from_raw_parts(params_ptr as *const u8, params_len) };

    let params: MirrorParams = match serde_json::from_slice(params_bytes) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid JSON params: {e}");
            return 2;
        }
    };

    if params.horizontal {
        horizontal_mirror(rgba, width as usize);
    }

    if params.vertical {
        vertical_mirror(rgba, width as usize, height as usize);
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

fn horizontal_mirror(rgba: &mut [u8], width: usize) {
    let row_len = width * 4;

    for row in rgba.chunks_exact_mut(row_len) {
        for x in 0..(width / 2) {
            let left = x * 4;
            let right = (width - 1 - x) * 4;

            for i in 0..4 {
                row.swap(left + i, right + i);
            }
        }
    }
}

fn vertical_mirror(rgba: &mut [u8], width: usize, height: usize) {
    let row_len = width * 4;

    for y in 0..(height / 2) {
        let top_start = y * row_len;
        let bottom_start = (height - 1 - y) * row_len;

        let (head, tail) = rgba.split_at_mut(bottom_start);
        let top_row = &mut head[top_start..top_start + row_len];
        let bottom_row = &mut tail[..row_len];

        top_row.swap_with_slice(bottom_row);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Каждый пиксель — это 4 байта (RGBA)
    // Строка 0: [Пиксель A, Пиксель B]
    // Строка 1: [Пиксель C, Пиксель D]
    fn create_test_image() -> Vec<u8> {
        vec![
            1, 2, 3, 4,       5, 6, 7, 8,
            9, 10, 11, 12,    13, 14, 15, 16,
        ]
    }

    #[test]
    fn test_horizontal_mirror_logic() {
        let mut img = create_test_image();
        horizontal_mirror(&mut img, 2);

        let expected = vec![
            5, 6, 7, 8,       1, 2, 3, 4,
            13, 14, 15, 16,   9, 10, 11, 12,
        ];
        assert_eq!(img, expected);
    }

    #[test]
    fn test_vertical_mirror_logic() {
        let mut img = create_test_image();
        vertical_mirror(&mut img, 2, 2);

        let expected = vec![
            9, 10, 11, 12,    13, 14, 15, 16,
            1, 2, 3, 4,       5, 6, 7, 8,
        ];
        assert_eq!(img, expected);
    }

    #[test]
    fn test_process_image_success() {
        let mut img = create_test_image();
        let params = c"{\"horizontal\": true, \"vertical\": true}";

        let result = unsafe {
            process_image(
                2, 2,
                img.as_mut_ptr(), img.len(),
                params.as_ptr(), params.count_bytes()
            )
        };

        assert_eq!(result, 0);

        let expected = vec![
            13, 14, 15, 16,   9, 10, 11, 12,
            5, 6, 7, 8,       1, 2, 3, 4,
        ];
        assert_eq!(img, expected);
    }

    #[test]
    fn test_process_image_validation_error_len() {
        let mut img = create_test_image();
        let params = c"{\"horizontal\": true, \"vertical\": false}";

        let result = unsafe {
            process_image(
                2, 2,
                img.as_mut_ptr(), 15, // Намеренно передаем неверную длину (15 вместо 16)
                params.as_ptr(), params.count_bytes()
            )
        };

        assert_eq!(result, 1);
    }

    #[test]
    fn test_process_image_json_error() {
        let mut img = create_test_image();
        let params = c"{\"horizontal\": true, \"vertical\": typo}"; // Ошибка синтаксиса JSON

        let result = unsafe {
            process_image(
                2, 2,
                img.as_mut_ptr(), img.len(),
                params.as_ptr(), params.count_bytes()
            )
        };

        assert_eq!(result, 2);
    }

    #[test]
    fn test_process_image_null_pointer() {
        let params = c"{\"horizontal\": true, \"vertical\": true}";

        let result = unsafe {
            process_image(
                2, 2,
                std::ptr::null_mut(), 16, // Передаем нулевой указатель
                params.as_ptr(), params.count_bytes()
            )
        };

        assert_eq!(result, 1);
    }

    #[test]
    fn test_process_image_zero_dimensions() {
        let mut img: Vec<u8> = vec![];
        let params = c"{\"horizontal\": true, \"vertical\": true}";

        let result = unsafe {
            process_image(
                0, 0, // Ширина и высота равны 0
                img.as_mut_ptr(), 0,
                params.as_ptr(), params.count_bytes()
            )
        };

        assert_eq!(result, 1);
    }
}