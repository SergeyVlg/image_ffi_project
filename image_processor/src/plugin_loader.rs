use libloading::{Library, Symbol};
use crate::error::ProcessError;
use crate::error::ProcessError::Validation;

/// Processing source image
/// # Safety
/// - `rgba_ptr` must be valid for writes of `rgba_len` bytes.
/// - `rgba_len` must equal `width * height * 4`.
/// - `params_ptr` must be valid for reads of `params_len` bytes.
/// - `params_ptr` must point to a valid UTF-8 JSON buffer if JSON parsing expects that.
/// - The memory referenced by `rgba_ptr` and `params_ptr` must remain valid for the duration of the call.
/// - `rgba_ptr` must not alias any other mutable reference.
type ProcessImageFn = unsafe extern "C" fn(
    width: u32,
    height: u32,
    rgba_ptr: *mut u8,
    rgba_len: usize,
    params_ptr: *const u8,
    params_len: usize,
);

pub struct Plugin {
    plugin: Library,
}

impl Plugin {
    pub fn new(filename: &str) -> Result<Self, ProcessError> {
        Ok(Plugin {
            plugin: unsafe { Library::new(filename) }?,
        })
    }

    pub fn process_image(&self, width: u32, height: u32, rgba_data: &mut [u8], params: &str) -> Result<(), ProcessError> {
        Self::check_image_params(width, height, rgba_data)?;

        let rgba_ptr = rgba_data.as_mut_ptr();
        let rgba_len = rgba_data.len();

        let params_bytes = params.as_bytes();
        let params_ptr = params_bytes.as_ptr();
        let params_len = params_bytes.len();

        unsafe {
            let function: Symbol<'_, ProcessImageFn> = self.plugin.get(b"process_image")?;
            function(width, height, rgba_ptr, rgba_len, params_ptr, params_len);
        }

        Ok(())
    }

    fn check_image_params(width: u32, height: u32, rgba_data: &mut [u8]) -> Result<(), ProcessError> {
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