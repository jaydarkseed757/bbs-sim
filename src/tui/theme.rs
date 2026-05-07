use macroquad::prelude::Color;

pub struct Theme {
    pub title:    Color,  // screen headers, [Key] hints
    pub primary:  Color,  // main content text
    pub secondary:Color,  // hints, descriptions, dimmer text
    pub highlight:Color,  // handles, section headers, accents
    pub dim:      Color,  // dates, percentages, very minor text
    pub border:   Color,  // box outlines
    pub sel_bg:   Color,  // selected-row background rectangle
    pub cursor:   Color,  // text-input block cursor
}

impl Theme {
    pub fn for_slug(slug: &str) -> Self {
        match slug {
            "rusty_nail"           => Self::rusty_nail(),
            "warp_factor_9"        => Self::warp_factor_9(),
            "digital_dungeon"      => Self::digital_dungeon(),
            "elite_force"          => Self::elite_force(),
            "underground_railroad" => Self::underground_railroad(),
            "midnight_rendezvous"  => Self::midnight_rendezvous(),
            _                      => Self::rusty_nail(),
        }
    }

    // Classic green phosphor CRT
    fn rusty_nail() -> Self {
        Self {
            title:    Color::new(1.0,  0.85, 0.0,  1.0),
            primary:  Color::new(0.0,  0.85, 0.0,  1.0),
            secondary:Color::new(0.0,  0.55, 0.0,  1.0),
            highlight:Color::new(0.0,  0.87, 0.87, 1.0),
            dim:      Color::new(0.35, 0.35, 0.35, 1.0),
            border:   Color::new(0.31, 0.31, 0.31, 1.0),
            sel_bg:   Color::new(0.0,  0.25, 0.0,  1.0),
            cursor:   Color::new(0.0,  0.75, 0.0,  0.8),
        }
    }

    // Sci-fi cyan/steel-blue — starship terminal
    fn warp_factor_9() -> Self {
        Self {
            title:    Color::new(0.0,  0.9,  1.0,  1.0),
            primary:  Color::new(0.45, 0.75, 1.0,  1.0),
            secondary:Color::new(0.25, 0.5,  0.75, 1.0),
            highlight:Color::new(1.0,  0.85, 0.0,  1.0),
            dim:      Color::new(0.25, 0.35, 0.5,  1.0),
            border:   Color::new(0.0,  0.4,  0.6,  1.0),
            sel_bg:   Color::new(0.0,  0.08, 0.22, 1.0),
            cursor:   Color::new(0.0,  0.7,  1.0,  0.8),
        }
    }

    // Amber phosphor monitor
    fn digital_dungeon() -> Self {
        Self {
            title:    Color::new(1.0,  0.65, 0.0,  1.0),
            primary:  Color::new(0.9,  0.52, 0.0,  1.0),
            secondary:Color::new(0.6,  0.35, 0.0,  1.0),
            highlight:Color::new(1.0,  0.82, 0.2,  1.0),
            dim:      Color::new(0.35, 0.2,  0.0,  1.0),
            border:   Color::new(0.4,  0.25, 0.0,  1.0),
            sel_bg:   Color::new(0.18, 0.10, 0.0,  1.0),
            cursor:   Color::new(1.0,  0.6,  0.0,  0.8),
        }
    }

    // Danger red — elite hacker
    fn elite_force() -> Self {
        Self {
            title:    Color::new(1.0,  0.25, 0.25, 1.0),
            primary:  Color::new(1.0,  0.8,  0.8,  1.0),
            secondary:Color::new(0.75, 0.4,  0.4,  1.0),
            highlight:Color::new(1.0,  0.45, 0.0,  1.0),
            dim:      Color::new(0.4,  0.22, 0.22, 1.0),
            border:   Color::new(0.5,  0.15, 0.15, 1.0),
            sel_bg:   Color::new(0.22, 0.05, 0.05, 1.0),
            cursor:   Color::new(1.0,  0.3,  0.3,  0.8),
        }
    }

    // Cool white / sky-blue — civil liberties
    fn underground_railroad() -> Self {
        Self {
            title:    Color::new(0.5,  0.8,  1.0,  1.0),
            primary:  Color::new(0.82, 0.87, 0.95, 1.0),
            secondary:Color::new(0.5,  0.62, 0.8,  1.0),
            highlight:Color::new(0.6,  0.9,  1.0,  1.0),
            dim:      Color::new(0.35, 0.42, 0.55, 1.0),
            border:   Color::new(0.2,  0.35, 0.55, 1.0),
            sel_bg:   Color::new(0.05, 0.10, 0.22, 1.0),
            cursor:   Color::new(0.5,  0.75, 1.0,  0.8),
        }
    }

    // Lavender / violet — late-night purple
    fn midnight_rendezvous() -> Self {
        Self {
            title:    Color::new(0.85, 0.3,  1.0,  1.0),
            primary:  Color::new(0.78, 0.58, 1.0,  1.0),
            secondary:Color::new(0.52, 0.35, 0.75, 1.0),
            highlight:Color::new(1.0,  0.45, 0.85, 1.0),
            dim:      Color::new(0.32, 0.22, 0.48, 1.0),
            border:   Color::new(0.38, 0.12, 0.55, 1.0),
            sel_bg:   Color::new(0.15, 0.05, 0.25, 1.0),
            cursor:   Color::new(0.8,  0.3,  1.0,  0.8),
        }
    }
}
