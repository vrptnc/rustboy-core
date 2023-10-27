#[derive(Copy, Clone, Debug)]
pub struct CPUInfo {
    pub af: u16,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub sp: u16,
    pub pc: u16,
    pub stopped: bool,
    pub enabled: bool,
}