use windows::Win32::Devices::Display::{
    DISPLAYCONFIG_PATH_SOURCE_INFO, DISPLAYCONFIG_PATH_TARGET_INFO,
};
use windows::Win32::Foundation::LUID;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceId {
    pub adapter_id: LUID,
    pub id: u32,
}

impl Eq for DeviceId {}

impl PartialOrd for DeviceId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DeviceId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let key = |did: &DeviceId| (did.adapter_id.HighPart, did.adapter_id.LowPart, did.id);
        key(self).cmp(&key(other))
    }
}

impl From<DISPLAYCONFIG_PATH_SOURCE_INFO> for DeviceId {
    fn from(value: DISPLAYCONFIG_PATH_SOURCE_INFO) -> Self {
        DeviceId {
            adapter_id: value.adapterId,
            id: value.id,
        }
    }
}

impl From<DISPLAYCONFIG_PATH_TARGET_INFO> for DeviceId {
    fn from(value: DISPLAYCONFIG_PATH_TARGET_INFO) -> Self {
        DeviceId {
            adapter_id: value.adapterId,
            id: value.id,
        }
    }
}
