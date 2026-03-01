pub mod overview;
pub mod performance;
pub mod fans;
pub mod aura;
pub mod gpu;
pub mod battery;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Performance,
    Fans,
    Aura,
    Gpu,
    Battery,
}

impl Tab {
    pub const ALL: [Tab; 6] = [
        Tab::Overview,
        Tab::Performance,
        Tab::Fans,
        Tab::Aura,
        Tab::Gpu,
        Tab::Battery,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Overview => "Overview",
            Tab::Performance => "Performance",
            Tab::Fans => "Fans",
            Tab::Aura => "Aura",
            Tab::Gpu => "GPU",
            Tab::Battery => "Battery",
        }
    }

    pub fn next(self) -> Self {
        let idx = Tab::ALL.iter().position(|&t| t == self).unwrap();
        Tab::ALL[(idx + 1) % Tab::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let idx = Tab::ALL.iter().position(|&t| t == self).unwrap();
        Tab::ALL[(idx + Tab::ALL.len() - 1) % Tab::ALL.len()]
    }

    pub fn from_number(n: u8) -> Option<Self> {
        Tab::ALL.get(n.saturating_sub(1) as usize).copied()
    }
}
