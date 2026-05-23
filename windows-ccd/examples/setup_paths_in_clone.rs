#![expect(clippy::nonminimal_bool)]

use anyhow::{Result, bail};
use windows_ccd::util::{PathInfoExt, U32Ext as _, from_windows_string};
use windows_ccd::windows::{
    DISPLAYCONFIG_PATH_ACTIVE, DISPLAYCONFIG_PATH_SUPPORT_VIRTUAL_MODE,
    DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ONLY_ACTIVE_PATHS, QDC_VIRTUAL_MODE_AWARE,
    SDC_ALLOW_CHANGES, SDC_APPLY, SDC_USE_SUPPLIED_DISPLAY_CONFIG, SDC_VALIDATE,
    SDC_VIRTUAL_MODE_AWARE,
};
use windows_ccd::{
    DeviceId, display_config_get_device_info, query_display_config, set_display_config,
};

const VIRTUAL_MODE_AWARE: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Apply,
    Validate,
}

// Example converted to Rust from https://learn.microsoft.com/en-us/windows-hardware/drivers/display/ccd-example-code.
fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);

    let action = match args.next().as_deref() {
        Some("apply") => Action::Apply,
        Some("validate") => Action::Validate,
        Some(arg) => bail!("What do you mean with {arg:?}?"),
        None => bail!("apply or validate?"),
    };

    if let Some(arg) = args.next() {
        bail!("What do you mean with {arg:?}?");
    }

    setup_paths_in_clone(action)
}

fn get_monitor_friendly_name(device_id: DeviceId) -> Result<String> {
    let device_name =
        display_config_get_device_info::<DISPLAYCONFIG_TARGET_DEVICE_NAME>(device_id)?;
    Ok(from_windows_string(&device_name.monitorFriendlyDeviceName))
}

fn setup_paths_in_clone(action: Action) -> Result<()> {
    // Obtain the path and mode information for all possible paths
    let mut flags = QDC_ONLY_ACTIVE_PATHS;
    if VIRTUAL_MODE_AWARE {
        flags |= QDC_VIRTUAL_MODE_AWARE;
    }
    let (mut paths, mut modes) = query_display_config(flags)?;

    // Find the primary path by searching for an active path that is located at desktop position (0, 0)
    let primary_path_index = paths.iter().position(|path| {
        path.flags.contains(DISPLAYCONFIG_PATH_ACTIVE) && {
            let source_mode_idx = path.source_mode_idx().unwrap();
            let source_mode = unsafe { modes[source_mode_idx].Anonymous.sourceMode };
            source_mode.position.x == 0 && source_mode.position.y == 0
        }
    });

    let Some(primary_path_index) = primary_path_index else {
        bail!("Primary path not found");
    };

    // Determine the user-friendly name of the primary monitor
    let primary_monitor_name =
        get_monitor_friendly_name(paths[primary_path_index].targetInfo.into())?;
    println!("Primary monitor: {primary_monitor_name}");

    // T0D0: Pick which monitors to clone
    // For this sample, we pick the first active monitor other than the primary
    let new_clone_path_index = paths.iter().position(|path| {
        !std::ptr::eq(path, &raw const paths[primary_path_index])
            && path.flags.contains(DISPLAYCONFIG_PATH_ACTIVE)
    });

    let Some(new_clone_path_index) = new_clone_path_index else {
        bail!("No suitable path found for cloning");
    };

    // Determine the user-friendly name of the clone monitor
    let new_clone_monitor_name =
        get_monitor_friendly_name(paths[new_clone_path_index].targetInfo.into())?;
    println!("Will clone with monitor: {new_clone_monitor_name}");

    let [primary_path, new_clone_path] =
        paths.get_disjoint_mut([primary_path_index, new_clone_path_index])?;

    // If the paths don't have the same support for virtual topology, we can't clone them together
    if primary_path
        .flags
        .contains(DISPLAYCONFIG_PATH_SUPPORT_VIRTUAL_MODE)
        != new_clone_path
            .flags
            .contains(DISPLAYCONFIG_PATH_SUPPORT_VIRTUAL_MODE)
    {
        bail!("Primary and clone paths do not have the same support for virtual topology");
    }

    // How to apply clone depends on whether the paths support virtual modes
    if primary_path
        .flags
        .contains(DISPLAYCONFIG_PATH_SUPPORT_VIRTUAL_MODE)
    {
        // In virtual clone, there are two possible options to setup clone depending on
        // whether we want to specify the resolution for the cloned paths

        let should_clone_without_specifying_mode = !false;

        if should_clone_without_specifying_mode {
            // Pick an arbitrary clone group ID to use to associate the paths
            // (anything that isn't DISPLAYCONFIG_PATH_CLONE_GROUP_INVALID).
            // QueryDisplayConfig will not return existing clone group IDs since it returns
            // the paths with their full source modes, so we can just use a constant value.
            let clone_group_id = 1;

            primary_path.set_source_mode_idx(None);
            primary_path.set_desktop_mode_idx(None);
            primary_path.set_target_mode_idx(None);
            primary_path.set_clone_group_id(Some(clone_group_id));

            new_clone_path.set_source_mode_idx(None);
            new_clone_path.set_desktop_mode_idx(None);
            new_clone_path.set_target_mode_idx(None);
            new_clone_path.set_clone_group_id(Some(clone_group_id));
        } else {
            // Alternatively, we can use the same source mode for all the cloned paths if we want to specifically control
            // the resolution (in this case we are just copying the source mode info from the primary path to the newly
            // selected path which takes the resolution, position, and pixel format). They are implicitly cloned because
            // they share the same position.

            let primary_path_source_mode_idx = primary_path.source_mode_idx().unwrap();
            let new_path_source_mode_idx = new_clone_path.source_mode_idx().unwrap();

            let [primary_path_source_mode, new_path_source_mode] = modes
                .get_disjoint_mut([primary_path_source_mode_idx, new_path_source_mode_idx])?
                .map(|mode| unsafe { &mut mode.Anonymous.sourceMode });

            *new_path_source_mode = *primary_path_source_mode;

            // We should also clear the desktop mode info since we are adjusting the source mode on the newly cloned path and
            // it may need to be recalculated
            new_clone_path.set_desktop_mode_idx(None);
        }
    } else {
        // Since the paths don't support virtual clone, we need to check if they are on the same adapter to support hardware clone
        if primary_path.sourceInfo.adapterId != new_clone_path.sourceInfo.adapterId {
            bail!("Primary and clone paths are not on the same adapter");
        }

        // For hardware clone, we simply assign the same source ID to both paths and clear all the
        // mode information from the second path since the hardware may not support the same mode
        new_clone_path.sourceInfo.id = primary_path.sourceInfo.id;

        primary_path.set_target_mode_idx(None);
        primary_path.set_desktop_mode_idx(None);

        new_clone_path.set_source_mode_idx(None);
        new_clone_path.set_target_mode_idx(None);
        new_clone_path.set_desktop_mode_idx(None);
    }

    // Apply the topology changes temporarily. If we want to persist the changes we should also set SDC_SAVE_TO_DATABASE.
    let mut flags = match action {
        Action::Apply => SDC_APPLY,
        Action::Validate => SDC_VALIDATE,
    };
    flags |= SDC_USE_SUPPLIED_DISPLAY_CONFIG | SDC_ALLOW_CHANGES;
    if VIRTUAL_MODE_AWARE {
        flags |= SDC_VIRTUAL_MODE_AWARE;
    }
    set_display_config(Some(&paths), Some(&modes), flags)?;

    Ok(())
}
