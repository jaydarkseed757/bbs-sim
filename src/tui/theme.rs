use macroquad::prelude::Color;

/// Per-BBS colour palette.  All render modules read from `app.theme` instead
/// of declaring their own constants, so switching BBS changes the entire look.
#[derive(Clone, Debug)]
pub struct BbsTheme {
    /// Header / section titles (was YELLOW)
    pub title:  Color,
    /// Selected / active foreground (was GREEN_BBS)
    pub hi:     Color,
    /// Secondary / unselected foreground (was DIM_GREEN)
    pub lo:     Color,
    /// Field labels, column headers (was CYAN_BBS)
    pub label:  Color,
    /// Message body text (was WHITE_BBS)
    pub body:   Color,
    /// Dim / muted info, timestamps (was DIM_GRAY)
    pub muted:  Color,
    /// Border lines (was DARKGRAY)
    pub border: Color,
    /// Selection row fill (was Color::new(0,0.25,0,1))
    pub sel_bg: Color,
    /// Cursor blink rectangle, includes alpha (was Color::new(0,0.75,0,0.8))
    pub cursor: Color,
    /// Stats / accent — speed, special readouts (was AMBER)
    pub stat:   Color,
}

impl Default for BbsTheme {
    fn default() -> Self { BbsTheme::rusty_nail() }
}

impl BbsTheme {
    pub fn for_slug(slug: &str) -> Self {
        match slug {
            "rusty_nail"           => Self::rusty_nail(),
            "warp_factor_9"        => Self::warp_factor_9(),
            "digital_dungeon"      => Self::digital_dungeon(),
            "elite_force"          => Self::elite_force(),
            "underground_railroad" => Self::underground_railroad(),
            "midnight_rendezvous"  => Self::midnight_rendezvous(),
            "freq_867"             => Self::freq_867(),
            _                      => Self::rusty_nail(),
        }
    }

    // ── Classic phosphor-green terminal ──────────────────────────────────────
    fn rusty_nail() -> Self {
        Self {
            title:  Color::new(1.0,  1.0,  0.0,  1.0), // bright yellow
            hi:     Color::new(0.0,  0.85, 0.0,  1.0), // green
            lo:     Color::new(0.0,  0.55, 0.0,  1.0), // dim green
            label:  Color::new(0.0,  0.87, 0.87, 1.0), // cyan
            body:   Color::new(0.85, 0.85, 0.85, 1.0), // light grey
            muted:  Color::new(0.35, 0.35, 0.35, 1.0), // grey
            border: Color::new(0.25, 0.25, 0.25, 1.0), // dark grey
            sel_bg: Color::new(0.0,  0.25, 0.0,  1.0), // dark green sel
            cursor: Color::new(0.0,  0.75, 0.0,  0.8), // green cursor
            stat:   Color::new(1.0,  0.75, 0.0,  1.0), // amber
        }
    }

    // ── Sci-fi LCARS blue ────────────────────────────────────────────────────
    fn warp_factor_9() -> Self {
        Self {
            title:  Color::new(0.0,  0.95, 1.0,  1.0), // bright cyan
            hi:     Color::new(0.3,  0.85, 1.0,  1.0), // sky blue
            lo:     Color::new(0.1,  0.5,  0.75, 1.0), // dim blue
            label:  Color::new(0.6,  1.0,  1.0,  1.0), // pale cyan
            body:   Color::new(0.8,  0.9,  1.0,  1.0), // blue-white
            muted:  Color::new(0.2,  0.35, 0.5,  1.0), // dark blue-grey
            border: Color::new(0.15, 0.3,  0.5,  1.0), // dark blue
            sel_bg: Color::new(0.0,  0.15, 0.4,  1.0), // dark blue sel
            cursor: Color::new(0.3,  0.8,  1.0,  0.8), // blue cursor
            stat:   Color::new(0.8,  1.0,  0.0,  1.0), // chartreuse
        }
    }

    // ── Amber/gold dungeon terminal ───────────────────────────────────────────
    fn digital_dungeon() -> Self {
        Self {
            title:  Color::new(1.0,  0.84, 0.0,  1.0), // gold
            hi:     Color::new(1.0,  0.7,  0.0,  1.0), // amber
            lo:     Color::new(0.7,  0.45, 0.0,  1.0), // dim amber
            label:  Color::new(1.0,  0.92, 0.5,  1.0), // pale gold
            body:   Color::new(0.9,  0.82, 0.65, 1.0), // parchment
            muted:  Color::new(0.45, 0.32, 0.1,  1.0), // dark brown
            border: Color::new(0.45, 0.32, 0.1,  1.0), // brown border
            sel_bg: Color::new(0.3,  0.2,  0.0,  1.0), // dark amber sel
            cursor: Color::new(1.0,  0.65, 0.0,  0.8), // amber cursor
            stat:   Color::new(1.0,  0.4,  0.0,  1.0), // orange
        }
    }

    // ── Hacker red/white ─────────────────────────────────────────────────────
    fn elite_force() -> Self {
        Self {
            title:  Color::new(1.0,  0.15, 0.15, 1.0), // bright red
            hi:     Color::new(1.0,  0.35, 0.35, 1.0), // red
            lo:     Color::new(0.6,  0.15, 0.15, 1.0), // dim red
            label:  Color::new(1.0,  0.85, 0.85, 1.0), // pink-white
            body:   Color::new(0.92, 0.92, 0.92, 1.0), // near-white
            muted:  Color::new(0.42, 0.18, 0.18, 1.0), // dark red-grey
            border: Color::new(0.45, 0.15, 0.15, 1.0), // dark red border
            sel_bg: Color::new(0.3,  0.05, 0.05, 1.0), // dark red sel
            cursor: Color::new(1.0,  0.2,  0.2,  0.8), // red cursor
            stat:   Color::new(1.0,  0.6,  0.0,  1.0), // orange
        }
    }

    // ── Cool blue-white, political ────────────────────────────────────────────
    fn underground_railroad() -> Self {
        Self {
            title:  Color::new(0.85, 0.9,  1.0,  1.0), // blue-white
            hi:     Color::new(0.65, 0.8,  1.0,  1.0), // pale blue
            lo:     Color::new(0.4,  0.55, 0.8,  1.0), // dim blue
            label:  Color::new(0.55, 0.7,  0.95, 1.0), // light blue
            body:   Color::new(0.88, 0.88, 0.9,  1.0), // near-white
            muted:  Color::new(0.28, 0.32, 0.48, 1.0), // dim blue-grey
            border: Color::new(0.25, 0.3,  0.48, 1.0), // dark blue border
            sel_bg: Color::new(0.08, 0.12, 0.3,  1.0), // dark blue sel
            cursor: Color::new(0.6,  0.75, 1.0,  0.8), // blue cursor
            stat:   Color::new(0.3,  0.9,  0.3,  1.0), // green data
        }
    }

    // ── Orange stage-lights / VU-meter green ─────────────────────────────────
    fn freq_867() -> Self {
        Self {
            title:  Color::new(1.0,  0.38, 0.0,  1.0), // hot orange (neon bar sign)
            hi:     Color::new(1.0,  0.55, 0.05, 1.0), // orange
            lo:     Color::new(0.72, 0.3,  0.02, 1.0), // dim orange
            label:  Color::new(1.0,  0.82, 0.3,  1.0), // warm gold
            body:   Color::new(0.92, 0.88, 0.8,  1.0), // warm cream
            muted:  Color::new(0.42, 0.28, 0.1,  1.0), // dark tan
            border: Color::new(0.38, 0.2,  0.05, 1.0), // dark warm brown
            sel_bg: Color::new(0.3,  0.14, 0.0,  1.0), // dark orange sel
            cursor: Color::new(1.0,  0.45, 0.05, 0.8), // orange cursor
            stat:   Color::new(0.1,  0.92, 0.25, 1.0), // VU-meter green
        }
    }

    // ── Purple/magenta late-night ─────────────────────────────────────────────
    fn midnight_rendezvous() -> Self {
        Self {
            title:  Color::new(0.9,  0.3,  1.0,  1.0), // magenta
            hi:     Color::new(0.75, 0.25, 0.9,  1.0), // purple
            lo:     Color::new(0.45, 0.1,  0.6,  1.0), // dim purple
            label:  Color::new(0.9,  0.65, 1.0,  1.0), // light purple
            body:   Color::new(0.88, 0.82, 0.92, 1.0), // lavender-white
            muted:  Color::new(0.32, 0.22, 0.42, 1.0), // dark purple-grey
            border: Color::new(0.38, 0.2,  0.48, 1.0), // dark purple border
            sel_bg: Color::new(0.2,  0.05, 0.3,  1.0), // dark purple sel
            cursor: Color::new(0.8,  0.3,  1.0,  0.8), // purple cursor
            stat:   Color::new(0.0,  0.9,  0.9,  1.0), // cyan accent
        }
    }
}
