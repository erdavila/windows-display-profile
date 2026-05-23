use windows::Win32::Devices::Display::{
    DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_PATH_SOURCE_INFO, DISPLAYCONFIG_PATH_TARGET_INFO,
};
use windows::Win32::Graphics::Gdi::{
    DISPLAYCONFIG_PATH_CLONE_GROUP_INVALID, DISPLAYCONFIG_PATH_DESKTOP_IMAGE_IDX_INVALID,
    DISPLAYCONFIG_PATH_MODE_IDX_INVALID, DISPLAYCONFIG_PATH_SOURCE_MODE_IDX_INVALID,
    DISPLAYCONFIG_PATH_SUPPORT_VIRTUAL_MODE, DISPLAYCONFIG_PATH_TARGET_MODE_IDX_INVALID,
};

pub trait PathInfoExt {
    fn support_virtual_mode(&self) -> bool;

    fn clone_group_id(&self) -> Option<usize>;
    fn source_mode_idx(&self) -> Option<usize>;
    fn target_mode_idx(&self) -> Option<usize>;
    fn desktop_mode_idx(&self) -> Option<usize>;

    fn set_clone_group_id(&mut self, id: Option<usize>);
    fn set_source_mode_idx(&mut self, idx: Option<usize>);
    fn set_target_mode_idx(&mut self, idx: Option<usize>);
    fn set_desktop_mode_idx(&mut self, idx: Option<usize>);
}

impl PathInfoExt for DISPLAYCONFIG_PATH_INFO {
    fn support_virtual_mode(&self) -> bool {
        self.flags.contains(DISPLAYCONFIG_PATH_SUPPORT_VIRTUAL_MODE)
    }

    fn clone_group_id(&self) -> Option<usize> {
        self.sourceInfo.clone_group_id(self.support_virtual_mode())
    }

    fn source_mode_idx(&self) -> Option<usize> {
        self.sourceInfo.source_mode_idx(self.support_virtual_mode())
    }

    fn target_mode_idx(&self) -> Option<usize> {
        self.targetInfo.target_mode_idx(self.support_virtual_mode())
    }

    fn desktop_mode_idx(&self) -> Option<usize> {
        self.targetInfo
            .desktop_mode_idx(self.support_virtual_mode())
    }

    fn set_clone_group_id(&mut self, id: Option<usize>) {
        self.sourceInfo
            .set_clone_group_id(id, self.support_virtual_mode());
    }

    fn set_source_mode_idx(&mut self, idx: Option<usize>) {
        self.sourceInfo
            .set_source_mode_idx(idx, self.support_virtual_mode());
    }

    fn set_target_mode_idx(&mut self, idx: Option<usize>) {
        self.targetInfo
            .set_target_mode_idx(idx, self.support_virtual_mode());
    }

    fn set_desktop_mode_idx(&mut self, idx: Option<usize>) {
        self.targetInfo
            .set_desktop_mode_idx(idx, self.support_virtual_mode());
    }
}

pub trait PathSourceInfoExt {
    fn source_mode_idx(&self, path_support_virtual_mode: bool) -> Option<usize>;
    fn clone_group_id(&self, path_support_virtual_mode: bool) -> Option<usize>;

    fn set_source_mode_idx(&mut self, idx: Option<usize>, path_support_virtual_mode: bool);
    fn set_clone_group_id(&mut self, id: Option<usize>, path_support_virtual_mode: bool);
}

impl PathSourceInfoExt for DISPLAYCONFIG_PATH_SOURCE_INFO {
    fn source_mode_idx(&self, path_support_virtual_mode: bool) -> Option<usize> {
        self.get_value(path_support_virtual_mode, PathInfoUnionValueKind::Primary)
    }

    fn clone_group_id(&self, path_support_virtual_mode: bool) -> Option<usize> {
        self.get_value(path_support_virtual_mode, PathInfoUnionValueKind::Secondary)
    }

    fn set_source_mode_idx(&mut self, idx: Option<usize>, path_support_virtual_mode: bool) {
        self.set_value(
            path_support_virtual_mode,
            PathInfoUnionValueKind::Primary,
            idx,
        );
    }

    fn set_clone_group_id(&mut self, id: Option<usize>, path_support_virtual_mode: bool) {
        self.set_value(
            path_support_virtual_mode,
            PathInfoUnionValueKind::Secondary,
            id,
        );
    }
}

pub trait PathTargetInfoExt {
    fn target_mode_idx(&self, path_support_virtual_mode: bool) -> Option<usize>;
    fn desktop_mode_idx(&self, path_support_virtual_mode: bool) -> Option<usize>;

    fn set_target_mode_idx(&mut self, idx: Option<usize>, path_support_virtual_mode: bool);
    fn set_desktop_mode_idx(&mut self, idx: Option<usize>, path_support_virtual_mode: bool);
}

impl PathTargetInfoExt for DISPLAYCONFIG_PATH_TARGET_INFO {
    fn target_mode_idx(&self, path_support_virtual_mode: bool) -> Option<usize> {
        self.get_value(path_support_virtual_mode, PathInfoUnionValueKind::Primary)
    }

    fn desktop_mode_idx(&self, path_support_virtual_mode: bool) -> Option<usize> {
        self.get_value(path_support_virtual_mode, PathInfoUnionValueKind::Secondary)
    }

    fn set_target_mode_idx(&mut self, idx: Option<usize>, path_support_virtual_mode: bool) {
        self.set_value(
            path_support_virtual_mode,
            PathInfoUnionValueKind::Primary,
            idx,
        );
    }

    fn set_desktop_mode_idx(&mut self, idx: Option<usize>, path_support_virtual_mode: bool) {
        self.set_value(
            path_support_virtual_mode,
            PathInfoUnionValueKind::Secondary,
            idx,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathInfoUnionValueKind {
    Primary,
    Secondary,
}

trait PathInfoUnionValuesAccessor {
    const INVALID_PRIMARY: u32;
    const INVALID_SECONDARY: u32;

    const SECONDARY_DESCRIPTION: &'static str;

    fn mode_info_idx(&self) -> u32;
    fn mode_info_idx_mut(&mut self) -> &mut u32;

    fn bitfield(&self) -> u32;
    fn bitfield_mut(&mut self) -> &mut u32;

    fn get_value(
        &self,
        path_support_virtual_mode: bool,
        kind: PathInfoUnionValueKind,
    ) -> Option<usize> {
        if path_support_virtual_mode {
            let (value, invalid) = match kind {
                PathInfoUnionValueKind::Primary => {
                    (self.bitfield().higher_u16(), Self::INVALID_PRIMARY)
                }
                PathInfoUnionValueKind::Secondary => {
                    (self.bitfield().lower_u16(), Self::INVALID_SECONDARY)
                }
            };
            (u32::from(value) != invalid).then_some(value as usize)
        } else {
            match kind {
                PathInfoUnionValueKind::Primary => {
                    let value = self.mode_info_idx();
                    (value != DISPLAYCONFIG_PATH_MODE_IDX_INVALID).then_some(value as usize)
                }
                PathInfoUnionValueKind::Secondary => None,
            }
        }
    }

    fn set_value(
        &mut self,
        path_support_virtual_mode: bool,
        kind: PathInfoUnionValueKind,
        value: Option<usize>,
    ) {
        if path_support_virtual_mode {
            #[expect(clippy::cast_possible_truncation)]
            let value = match value {
                Some(val) => val as u16,
                None => Self::INVALID_PRIMARY as u16,
            };

            match kind {
                PathInfoUnionValueKind::Primary => {
                    self.bitfield_mut().set_higher_u16(value);
                }
                PathInfoUnionValueKind::Secondary => {
                    self.bitfield_mut().set_lower_u16(value);
                }
            }
        } else {
            match kind {
                PathInfoUnionValueKind::Primary => {
                    *self.mode_info_idx_mut() = match value {
                        #[expect(clippy::cast_possible_truncation)]
                        Some(val) => val as u32,
                        None => DISPLAYCONFIG_PATH_MODE_IDX_INVALID,
                    };
                }
                PathInfoUnionValueKind::Secondary => {
                    assert!(
                        value.is_none(),
                        "{} cannot be set",
                        Self::SECONDARY_DESCRIPTION
                    );
                }
            }
        }
    }
}
impl PathInfoUnionValuesAccessor for DISPLAYCONFIG_PATH_SOURCE_INFO {
    const INVALID_PRIMARY: u32 = DISPLAYCONFIG_PATH_SOURCE_MODE_IDX_INVALID;
    const INVALID_SECONDARY: u32 = DISPLAYCONFIG_PATH_CLONE_GROUP_INVALID;

    const SECONDARY_DESCRIPTION: &'static str = "Clone group id";

    fn mode_info_idx(&self) -> u32 {
        unsafe { self.Anonymous.modeInfoIdx }
    }

    fn mode_info_idx_mut(&mut self) -> &mut u32 {
        unsafe { &mut self.Anonymous.modeInfoIdx }
    }

    fn bitfield(&self) -> u32 {
        unsafe { self.Anonymous.Anonymous._bitfield }
    }

    fn bitfield_mut(&mut self) -> &mut u32 {
        unsafe { &mut self.Anonymous.Anonymous._bitfield }
    }
}
impl PathInfoUnionValuesAccessor for DISPLAYCONFIG_PATH_TARGET_INFO {
    const INVALID_PRIMARY: u32 = DISPLAYCONFIG_PATH_TARGET_MODE_IDX_INVALID;
    const INVALID_SECONDARY: u32 = DISPLAYCONFIG_PATH_DESKTOP_IMAGE_IDX_INVALID;

    const SECONDARY_DESCRIPTION: &'static str = "Desktop mode idx";

    fn mode_info_idx(&self) -> u32 {
        unsafe { self.Anonymous.modeInfoIdx }
    }

    fn mode_info_idx_mut(&mut self) -> &mut u32 {
        unsafe { &mut self.Anonymous.modeInfoIdx }
    }

    fn bitfield(&self) -> u32 {
        unsafe { self.Anonymous.Anonymous._bitfield }
    }

    fn bitfield_mut(&mut self) -> &mut u32 {
        unsafe { &mut self.Anonymous.Anonymous._bitfield }
    }
}

pub trait U32Ext {
    fn contains(self, flag: u32) -> bool;

    fn lower_u16(self) -> u16;
    fn higher_u16(self) -> u16;

    fn set_lower_u16(&mut self, lower: u16);
    fn set_higher_u16(&mut self, higher: u16);
}
impl U32Ext for u32 {
    fn contains(self, flag: u32) -> bool {
        self & flag == flag
    }

    fn lower_u16(self) -> u16 {
        (self & 0x0000_FFFF) as u16
    }

    fn higher_u16(self) -> u16 {
        (self >> 16) as u16
    }

    fn set_lower_u16(&mut self, lower: u16) {
        *self &= 0xFFFF_0000;
        *self |= u32::from(lower) & 0x0000_FFFF;
    }

    fn set_higher_u16(&mut self, higher: u16) {
        *self &= 0x0000_FFFF;
        *self |= u32::from(higher) << 16;
    }
}

#[must_use]
pub fn from_windows_string(s: &[u16]) -> String {
    let len = s.iter().position(|&c| c == 0).unwrap_or(s.len());
    String::from_utf16_lossy(&s[..len])
}
