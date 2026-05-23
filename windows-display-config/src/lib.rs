#![expect(clippy::missing_errors_doc)]
#![cfg_attr(all(doc, not(doctest)), feature(doc_cfg))]

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

pub type Result<T, E = error::Error> = std::result::Result<T, E>;

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

pub trait GetDeviceInfo: private::GetDeviceInfoBase {
    const TYPE: DISPLAYCONFIG_DEVICE_INFO_TYPE;

    #[cfg(feature = "dump")]
    fn header(&self) -> &DISPLAYCONFIG_DEVICE_INFO_HEADER;

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
