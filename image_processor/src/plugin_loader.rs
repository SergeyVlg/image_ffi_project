use std::ffi::{c_char, CString};
use std::path::PathBuf;
use std::str::FromStr;
use libloading::{Library, Symbol};
use crate::error::ProcessError;
use crate::error::ProcessError::Validation;

/// Processing source image
/// # Safety
/// - `width` and `height` must be greater than 0.
/// - `rgba_ptr` must be valid for writes of `rgba_len` bytes.
/// - `rgba_len` must equal `width * height * 4`.
/// - `params_ptr` must be valid for reads of `params_len` bytes.
/// - `params_ptr` must point to a valid C string with params at JSON structure.
/// - The memory referenced by `rgba_ptr` and `params_ptr` must remain valid for the duration of the call.
/// - `rgba_ptr` must not alias any other mutable reference.
type ProcessImageFn = unsafe extern "C" fn(
    width: u32,
    height: u32,
    rgba_ptr: *mut u8,
    rgba_len: usize,
    params_ptr: *const c_char,
    params_len: usize,
) -> i32;

pub struct Plugin {
    plugin: Library,
}

impl Plugin {
    pub fn new(filename: PathBuf) -> Result<Self, ProcessError> {
        Ok(Plugin {
            plugin: unsafe { Library::new(filename) }?,
        })
    }

    pub fn process_image(&self, width: u32, height: u32, rgba_data: &mut [u8], params: &str) -> Result<(), ProcessError> {
        Self::check_image_params(width, height, rgba_data)?;

        let rgba_ptr = rgba_data.as_mut_ptr();
        let rgba_len = rgba_data.len();

        let c_params = CString::from_str(params)?;
        let params_ptr = c_params.as_ptr();
        let params_len = c_params.as_bytes().len();

        // # SAFETY
        // - `width` and `height` greater than 0.
        // - `rgba_ptr` is valid for writes of `rgba_len` bytes.
        // - `rgba_len` equal `width * height * 4`.
        // - `params_ptr` valid for reads of `params_len` bytes.
        // - `params_ptr` points to a valid C string with params at JSON structure.
        // - The memory referenced by `rgba_ptr` and `params_ptr` remain valid for the duration of the call.
        // - `rgba_ptr` not alias any other mutable reference.
        unsafe {
            let function: Symbol<'_, ProcessImageFn> = self.plugin.get(b"process_image")?;
            let result = function(width, height, rgba_ptr, rgba_len, params_ptr, params_len);

            if result != 0 {
                return Err(ProcessError::ImageProcessing(result));
            }
        }

        Ok(())
    }

    fn check_image_params(width: u32, height: u32, rgba_data: &mut [u8]) -> Result<(), ProcessError> {
        if width == 0 || height == 0 {
            return Err(Validation("width and height must be greater than 0".to_string()));
        }

        let expected_len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|v| v.checked_mul(4))
            .ok_or(Validation("image size overflow".to_string()))?;

        if rgba_data.len() != expected_len {
            return Err(Validation("invalid RGBA buffer length".to_string()));
        }

        Ok(())
    }
}