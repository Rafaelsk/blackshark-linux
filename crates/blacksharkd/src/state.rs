#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MicMuteState {
    Unknown,
    Unmuted,
    Muted,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Transport {
    #[default]
    None,
    Wireless,
    Usb,
}

impl Transport {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Wireless => "wireless",
            Self::Usb => "usb",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SharedState {
    pub connected: bool,
    pub transport: Transport,
    pub battery_pct: u8,
    pub charging: bool,
    pub mic_mute: MicMuteState,
    pub sidetone: u8,
    pub eq_preset: u8,
    pub thx_enabled: bool,
    pub anc_enabled: bool,
    pub anc_level: u8,
    pub power_savings_minutes: u8,
}

impl Default for SharedState {
    fn default() -> Self {
        Self {
            connected: false,
            transport: Transport::None,
            battery_pct: 0,
            charging: false,
            mic_mute: MicMuteState::Unknown,
            sidetone: 0,
            eq_preset: 0,
            thx_enabled: false,
            anc_enabled: false,
            anc_level: 1,
            power_savings_minutes: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Transport;

    #[test]
    fn transport_values_are_stable_for_dbus() {
        assert_eq!(Transport::None.as_str(), "none");
        assert_eq!(Transport::Wireless.as_str(), "wireless");
        assert_eq!(Transport::Usb.as_str(), "usb");
    }
}
