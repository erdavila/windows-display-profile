#![expect(clippy::missing_errors_doc)]
#![cfg_attr(all(doc, not(doctest)), feature(doc_cfg))]

pub use error::Error;
pub use get_profile::get_profile;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
pub use set_profile::{SetProfileAction, set_profile};
use windows_ccd::windows::DISPLAYCONFIG_2DREGION;

mod error;
mod get_profile;
mod set_profile;
mod util;

const VIRTUAL_MODE_AWARE: bool = true;

pub type Result<T, E = Error> = std::result::Result<T, E>;

macro_rules! define_windows_mapped_struct {
    (
        $name:ident => $win_name:ident in $($win_path:ident)::+ {
            $(
                $field:ident => $win_field:ident : $field_type:ty ,
            )*
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name {
            $(
                pub $field: $field_type,
            )*
        }

        impl From<$($win_path::)+ $win_name> for $name {
            fn from(value: $($win_path::)+ $win_name) -> Self {
                $name {
                    $(
                        $field: value.$win_field,
                    )*
                }
            }
        }

        impl From<$name> for $($win_path::)+ $win_name {
            fn from(value: $name) -> Self {
                $($win_path::)+ $win_name {
                    $(
                        $win_field: value.$field,
                    )*
                }
            }
        }
    };
}

macro_rules! define_windows_mapped_enum {
    (
        $name:ident => $win_type:ident in $($win_path:ident)::+ {
            $(
                $( #[ $attribute:meta ] )?
                $variant:ident => $win_val:ident ,
            )*
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        pub enum $name {
            $(
                $( #[ $attribute ] )?
                $variant,
            )*
        }

        impl From<$($win_path::)+ $win_type> for $name {
            fn from(value: $($win_path::)+ $win_type) -> Self {
                use $($win_path)::+ as WinPath;
                match value {
                    $( WinPath::$win_val => $name::$variant, )*
                    _ => unreachable!(),
                }
            }
        }

        impl From<$name> for $($win_path::)+ $win_type {
            fn from(value: $name) -> Self {
                use $($win_path)::+ as WinPath;
                match value {
                    $( $name::$variant => WinPath::$win_val, )*
                }
            }
        }
    };
}

pub type Profile = Vec<Monitor>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Monitor {
    pub friendly_device_name: String,
    pub source_device_name: String,
    pub device_path: String,
    pub dimensions: Dimensions,
    pub pixel_format: PixelFormat,
    pub position: Position,
    pub rotation: Rotation,
    pub scaling: Scaling,
    pub refresh_rate: Rational,
    pub path_source_size: Position,
    pub desktop_image_region: Rect,
    pub desktop_image_clip: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}
impl From<Dimensions> for DISPLAYCONFIG_2DREGION {
    fn from(value: Dimensions) -> Self {
        DISPLAYCONFIG_2DREGION {
            cx: value.width,
            cy: value.height,
        }
    }
}

define_windows_mapped_enum!(
    PixelFormat => DISPLAYCONFIG_PIXELFORMAT in windows_ccd::windows {
        #[cfg_attr(feature = "serde", serde(rename = "8BPP"))]
        BitsPerPixel8 => DISPLAYCONFIG_PIXELFORMAT_8BPP,
        #[cfg_attr(feature = "serde", serde(rename = "16BPP"))]
        BitsPerPixel16 => DISPLAYCONFIG_PIXELFORMAT_16BPP,
        #[cfg_attr(feature = "serde", serde(rename = "24BPP"))]
        BitsPerPixel24 => DISPLAYCONFIG_PIXELFORMAT_24BPP,
        #[cfg_attr(feature = "serde", serde(rename = "32BPP"))]
        BitsPerPixel32 => DISPLAYCONFIG_PIXELFORMAT_32BPP,
        NONGDI => DISPLAYCONFIG_PIXELFORMAT_NONGDI,
    }
);

define_windows_mapped_struct!(
    Position => POINTL in windows_ccd::windows {
        x => x: i32,
        y => y: i32,
    }
);

define_windows_mapped_enum!(
    Rotation => DISPLAYCONFIG_ROTATION in windows_ccd::windows {
        IDENTITY => DISPLAYCONFIG_ROTATION_IDENTITY,
        ROTATE90 => DISPLAYCONFIG_ROTATION_ROTATE90,
        ROTATE180 => DISPLAYCONFIG_ROTATION_ROTATE180,
        ROTATE270 => DISPLAYCONFIG_ROTATION_ROTATE270,
    }
);

define_windows_mapped_enum!(
    Scaling => DISPLAYCONFIG_SCALING in windows_ccd::windows {
        IDENTITY => DISPLAYCONFIG_SCALING_IDENTITY,
        CENTERED => DISPLAYCONFIG_SCALING_CENTERED,
        STRETCHED => DISPLAYCONFIG_SCALING_STRETCHED,
        ASPECTRATIOCENTEREDMAX => DISPLAYCONFIG_SCALING_ASPECTRATIOCENTEREDMAX,
        CUSTOM => DISPLAYCONFIG_SCALING_CUSTOM,
        PREFERRED => DISPLAYCONFIG_SCALING_PREFERRED,
    }
);

define_windows_mapped_struct!(
    Rational => DISPLAYCONFIG_RATIONAL in windows_ccd::windows {
        numerator => Numerator: u32,
        denominator => Denominator: u32,
    }
);

define_windows_mapped_struct!(
    Rect => RECTL in windows_ccd::windows {
        left => left: i32,
        top => top: i32,
        right => right: i32,
        bottom => bottom: i32,
    }
);
