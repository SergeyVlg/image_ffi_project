use std::ffi::c_char;
use libloading::{Library, Symbol};

pub struct PluginInterface<'a> {
    pub process_image: Symbol<'a, unsafe extern "C" fn(width: u32, height: u32, rgba_data: *mut u8, params: *const c_char)>
}

pub struct Plugin {
    plugin: Library,
}

impl Plugin {
    pub fn new(filename: &str) -> Result<Self, libloading::Error> {
        Ok(Plugin {
            plugin: unsafe { Library::new(filename) }?,
        })
    }
    pub fn interface(&self) -> Result<PluginInterface<'_>, libloading::Error> {
        Ok(PluginInterface {
            process_image: unsafe { self.plugin.get("process_image") }?,
        })
    }
}