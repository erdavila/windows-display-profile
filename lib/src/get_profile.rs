use windows_ccd::util::{PathInfoExt as _, from_windows_string};
use windows_ccd::windows::{
    DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ONLY_ACTIVE_PATHS,
    QDC_VIRTUAL_MODE_AWARE,
};
use windows_ccd::{display_config_get_device_info, query_display_config};

use crate::error::Error;
use crate::{Dimensions, Monitor, Profile, Result, VIRTUAL_MODE_AWARE};

/// Gets the current Windows display profile.
pub fn get_profile() -> Result<Profile> {
    let mut flags = QDC_ONLY_ACTIVE_PATHS;
    if VIRTUAL_MODE_AWARE {
        flags |= QDC_VIRTUAL_MODE_AWARE;
    }
    let (paths, modes) = query_display_config(flags)?;

    paths
        .into_iter()
        .map(|path| {
            let source_device_name = display_config_get_device_info::<
                DISPLAYCONFIG_SOURCE_DEVICE_NAME,
            >(path.sourceInfo)?;

            let target_device_name = display_config_get_device_info::<
                DISPLAYCONFIG_TARGET_DEVICE_NAME,
            >(path.targetInfo)?;

            let source_mode = unsafe {
                let Some(idx) = path.source_mode_idx() else {
                    return Err(Error::Custom(
                        "no source mode in the display path info".to_string(),
                    ));
                };
                &modes[idx].Anonymous.sourceMode
            };

            let desktop_mode = unsafe {
                let Some(idx) = path.desktop_mode_idx() else {
                    return Err(Error::Custom(
                        "no desktop mode in the display path info".to_string(),
                    ));
                };
                &modes[idx].Anonymous.desktopImageInfo
            };

            Ok(Monitor {
                friendly_device_name: from_windows_string(
                    &target_device_name.monitorFriendlyDeviceName,
                ),
                source_device_name: from_windows_string(&source_device_name.viewGdiDeviceName),
                device_path: from_windows_string(&target_device_name.monitorDevicePath),
                dimensions: Dimensions {
                    width: source_mode.width,
                    height: source_mode.height,
                },
                pixel_format: source_mode.pixelFormat.into(),
                position: source_mode.position.into(),
                rotation: path.targetInfo.rotation.into(),
                scaling: path.targetInfo.scaling.into(),
                refresh_rate: path.targetInfo.refreshRate.into(),
                path_source_size: desktop_mode.PathSourceSize.into(),
                desktop_image_region: desktop_mode.DesktopImageRegion.into(),
                desktop_image_clip: desktop_mode.DesktopImageClip.into(),
            })
        })
        .collect()
}
