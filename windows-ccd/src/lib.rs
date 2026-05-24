#![cfg_attr(all(doc, not(doctest)), feature(doc_cfg))]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use std::mem;

use ::windows::Win32::Devices::Display::{
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_DEVICE_INFO_TYPE, DISPLAYCONFIG_MODE_INFO,
    DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME,
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QUERY_DISPLAY_CONFIG_FLAGS,
    QueryDisplayConfig, SET_DISPLAY_CONFIG_FLAGS, SetDisplayConfig,
};
use ::windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;

pub use crate::device_id::DeviceId;
pub use crate::error::Error;

mod device_id;
#[cfg(feature = "dump")]
mod dump;
mod error;
pub mod util;
pub mod windows;

/// Result type for this crate.
pub type Result<T, E = error::Error> = std::result::Result<T, E>;

/// Convenience function for the [QueryDisplayConfig] function.
///
/// The call to [GetDisplayConfigBufferSizes] and the sizing of the paths and modes buffers are automatically done.
///
/// # Errors
/// Returns an error when [QueryDisplayConfig] fails.
///
/// [QueryDisplayConfig]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig
/// [GetDisplayConfigBufferSizes]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdisplayconfigbuffersizes
///
// TODO: currenttopologyid parameter.
pub fn query_display_config(
    flags: QUERY_DISPLAY_CONFIG_FLAGS,
) -> Result<(Vec<DISPLAYCONFIG_PATH_INFO>, Vec<DISPLAYCONFIG_MODE_INFO>)> {
    loop {
        let mut path_count = 0;
        let mut mode_count = 0;

        let result =
            unsafe { GetDisplayConfigBufferSizes(flags, &raw mut path_count, &raw mut mode_count) };
        Error::new(result, "GetDisplayConfigBufferSizes").to_result(())?;

        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];

        let result = unsafe {
            QueryDisplayConfig(
                flags,
                &raw mut path_count,
                paths.as_mut_ptr(),
                &raw mut mode_count,
                modes.as_mut_ptr(),
                None,
            )
        };

        if result != ERROR_INSUFFICIENT_BUFFER {
            let error = Error::new(result, "QueryDisplayConfig");

            #[cfg(feature = "dump")]
            crate::dump::dump_query_display_config(
                error, flags, &paths, path_count, &modes, mode_count, None,
            );

            return error.to_result_with(|| {
                paths.truncate(path_count as usize);
                modes.truncate(mode_count as usize);
                (paths, modes)
            });
        }

        // Try again.
    }
}

/// Convenience function for the [SetDisplayConfig] function.
///
/// # Errors
/// Returns an error when [SetDisplayConfig] fails.
///
/// [SetDisplayConfig]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setdisplayconfig
pub fn set_display_config(
    paths: Option<&[DISPLAYCONFIG_PATH_INFO]>,
    modes: Option<&[DISPLAYCONFIG_MODE_INFO]>,
    flags: SET_DISPLAY_CONFIG_FLAGS,
) -> Result<()> {
    let result = unsafe { SetDisplayConfig(paths, modes, flags) };
    let error = Error::new(result, "SetDisplayConfig");

    #[cfg(feature = "dump")]
    crate::dump::dump_set_display_config(error, paths, modes, flags);

    error.to_result(())
}

mod private {
    cfg_select! {
        feature = "dump" => {
            pub trait GetDeviceInfoBase: Default + crate::dump::ToJsonValue {}
            impl<T: Default + crate::dump::ToJsonValue> GetDeviceInfoBase for T {}
        }
        _ => {
            pub trait GetDeviceInfoBase: Default {}
            impl<T: Default> GetDeviceInfoBase for T {}
        }
    }
}

/// A trait for the structures that can be returned by the [`display_config_get_device_info`] function.
pub trait GetDeviceInfo: private::GetDeviceInfoBase {
    /// The type of the structure.
    const TYPE: DISPLAYCONFIG_DEVICE_INFO_TYPE;

    /// A reference to the [`DISPLAYCONFIG_DEVICE_INFO_HEADER`].
    #[cfg(feature = "dump")]
    fn header(&self) -> &DISPLAYCONFIG_DEVICE_INFO_HEADER;

    /// A mutable reference to the [`DISPLAYCONFIG_DEVICE_INFO_HEADER`].
    fn header_mut(&mut self) -> &mut DISPLAYCONFIG_DEVICE_INFO_HEADER;
}

impl GetDeviceInfo for DISPLAYCONFIG_SOURCE_DEVICE_NAME {
    const TYPE: DISPLAYCONFIG_DEVICE_INFO_TYPE = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;

    #[cfg(feature = "dump")]
    fn header(&self) -> &DISPLAYCONFIG_DEVICE_INFO_HEADER {
        &self.header
    }

    fn header_mut(&mut self) -> &mut DISPLAYCONFIG_DEVICE_INFO_HEADER {
        &mut self.header
    }
}

impl GetDeviceInfo for DISPLAYCONFIG_TARGET_DEVICE_NAME {
    const TYPE: DISPLAYCONFIG_DEVICE_INFO_TYPE = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;

    #[cfg(feature = "dump")]
    fn header(&self) -> &DISPLAYCONFIG_DEVICE_INFO_HEADER {
        &self.header
    }

    fn header_mut(&mut self) -> &mut DISPLAYCONFIG_DEVICE_INFO_HEADER {
        &mut self.header
    }
}

/// Convenience function for the [DisplayConfigGetDeviceInfo] function.
///
/// The [`DISPLAYCONFIG_DEVICE_INFO_HEADER`] set up is done automatically based on the structure you want to obtain.
///
/// Also, the `adapterId` and `id` are automatically obtained from the [`DISPLAYCONFIG_PATH_SOURCE_INFO`].
///
/// # Example
///
/// To obtain a [`DISPLAYCONFIG_SOURCE_DEVICE_NAME`]:
///
/// ```
/// use windows_ccd::{display_config_get_device_info, DeviceId, Result};
/// use windows_ccd::windows::{DISPLAYCONFIG_PATH_SOURCE_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME};
///
/// fn get_device_name(source_info: DISPLAYCONFIG_PATH_SOURCE_INFO) -> Result<DISPLAYCONFIG_SOURCE_DEVICE_NAME> {
///     display_config_get_device_info(source_info)
/// }
/// ```
///
/// # Errors
/// Returns an error when [DisplayConfigGetDeviceInfo] fails.
///
/// [`DISPLAYCONFIG_PATH_SOURCE_INFO`]: crate::windows::DISPLAYCONFIG_PATH_SOURCE_INFO
/// [DisplayConfigGetDeviceInfo]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-displayconfiggetdeviceinfo
pub fn display_config_get_device_info<T: GetDeviceInfo>(
    device_id: impl Into<DeviceId>,
) -> Result<T> {
    let device_id = device_id.into();

    let mut device_info = T::default();
    *device_info.header_mut() = DISPLAYCONFIG_DEVICE_INFO_HEADER {
        r#type: T::TYPE,
        #[expect(clippy::cast_possible_truncation)]
        size: mem::size_of::<T>() as u32,
        adapterId: device_id.adapter_id,
        id: device_id.id,
    };

    let result = unsafe { DisplayConfigGetDeviceInfo(device_info.header_mut()) };
    let error = Error::new(result, "DisplayConfigGetDeviceInfo");

    #[cfg(feature = "dump")]
    crate::dump::dump_display_config_get_device_info(error, &device_info);

    error.to_result(device_info)
}
