use std::fmt;
use indoc::indoc;
use ratatui::prelude::Color;

#[derive(Copy, Clone)]
pub enum Rank {
    SSSP,
    SSS,
    SSP,
    SS,
    SP,
    S,
    AAA,
    AA,
    A,
    BBB,
    BB,
    B,
    C,
    D,
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Rank::SSSP => "SSS+",
            Rank::SSS  => "SSS",
            Rank::SSP  => "SS+",
            Rank::SS   => "SS",
            Rank::SP   => "S+",
            Rank::S    => "S",
            Rank::AAA  => "AAA",
            Rank::AA   => "AA",
            Rank::A    => "A",
            Rank::BBB  => "BBB",
            Rank::BB   => "BB",
            Rank::B    => "B",
            Rank::C    => "C",
            Rank::D    => "D",
        };
        write!(f, "{}", s)
    }
}
impl Rank {
    pub fn from_percentage(p: f64) -> Rank {
        if p >= 100.5 {
            Rank::SSSP
        } else if p >= 100.0 {
            Rank::SSS
        } else if p >= 99.5 {
            Rank::SSP
        } else if p >= 99.0 {
            Rank::SS
        } else if p >= 98.0 {
            Rank::SP
        } else if p >= 97.0 {
            Rank::S
        } else if p >= 94.0 {
            Rank::AAA
        } else if p >= 90.0 {
            Rank::AA
        } else if p >= 80.0 {
            Rank::A
        } else if p >= 75.0 {
            Rank::BBB
        } else if p >= 70.0 {
            Rank::BB
        } else if p >= 60.0 {
            Rank::B
        } else if p >= 50.0 {
            Rank::C
        } else {
            Rank::D
        }
    }
    // 辅助函数：根据 Rank 给颜色
    pub fn to_color(&self) -> Color {
        match self {
            Rank::SSSP | Rank::SSS => Color::Magenta,
            Rank::SSP | Rank::SS => Color::LightYellow,
            Rank::SP | Rank::S => Color::Yellow,
            Rank::AAA | Rank::AA | Rank::A => Color::Red,
            _ => Color::Gray,
        }
    }
}
impl Rank {
    pub fn to_large_ascii(&self) -> &str {
        match self {
            Rank::SSSP => indoc! {
                r"
                       _______________
                      / ___/ ___/ ___/  __
                      \__ \\__ \\__ \__/ /_
                     ___/ /__/ /__/ /_  __/
                    /____/____/____/ /_/
                "
            },
            Rank::SSS => indoc! {
                r"
                       _______________
                      / ___/ ___/ ___/
                      \__ \\__ \\__ \
                     ___/ /__/ /__/ /
                    /____/____/____/
                "
            },
            Rank::SSP => indoc! {
                r"
                       __________
                      / ___/ ___/  __
                      \__ \\__ \__/ /_
                     ___/ /__/ /_  __/
                    /____/____/ /_/
                "
            },
            Rank::SS => indoc! {
                r"
                       __________
                      / ___/ ___/
                      \__ \\__ \
                     ___/ /__/ /
                    /____/____/
                "
            },
            Rank::SP => indoc! {
                r"
                       _____
                      / ___/  __
                      \__ \__/ /_
                     ___/ /_  __/
                    /____/ /_/
                "
            },
            Rank::S => indoc! {
                r"
                       _____
                      / ___/
                      \__ \
                     ___/ /
                    /____/
                "
            },
            Rank::AAA => indoc! {
                r"
                        ___    ___    ___
                       /   |  /   |  /   |
                      / /| | / /| | / /| |
                     / ___ |/ ___ |/ ___ |
                    /_/  |_/_/  |_/_/  |_|
                "
            },
            Rank::AA => indoc! {
                r"
                        ___    ___
                       /   |  /   |
                      / /| | / /| |
                     / ___ |/ ___ |
                    /_/  |_/_/  |_|
                "
            },
            Rank::A => indoc! {
                r"
                        ___
                       /   |
                      / /| |
                     / ___ |
                    /_/  |_|
                "
            },
            Rank::BBB => indoc! {
                r"
                        ____  ____  ____
                       / __ )/ __ )/ __ )
                      / __  / __  / __  |
                     / /_/ / /_/ / /_/ /
                    /_____/_____/_____/
                "
            },
            Rank::BB => indoc! {
                r"
                        ____  ____
                       / __ )/ __ )
                      / __  / __  |
                     / /_/ / /_/ /
                    /_____/_____/
               "
            },
            Rank::B => indoc! {
                r"
                        ____
                       / __ )
                      / __  |
                     / /_/ /
                    /_____/
                "
            },
            Rank::C => indoc! {
                r"
                       ______
                      / ____/
                     / /
                    / /___
                    \____/
                "
            },
            Rank::D => indoc! {
                r"
                        ____
                       / __ \
                      / / / /
                     / /_/ /
                    /_____/
                "
            },
        }
    }
}
