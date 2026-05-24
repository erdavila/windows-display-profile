use std::fs;

use serde_json::{Value as JsonValue, json};

use crate::util::{
    PathSourceInfoExt as _, PathTargetInfoExt as _, U32Ext as _, from_windows_string,
};
use crate::windows::{
    D3DKMDT_VIDEO_SIGNAL_STANDARD, DISPLAYCONFIG_2DREGION, DISPLAYCONFIG_DESKTOP_IMAGE_INFO,
    DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_DEVICE_INFO_TYPE, DISPLAYCONFIG_MODE_INFO,
    DISPLAYCONFIG_MODE_INFO_TYPE_DESKTOP_IMAGE, DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE,
    DISPLAYCONFIG_MODE_INFO_TYPE_TARGET, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_PATH_SOURCE_INFO,
    DISPLAYCONFIG_PATH_SUPPORT_VIRTUAL_MODE, DISPLAYCONFIG_PATH_TARGET_INFO,
    DISPLAYCONFIG_PIXELFORMAT, DISPLAYCONFIG_RATIONAL, DISPLAYCONFIG_ROTATION,
    DISPLAYCONFIG_SCALING, DISPLAYCONFIG_SCANLINE_ORDERING, DISPLAYCONFIG_SOURCE_DEVICE_NAME,
    DISPLAYCONFIG_SOURCE_MODE, DISPLAYCONFIG_TARGET_DEVICE_NAME, DISPLAYCONFIG_TARGET_MODE,
    DISPLAYCONFIG_TOPOLOGY_ID, DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY, LUID, POINTL, QDC_ALL_PATHS,
    QDC_ONLY_ACTIVE_PATHS, QUERY_DISPLAY_CONFIG_FLAGS, RECTL, SDC_APPLY, SDC_VALIDATE,
    SET_DISPLAY_CONFIG_FLAGS,
};
use crate::{Error, GetDeviceInfo};

macro_rules! conditional_json {
    ({
        $(
            $key:literal: $value:expr $( => if $condition:expr )?
        ),*
        $(,)?
    }) => {
        [
            ("!", None::<JsonValue>),
            $(
                ($key, conditional_json!(@ JsonValue::from($value) $(=> if $condition)?)),
            )*
        ]
        .into_iter()
        .filter_map(|(key, value_opt)| value_opt.map(|value| (key.to_string(), value)))
        .collect::<JsonValue>()
    };

    (@ $value:expr) => { Some($value) };
    (@ $value:expr => if $condition:expr) => { $condition.then(|| $value) };
}

pub(crate) fn dump_query_display_config(
    error: Error,
    flags: QUERY_DISPLAY_CONFIG_FLAGS,
    paths: &[DISPLAYCONFIG_PATH_INFO],
    path_count: u32,
    modes: &[DISPLAYCONFIG_MODE_INFO],
    mode_count: u32,
    current_topology_id: Option<&DISPLAYCONFIG_TOPOLOGY_ID>,
) {
    let value = conditional_json!({
        "error": error.win32_error.to_string()
            => if error.is_err(),
        "flags": flags.to_json_value(),
        "paths": paths[..path_count as usize].to_json_value()
            => if error.is_ok(),
        "modes": modes[..mode_count as usize].to_json_value()
            => if error.is_ok(),
        "current_topology_id": current_topology_id.map(ToJsonValue::to_json_value),
    });

    let argument = if flags.contains(QDC_ALL_PATHS) {
        "ALL_PATHS"
    } else if flags.contains(QDC_ONLY_ACTIVE_PATHS) {
        "ONLY_ACTIVE_PATHS"
    } else {
        "UNKNOWN"
    };

    dump(error.function, argument, &value);
}

pub(crate) fn dump_set_display_config(
    error: Error,
    paths: Option<&[DISPLAYCONFIG_PATH_INFO]>,
    modes: Option<&[DISPLAYCONFIG_MODE_INFO]>,
    flags: SET_DISPLAY_CONFIG_FLAGS,
) {
    let value = conditional_json!({
        "error": error.win32_error.to_string() =>
            if error.is_err(),
        "paths": paths.map(ToJsonValue::to_json_value),
        "modes": modes.map(ToJsonValue::to_json_value),
        "flags": flags.to_json_value(),
    });

    let argument = if flags.contains(SDC_APPLY) {
        "APPLY"
    } else if flags.contains(SDC_VALIDATE) {
        "VALIDATE"
    } else {
        "UNKNOWN"
    };

    dump(error.function, argument, &value);
}

pub(crate) fn dump_display_config_get_device_info<T: GetDeviceInfo>(error: Error, device_info: &T) {
    let value = if error.is_ok() {
        device_info.to_json_value()
    } else {
        json!({
            "error": error.win32_error.to_string(),
            "header": device_info.header().to_json_value(),
        })
    };

    let argument = format!(
        "{}-{}-{}-{}",
        device_info
            .header()
            .r#type
            .to_json_value()
            .as_str()
            .unwrap(),
        device_info.header().adapterId.HighPart,
        device_info.header().adapterId.LowPart,
        device_info.header().id,
    );

    dump(error.function, &argument, &value);
}

fn dump(function: &'static str, argument: &str, value: &JsonValue) {
    let file_name = format!("dump-{function}-{argument}.json");
    eprintln!("Dumping {function} parameters and output to {file_name}");

    let file = match fs::File::create(&file_name) {
        Ok(file) => file,
        Err(err) => {
            eprint!("Could not open file {file_name}: {err}");
            return;
        }
    };

    if let Err(err) = serde_json::to_writer_pretty(file, &value) {
        eprintln!("Could not write to file {file_name}: {err}");
    }
}

macro_rules! with_stringified {
    (
        $( $path:ident )::+ ;
        $( $value:ident , )*
    ) => {
        {
            use $($path)::+ as WinPath;
            [ $( (WinPath::$value, stringify!($value)) ),* ]
        }
    };
}

macro_rules! enum_to_json_value {
    (
        $enum:expr ;
        $( $path:ident )::+ ;
        $strip:literal ;
        $( $value:ident , )*
    ) => {
        {
        use $($path)::+ as WinPath;
        JsonValue::from(match $enum {
            $( WinPath::$value => stringify!($value)[$strip..].to_string(), )*
            value => format!("??? {:#}", value.0),
        })}
    };
}

macro_rules! flags_to_json_value {
    (
        $flags:expr => $type:ident ;
        $( $path:ident )::+ ;
        $strip:literal ;
        $( $value:ident , )*
    ) => {{
        fn eq_default<T: PartialEq + Default>(flags: T) -> bool {
            flags == T::default()
        }

        let mut flags = $flags;
        let mut strs = Vec::new();

        let values_and_strs = with_stringified!(
            $($path)::+;
            $($value,)*
        );

        for (value, str) in values_and_strs {
            if flags.contains(value) {
                strs.push(str[$strip..].to_string());
                flags &= !value;
            }
        }

        if !eq_default(flags) {
            strs.push(format!("??? {:#X}", flags_to_json_value!(@ flags : $type)));
        }

        JsonValue::from(strs)
    }};

    (@ $var:ident : Self) => { $var.0 };
    (@ $var:ident : u32) => { $var };
}

pub trait ToJsonValue {
    fn to_json_value(&self) -> JsonValue;
}

impl<T: ToJsonValue> ToJsonValue for [T] {
    fn to_json_value(&self) -> JsonValue {
        self.iter().map(ToJsonValue::to_json_value).collect()
    }
}

impl ToJsonValue for D3DKMDT_VIDEO_SIGNAL_STANDARD {
    fn to_json_value(&self) -> JsonValue {
        enum_to_json_value!(
            *self;
            windows::Wdk::Graphics::Direct3D;
            12;
            D3DKMDT_VSS_UNINITIALIZED,
            D3DKMDT_VSS_VESA_DMT,
            D3DKMDT_VSS_VESA_GTF,
            D3DKMDT_VSS_VESA_CVT,
            D3DKMDT_VSS_IBM,
            D3DKMDT_VSS_APPLE,
            D3DKMDT_VSS_NTSC_M,
            D3DKMDT_VSS_NTSC_J,
            D3DKMDT_VSS_NTSC_443,
            D3DKMDT_VSS_PAL_B,
            D3DKMDT_VSS_PAL_B1,
            D3DKMDT_VSS_PAL_G,
            D3DKMDT_VSS_PAL_H,
            D3DKMDT_VSS_PAL_I,
            D3DKMDT_VSS_PAL_D,
            D3DKMDT_VSS_PAL_N,
            D3DKMDT_VSS_PAL_NC,
            D3DKMDT_VSS_SECAM_B,
            D3DKMDT_VSS_SECAM_D,
            D3DKMDT_VSS_SECAM_G,
            D3DKMDT_VSS_SECAM_H,
            D3DKMDT_VSS_SECAM_K,
            D3DKMDT_VSS_SECAM_K1,
            D3DKMDT_VSS_SECAM_L,
            D3DKMDT_VSS_SECAM_L1,
            D3DKMDT_VSS_EIA_861,
            D3DKMDT_VSS_EIA_861A,
            D3DKMDT_VSS_EIA_861B,
            D3DKMDT_VSS_PAL_K,
            D3DKMDT_VSS_PAL_K1,
            D3DKMDT_VSS_PAL_L,
            D3DKMDT_VSS_PAL_M,
            D3DKMDT_VSS_OTHER,
        )
    }
}

impl ToJsonValue for DISPLAYCONFIG_2DREGION {
    fn to_json_value(&self) -> JsonValue {
        json!({
            "cx": self.cx,
            "cy": self.cy,
        })
    }
}

impl ToJsonValue for DISPLAYCONFIG_DESKTOP_IMAGE_INFO {
    fn to_json_value(&self) -> JsonValue {
        json!({
            "PathSourceSize": self.PathSourceSize.to_json_value(),
            "DesktopImageRegion": self.DesktopImageRegion.to_json_value(),
            "DesktopImageClip": self.DesktopImageClip.to_json_value(),
        })
    }
}

impl ToJsonValue for DISPLAYCONFIG_DEVICE_INFO_HEADER {
    fn to_json_value(&self) -> JsonValue {
        json!({
            "type": self.r#type.to_json_value(),
            "size": self.size,
            "adapterId": self.adapterId.to_json_value(),
            "id": self.id,
        })
    }
}

impl ToJsonValue for DISPLAYCONFIG_DEVICE_INFO_TYPE {
    fn to_json_value(&self) -> JsonValue {
        enum_to_json_value!(
            *self;
            windows::Win32::Devices::Display;
            26;
            DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
            DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
            DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_PREFERRED_MODE,
            DISPLAYCONFIG_DEVICE_INFO_GET_ADAPTER_NAME,
            DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_BASE_TYPE,
            DISPLAYCONFIG_DEVICE_INFO_GET_SUPPORT_VIRTUAL_RESOLUTION,
            DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO,
            DISPLAYCONFIG_DEVICE_INFO_GET_SDR_WHITE_LEVEL,
            DISPLAYCONFIG_DEVICE_INFO_GET_MONITOR_SPECIALIZATION,
        )
    }
}

impl ToJsonValue for DISPLAYCONFIG_MODE_INFO {
    fn to_json_value(&self) -> JsonValue {
        conditional_json!({
            "id": self.id,
            "adapterId": self.adapterId.to_json_value(),
            "sourceMode": unsafe { self.Anonymous.sourceMode }.to_json_value()
                => if self.infoType == DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE,
            "targetMode": unsafe { self.Anonymous.targetMode }.to_json_value()
                => if self.infoType == DISPLAYCONFIG_MODE_INFO_TYPE_TARGET,
            "desktopImageInfo": unsafe { self.Anonymous.desktopImageInfo }.to_json_value()
                => if self.infoType == DISPLAYCONFIG_MODE_INFO_TYPE_DESKTOP_IMAGE,
        })
    }
}

impl ToJsonValue for DISPLAYCONFIG_PATH_INFO {
    fn to_json_value(&self) -> JsonValue {
        let support_virtual_mode = self.flags.contains(DISPLAYCONFIG_PATH_SUPPORT_VIRTUAL_MODE);

        json!({
            "sourceInfo": (self.sourceInfo, support_virtual_mode).to_json_value(),
            "targetInfo": (self.targetInfo, support_virtual_mode).to_json_value(),
            "flags": path_info_flags_to_json_value(self.flags),
        })
    }
}

impl ToJsonValue for (DISPLAYCONFIG_PATH_SOURCE_INFO, bool) {
    fn to_json_value(&self) -> JsonValue {
        let (source_info, path_support_virtual_mode) = self;

        conditional_json!({
            "adapterId": source_info.adapterId.to_json_value(),
            "id": source_info.id,
            "cloneGroupId": source_info.clone_group_id(*path_support_virtual_mode)
                => if path_support_virtual_mode,
            "sourceModeIdx": source_info.source_mode_idx(*path_support_virtual_mode),
            "statusFlags": path_source_info_status_flags_to_json_value(source_info.statusFlags),
        })
    }
}

impl ToJsonValue for (DISPLAYCONFIG_PATH_TARGET_INFO, bool) {
    fn to_json_value(&self) -> JsonValue {
        let (target_info, path_support_virtual_mode) = self;

        conditional_json!({
            "adapterId": target_info.adapterId.to_json_value(),
            "id": target_info.id,
            "desktopModeIdx": target_info.desktop_mode_idx(*path_support_virtual_mode)
                => if path_support_virtual_mode,
            "targetModeIdx": target_info.target_mode_idx(*path_support_virtual_mode),
            "outputTechnology": target_info.outputTechnology.to_json_value(),
            "rotation": target_info.rotation.to_json_value(),
            "scaling": target_info.scaling.to_json_value(),
            "refreshRate": target_info.refreshRate.to_json_value(),
            "scanLineOrdering": target_info.scanLineOrdering.to_json_value(),
            "targetAvailable": target_info.targetAvailable.as_bool(),
            "statusFlags": path_target_info_status_flags_to_json_value(target_info.statusFlags),
        })
    }
}

impl ToJsonValue for DISPLAYCONFIG_PIXELFORMAT {
    fn to_json_value(&self) -> JsonValue {
        enum_to_json_value!(
            *self;
            windows::Win32::Devices::Display;
            26;
            DISPLAYCONFIG_PIXELFORMAT_8BPP,
            DISPLAYCONFIG_PIXELFORMAT_16BPP,
            DISPLAYCONFIG_PIXELFORMAT_24BPP,
            DISPLAYCONFIG_PIXELFORMAT_32BPP,
            DISPLAYCONFIG_PIXELFORMAT_NONGDI,
        )
    }
}

impl ToJsonValue for DISPLAYCONFIG_RATIONAL {
    fn to_json_value(&self) -> JsonValue {
        json!({
            "Numerator": self.Numerator,
            "Denominator": self.Denominator,
        })
    }
}

impl ToJsonValue for DISPLAYCONFIG_ROTATION {
    fn to_json_value(&self) -> JsonValue {
        enum_to_json_value!(
            *self;
            windows::Win32::Devices::Display;
            23;
            DISPLAYCONFIG_ROTATION_IDENTITY,
            DISPLAYCONFIG_ROTATION_ROTATE90,
            DISPLAYCONFIG_ROTATION_ROTATE180,
            DISPLAYCONFIG_ROTATION_ROTATE270,
        )
    }
}

impl ToJsonValue for DISPLAYCONFIG_SCALING {
    fn to_json_value(&self) -> JsonValue {
        enum_to_json_value!(
            *self;
            windows::Win32::Devices::Display;
            22;
            DISPLAYCONFIG_SCALING_IDENTITY,
            DISPLAYCONFIG_SCALING_CENTERED,
            DISPLAYCONFIG_SCALING_STRETCHED,
            DISPLAYCONFIG_SCALING_ASPECTRATIOCENTEREDMAX,
            DISPLAYCONFIG_SCALING_CUSTOM,
            DISPLAYCONFIG_SCALING_PREFERRED,
        )
    }
}

impl ToJsonValue for DISPLAYCONFIG_SCANLINE_ORDERING {
    fn to_json_value(&self) -> JsonValue {
        enum_to_json_value!(
            *self;
            windows::Win32::Devices::Display;
            32;
            DISPLAYCONFIG_SCANLINE_ORDERING_UNSPECIFIED,
            DISPLAYCONFIG_SCANLINE_ORDERING_PROGRESSIVE,
            // DISPLAYCONFIG_SCANLINE_ORDERING_INTERLACED, // Same as DISPLAYCONFIG_SCANLINE_ORDERING_INTERLACED_UPPERFIELDFIRST.
            DISPLAYCONFIG_SCANLINE_ORDERING_INTERLACED_UPPERFIELDFIRST,
            DISPLAYCONFIG_SCANLINE_ORDERING_INTERLACED_LOWERFIELDFIRST,
        )
    }
}

impl ToJsonValue for DISPLAYCONFIG_SOURCE_DEVICE_NAME {
    fn to_json_value(&self) -> JsonValue {
        json!({
            "header": self.header.to_json_value(),
            "viewGdiDeviceName": from_windows_string(&self.viewGdiDeviceName),
        })
    }
}

impl ToJsonValue for DISPLAYCONFIG_SOURCE_MODE {
    fn to_json_value(&self) -> JsonValue {
        json!({
            "width": self.width,
            "height": self.height,
            "pixelFormat": self.pixelFormat.to_json_value(),
            "position": self.position.to_json_value(),
        })
    }
}

impl ToJsonValue for DISPLAYCONFIG_TARGET_DEVICE_NAME {
    fn to_json_value(&self) -> JsonValue {
        let flags = unsafe { self.flags.Anonymous.value };
        let friendly_name_from_edid = flags.contains(0b0000_0001);
        let friendly_name_forced = flags.contains(0b0000_0010);
        let edid_ids_valid = flags.contains(0b0000_0100);

        json!({
            "header": self.header.to_json_value(),
            "flags": {
                "friendlyNameFromEdid": friendly_name_from_edid,
                "friendlyNameForced": friendly_name_forced,
                "edidIdsValid": edid_ids_valid,
            },
            "outputTechnology": self.outputTechnology.to_json_value(),
            "edidManufactureId": edid_ids_valid.then_some(self.edidManufactureId),
            "edidProductCodeId": edid_ids_valid.then_some(self.edidProductCodeId),
            "connectorInstance": self.connectorInstance,
            "monitorFriendlyDeviceName": from_windows_string(&self.monitorFriendlyDeviceName),
            "monitorDevicePath": from_windows_string(&self.monitorDevicePath),
        })
    }
}

impl ToJsonValue for DISPLAYCONFIG_TARGET_MODE {
    fn to_json_value(&self) -> JsonValue {
        json!({
            "targetVideoSignalInfo": {
                "pixelRate": self.targetVideoSignalInfo.pixelRate,
                "hSyncFreq": self.targetVideoSignalInfo.hSyncFreq.to_json_value(),
                "vSyncFreq": self.targetVideoSignalInfo.vSyncFreq.to_json_value(),
                "activeSize": self.targetVideoSignalInfo.activeSize.to_json_value(),
                "totalSize": self.targetVideoSignalInfo.totalSize.to_json_value(),
                "videoStandard": D3DKMDT_VIDEO_SIGNAL_STANDARD(unsafe { self.targetVideoSignalInfo.Anonymous.videoStandard.cast_signed() }).to_json_value(),
                "scanLineOrdering": self.targetVideoSignalInfo.scanLineOrdering.to_json_value(),
            }
        })
    }
}

impl ToJsonValue for DISPLAYCONFIG_TOPOLOGY_ID {
    fn to_json_value(&self) -> JsonValue {
        enum_to_json_value!(
            *self;
            windows::Win32::Devices::Display;
            23;
            DISPLAYCONFIG_TOPOLOGY_INTERNAL,
            DISPLAYCONFIG_TOPOLOGY_CLONE,
            DISPLAYCONFIG_TOPOLOGY_EXTEND,
            DISPLAYCONFIG_TOPOLOGY_EXTERNAL,
        )
    }
}

impl ToJsonValue for DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY {
    fn to_json_value(&self) -> JsonValue {
        enum_to_json_value!(
            *self;
            windows::Win32::Devices::Display;
            32;
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_OTHER,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HD15,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_SVIDEO,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_COMPOSITE_VIDEO,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_COMPONENT_VIDEO,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DVI,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_HDMI,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_LVDS,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_D_JPN,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_SDI,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EXTERNAL,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_EMBEDDED,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_UDI_EXTERNAL,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_UDI_EMBEDDED,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_SDTVDONGLE,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_MIRACAST,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INDIRECT_WIRED,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INDIRECT_VIRTUAL,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_DISPLAYPORT_USB_TUNNEL,
            DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL,
        )
    }
}

impl ToJsonValue for LUID {
    fn to_json_value(&self) -> JsonValue {
        json!({
            "LowPart": self.LowPart,
            "HighPart": self.HighPart,
        })
    }
}

impl ToJsonValue for POINTL {
    fn to_json_value(&self) -> JsonValue {
        json!({
            "x": self.x,
            "y": self.y,
        })
    }
}

impl ToJsonValue for QUERY_DISPLAY_CONFIG_FLAGS {
    fn to_json_value(&self) -> JsonValue {
        flags_to_json_value!(
            *self => Self;
            windows::Win32::Devices::Display;
            4;
            QDC_ALL_PATHS,
            QDC_ONLY_ACTIVE_PATHS,
            QDC_DATABASE_CURRENT,
            QDC_VIRTUAL_MODE_AWARE,
            QDC_INCLUDE_HMD,
            QDC_VIRTUAL_REFRESH_RATE_AWARE,
        )
    }
}

impl ToJsonValue for RECTL {
    fn to_json_value(&self) -> JsonValue {
        json!({
            "left": self.left,
            "top": self.top,
            "right": self.right,
            "bottom": self.bottom,
        })
    }
}

impl ToJsonValue for SET_DISPLAY_CONFIG_FLAGS {
    fn to_json_value(&self) -> JsonValue {
        flags_to_json_value!(
            *self => Self;
            windows::Win32::Devices::Display;
            4;
            SDC_APPLY,
            SDC_NO_OPTIMIZATION,
            SDC_USE_SUPPLIED_DISPLAY_CONFIG,
            SDC_SAVE_TO_DATABASE,
            SDC_VALIDATE,
            SDC_ALLOW_CHANGES,
            SDC_TOPOLOGY_CLONE,
            SDC_TOPOLOGY_EXTEND,
            SDC_TOPOLOGY_INTERNAL,
            SDC_TOPOLOGY_EXTERNAL,
            SDC_TOPOLOGY_SUPPLIED,
            SDC_USE_DATABASE_CURRENT,
            SDC_PATH_PERSIST_IF_REQUIRED,
            SDC_FORCE_MODE_ENUMERATION,
            SDC_ALLOW_PATH_ORDER_CHANGES,
            SDC_VIRTUAL_MODE_AWARE,
            SDC_VIRTUAL_REFRESH_RATE_AWARE,
        )
    }
}

fn path_info_flags_to_json_value(flags: u32) -> JsonValue {
    flags_to_json_value!(
        flags => u32;
        windows::Win32::Graphics::Gdi;
        19;
        DISPLAYCONFIG_PATH_ACTIVE,
        DISPLAYCONFIG_PATH_SUPPORT_VIRTUAL_MODE,
        // DISPLAYCONFIG_PATH_BOOST_REFRESH_RATE, // Not declared.
    )
}

fn path_source_info_status_flags_to_json_value(flags: u32) -> JsonValue {
    flags_to_json_value!(
        flags => u32;
        windows::Win32::Graphics::Gdi;
        21;
        DISPLAYCONFIG_SOURCE_IN_USE,
    )
}

fn path_target_info_status_flags_to_json_value(flags: u32) -> JsonValue {
    flags_to_json_value!(
        flags => u32;
        windows::Win32::Graphics::Gdi;
        21;
        DISPLAYCONFIG_TARGET_IN_USE,
        DISPLAYCONFIG_TARGET_FORCIBLE,
        DISPLAYCONFIG_TARGET_FORCED_AVAILABILITY_BOOT,
        DISPLAYCONFIG_TARGET_FORCED_AVAILABILITY_PATH,
        DISPLAYCONFIG_TARGET_FORCED_AVAILABILITY_SYSTEM,
        DISPLAYCONFIG_TARGET_IS_HMD,
    )
}
