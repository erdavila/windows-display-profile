use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use windows_display_config::util::from_windows_string;
pub use windows_display_config::util::{PathInfoExt as _, U32Ext as _};
use windows_display_config::windows::{
    DISPLAYCONFIG_DESKTOP_IMAGE_INFO, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_MODE_INFO_0,
    DISPLAYCONFIG_MODE_INFO_TYPE_DESKTOP_IMAGE, DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE,
    DISPLAYCONFIG_MODE_INFO_TYPE_TARGET, DISPLAYCONFIG_PATH_ACTIVE, DISPLAYCONFIG_PATH_INFO,
    DISPLAYCONFIG_SCANLINE_ORDERING_PROGRESSIVE, DISPLAYCONFIG_SOURCE_DEVICE_NAME,
    DISPLAYCONFIG_SOURCE_MODE, DISPLAYCONFIG_TARGET_DEVICE_NAME, DISPLAYCONFIG_TARGET_MODE,
    DISPLAYCONFIG_VIDEO_SIGNAL_INFO, QDC_ALL_PATHS, QDC_VIRTUAL_MODE_AWARE, SDC_ALLOW_CHANGES,
    SDC_APPLY, SDC_SAVE_TO_DATABASE, SDC_USE_SUPPLIED_DISPLAY_CONFIG, SDC_VALIDATE,
    SDC_VIRTUAL_MODE_AWARE,
};
pub use windows_display_config::{
    DeviceId, GetDeviceInfo, display_config_get_device_info, query_display_config,
    set_display_config,
};

use crate::error::Error;
use crate::util::TryFind as _;
use crate::{Monitor, Profile, Result, VIRTUAL_MODE_AWARE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SetProfileAction {
    Apply,
    Validate,
}

struct DeviceNames {
    cache: BTreeMap<DeviceId, String>,
}
impl DeviceNames {
    fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
        }
    }

    fn source(&mut self, device_id: impl Into<DeviceId>) -> Result<String> {
        self.get::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>(device_id, |dev_name| {
            &dev_name.viewGdiDeviceName
        })
    }

    fn target(&mut self, device_id: impl Into<DeviceId>) -> Result<String> {
        self.get::<DISPLAYCONFIG_TARGET_DEVICE_NAME>(device_id, |dev_name| {
            &dev_name.monitorDevicePath
        })
    }

    fn get<T: GetDeviceInfo>(
        &mut self,
        device_id: impl Into<DeviceId>,
        extract: impl FnOnce(&T) -> &[u16],
    ) -> Result<String> {
        let device_id = device_id.into();

        match self.cache.entry(device_id) {
            Entry::Vacant(entry) => {
                let device_name = display_config_get_device_info::<T>(device_id)?;
                let device_name = from_windows_string(extract(&device_name));
                entry.insert(device_name.clone());
                Ok(device_name)
            }
            Entry::Occupied(entry) => Ok(entry.get().clone()),
        }
    }
}

pub fn set_profile(profile: &Profile, action: SetProfileAction) -> Result<()> {
    let mut flags = QDC_ALL_PATHS;
    if VIRTUAL_MODE_AWARE {
        flags |= QDC_VIRTUAL_MODE_AWARE;
    }
    let (mut input_paths, _input_modes) = query_display_config(flags)?;
    input_paths.retain(|path| path.targetInfo.targetAvailable.as_bool());

    let solved_profile = solve_profile(profile, &input_paths)?;
    let (paths, modes) = make_paths_and_modes(solved_profile);

    let mut flags = SDC_USE_SUPPLIED_DISPLAY_CONFIG | SDC_ALLOW_CHANGES | SDC_SAVE_TO_DATABASE;
    flags |= match action {
        SetProfileAction::Apply => SDC_APPLY,
        SetProfileAction::Validate => SDC_VALIDATE,
    };
    if VIRTUAL_MODE_AWARE {
        flags |= SDC_VIRTUAL_MODE_AWARE;
    }
    set_display_config(Some(&paths), Some(&modes), flags)?;

    Ok(())
}

fn solve_profile<'m, 'p>(
    profile: &'m Profile,
    paths: &'p [DISPLAYCONFIG_PATH_INFO],
) -> Result<Vec<(&'m Monitor, &'p DISPLAYCONFIG_PATH_INFO)>> {
    const ACTIVE_PATH: fn(&DISPLAYCONFIG_PATH_INFO) -> bool =
        |path| path.flags.contains(DISPLAYCONFIG_PATH_ACTIVE);
    const ANY_PATH: fn(&DISPLAYCONFIG_PATH_INFO) -> bool = |_path| true;

    // The order of monitors is preserved throught the processing to keep the path order from when the profile was gotten.
    let mut monitors: Vec<_> = profile.iter().map(|monitor| (monitor, None)).collect();

    let mut device_names = DeviceNames::new();
    let mut source_device_ids_in_use = BTreeSet::new();

    let mut all_solved = false;
    for path_condition in [ACTIVE_PATH, ANY_PATH] {
        all_solved = solve_with_paths(
            &mut monitors,
            paths,
            path_condition,
            &mut device_names,
            &mut source_device_ids_in_use,
        )?;
        if all_solved {
            break;
        }
    }
    if !all_solved {
        return Err(Error::Custom(
            "It was not possible to find paths for all monitors".to_string(),
        ));
    }

    let profile = monitors
        .into_iter()
        .map(|(monitor, path)| (monitor, path.unwrap()))
        .collect();
    Ok(profile)
}

fn solve_with_paths<'p>(
    monitors: &mut [(&Monitor, Option<&'p DISPLAYCONFIG_PATH_INFO>)],
    paths: &'p [DISPLAYCONFIG_PATH_INFO],
    mut path_condition: impl FnMut(&'p DISPLAYCONFIG_PATH_INFO) -> bool,
    device_names: &mut DeviceNames,
    source_device_ids_in_use: &mut BTreeSet<DeviceId>,
) -> Result<bool> {
    let mut all_solved = true;

    let unsolved_monitors = monitors.iter_mut().filter(|(_, path)| path.is_none());

    for (monitor, monitor_path) in unsolved_monitors {
        #[expect(unstable_name_collisions)]
        let active_path = paths.iter().try_find(|path| -> Result<bool> {
            let condition = path_condition(path)
                && !source_device_ids_in_use.contains(&path.sourceInfo.into())
                && device_names.source(path.sourceInfo)? == monitor.source_device_name
                && device_names.target(path.targetInfo)? == monitor.device_path;
            Ok(condition)
        })?;

        if let Some(path) = active_path {
            // Solved.
            *monitor_path = Some(path);
            source_device_ids_in_use.insert(path.sourceInfo.into());
        } else {
            // Still unsolved.
            all_solved = false;
        }
    }

    Ok(all_solved)
}

fn make_paths_and_modes<'m, 'p>(
    solved_profile: impl IntoIterator<Item = (&'m Monitor, &'p DISPLAYCONFIG_PATH_INFO)>,
) -> (Vec<DISPLAYCONFIG_PATH_INFO>, Vec<DISPLAYCONFIG_MODE_INFO>) {
    let mut modes = Vec::new();
    let mut add_mode = |mode: DISPLAYCONFIG_MODE_INFO| {
        let idx = modes.len();
        modes.push(mode);
        idx
    };

    let paths = solved_profile
        .into_iter()
        .map(|(monitor, path)| {
            let mut path = *path;

            path.targetInfo.rotation = monitor.rotation.into();
            path.targetInfo.scaling = monitor.scaling.into();
            path.targetInfo.refreshRate = monitor.refresh_rate.into();

            let source_mode = DISPLAYCONFIG_MODE_INFO {
                infoType: DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE,
                id: path.sourceInfo.id,
                adapterId: path.sourceInfo.adapterId,
                Anonymous: DISPLAYCONFIG_MODE_INFO_0 {
                    sourceMode: DISPLAYCONFIG_SOURCE_MODE {
                        width: monitor.dimensions.width,
                        height: monitor.dimensions.height,
                        pixelFormat: monitor.pixel_format.into(),
                        position: monitor.position.into(),
                    },
                },
            };

            let target_mode = DISPLAYCONFIG_MODE_INFO {
                infoType: DISPLAYCONFIG_MODE_INFO_TYPE_TARGET,
                id: path.targetInfo.id,
                adapterId: path.targetInfo.adapterId,
                Anonymous: DISPLAYCONFIG_MODE_INFO_0 {
                    targetMode: DISPLAYCONFIG_TARGET_MODE {
                        targetVideoSignalInfo: DISPLAYCONFIG_VIDEO_SIGNAL_INFO {
                            vSyncFreq: monitor.refresh_rate.into(),
                            activeSize: monitor.dimensions.clone().into(),
                            scanLineOrdering: DISPLAYCONFIG_SCANLINE_ORDERING_PROGRESSIVE,
                            ..Default::default()
                        },
                    },
                },
            };

            let desktop_mode = DISPLAYCONFIG_MODE_INFO {
                infoType: DISPLAYCONFIG_MODE_INFO_TYPE_DESKTOP_IMAGE,
                id: path.targetInfo.id,
                adapterId: path.targetInfo.adapterId,
                Anonymous: DISPLAYCONFIG_MODE_INFO_0 {
                    desktopImageInfo: DISPLAYCONFIG_DESKTOP_IMAGE_INFO {
                        PathSourceSize: monitor.path_source_size.into(),
                        DesktopImageRegion: monitor.desktop_image_region.into(),
                        DesktopImageClip: monitor.desktop_image_clip.into(),
                    },
                },
            };

            let source_mode_idx = add_mode(source_mode);
            let target_mode_idx = add_mode(target_mode);
            let desktop_mode_idx = add_mode(desktop_mode);

            path.set_source_mode_idx(Some(source_mode_idx));
            path.set_target_mode_idx(Some(target_mode_idx));
            path.set_desktop_mode_idx(Some(desktop_mode_idx));

            path.flags |= DISPLAYCONFIG_PATH_ACTIVE;
            path
        })
        .collect();

    (paths, modes)
}
