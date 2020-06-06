
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum ManagerType {
    Event = 1,
    Entity = 2,
    Unknown = 255
}

impl From<u8> for ManagerType {
    fn from(orig: u8) -> Self {
        match orig {
            1 => return ManagerType::Event,
            2 => return ManagerType::Entity,
            _ => return ManagerType::Unknown,
        };
    }
}