use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MirrorParams {
    horizontal: bool,
    vertical: bool
}

#[unsafe(no_mangle)]
extern "C" fn process_image(
    width: u32,
    height: u32,
    rgba_ptr: *mut u8,
    rgba_len: usize,
    params_ptr: *const u8,
    params_len: usize) {
    println!("Mirroring image {width} x {height}");

    if !validate_input(width, height, rgba_ptr, rgba_len, params_ptr, params_len) {
        return;
    }

    let rgba: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(rgba_ptr, rgba_len) };
    let params_bytes: &[u8] = unsafe { std::slice::from_raw_parts(params_ptr, params_len) };

    let params: MirrorParams = match serde_json::from_slice(params_bytes) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid JSON params: {e}");
            return;
        }
    };

    if params.horizontal {
        horizontal_mirror(rgba, width as usize);
    }

    if params.vertical {
        vertical_mirror(rgba, width as usize, height as usize);
    }
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

    let expected_len = width as usize * height as usize * 4;

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

        for i in 0..row_len {
            rgba.swap(top_start + i, bottom_start + i);
        }
    }
}