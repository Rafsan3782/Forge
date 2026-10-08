use libc::{c_int, wchar_t};

#[repr(C)]
pub struct Find_Result {
    pub windows_sdk_version: c_int,
    pub windows_sdk_root: *mut wchar_t,
    pub windows_sdk_um_library_path: *mut wchar_t,
    pub windows_sdk_ucrt_library_path: *mut wchar_t,
    pub vs_exe_path: *mut wchar_t,
    pub vs_library_path: *mut wchar_t,
}

#[no_mangle]
pub unsafe extern "C" fn vswhom_find_visual_studio_and_windows_sdk() -> Find_Result {
    Find_Result {
        windows_sdk_version: 0,
        windows_sdk_root: std::ptr::null_mut(),
        windows_sdk_um_library_path: std::ptr::null_mut(),
        windows_sdk_ucrt_library_path: std::ptr::null_mut(),
        vs_exe_path: std::ptr::null_mut(),
        vs_library_path: std::ptr::null_mut(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn vswhom_free_resources(_result: *mut Find_Result) {}
