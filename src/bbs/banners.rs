use crate::bbs::data::BbsEntry;

/// Returns the full ANSI-free ASCII art welcome banner for a BBS,
/// formatted for an 80-column terminal with \r\n line endings.
pub fn bbs_banner(bbs: &BbsEntry) -> String {
    match bbs.slug.as_str() {
        "rusty_nail"         => rusty_nail(bbs),
        "warp_factor_9"      => warp_factor_9(bbs),
        "digital_dungeon"    => digital_dungeon(bbs),
        "elite_force"        => elite_force(bbs),
        "underground_railroad" => underground_railroad(bbs),
        "midnight_rendezvous"  => midnight_rendezvous(bbs),
        "freq_867"           => freq_867(bbs),
        _                    => generic(bbs),
    }
}

// ── Individual banners ────────────────────────────────────────────────────────

fn rusty_nail(_bbs: &BbsEntry) -> String {
    [
        "",
        r"  ___________________________________________",
        r" /  _________________________________________/|",
        r"/  /                                        / |",
        r"|  |    T H E   R U S T Y   N A I L        | |",
        r"|  |                                        | |",
        r"|  |               B B S                   | |",
        r"|  |________________________________________| /",
        r" \_________________________________________/",
        r"",
        r"           |",
        r"      -----+-----    Cincinnati, OH",
        r"           |         Sysop: ShadowByte",
        r"          \|/        2400 baud  --  Est. 1989",
        r"     ------+------",
        r"",
        r"   Boards: General | Warez | C64 | Tech Talk",
        r"",
        r"   'Your home in cyberspace since 1989'",
        "",
    ].join("\r\n")
}

fn warp_factor_9(_bbs: &BbsEntry) -> String {
    [
        "",
        r"   .  *  .  *  .  *  .  *  .  *  .  *  .  *  .",
        r"  *                                             *",
        r"  .     __|__                                   .",
        r"  *    /  |  \    W A R P   F A C T O R   9    *",
        r"  .   /   |   \                                 .",
        r"  *  /    |    \           B  B  S              *",
        r"  .       |                                     .",
        r"  *    ---+---       Seattle, WA                *",
        r"  .                  Sysop: KirkFan             .",
        r"  *    ---+---       9600 baud                  *",
        r"  .                                             .",
        r"  *   'To boldly go where no modem has gone'   *",
        r"  .                                             .",
        r"  *   Boards: SciFi | Gaming | Anime           *",
        r"  .                                             .",
        r"   .  *  .  *  .  *  .  *  .  *  .  *  .  *  .",
        "",
    ].join("\r\n")
}

fn digital_dungeon(_bbs: &BbsEntry) -> String {
    [
        "",
        r"  +----<[ WELCOME, ADVENTURER ]>----+",
        r"  |                                 |",
        r"  |   ~~~  THE DIGITAL DUNGEON  ~~~ |",
        r"  |                                 |",
        r"  |         B  B  S                 |",
        r"  |                                 |",
        r"  |  \/\/\/\/  ENTER  \/\/\/\/      |",
        r"  |                                 |",
        r"  |  Sysop : DungeonMstr            |",
        r"  |  Loc   : Austin, TX             |",
        r"  |  Baud  : 2400                   |",
        r"  |  Boards: RPG | D&D | General    |",
        r"  |                                 |",
        r"  |  'Roll for initiative...'       |",
        r"  |                                 |",
        r"  +--<[ Save vs. Disconnection ]>---+",
        "",
    ].join("\r\n")
}

fn elite_force(_bbs: &BbsEntry) -> String {
    [
        "",
        r"  <:::::::::::::::::::::::::::::::::::::::::>",
        r"  <:                                       :>",
        r"  <:   *** E L I T E   F O R C E ***      :>",
        r"  <:                                       :>",
        r"  <:              B  B  S                  :>",
        r"  <:                                       :>",
        r"  <:::::::::::::::::::::::::::::::::::::::::>",
        r"  <:                                       :>",
        r"  <:  >>  ACCESS LEVEL  : UNVERIFIED   <<  :>",
        r"  <:  >>  PROTOCOL      : ZMODEM 14.4k <<  :>",
        r"  <:  >>  LOCATION      : BOSTON, MA   <<  :>",
        r"  <:  >>  SYSOP         : 31337         <<  :>",
        r"  <:  >>  BOARDS        : WAREZ/HACK/   <<  :>",
        r"  <:  >>                  MUSIC         <<  :>",
        r"  <:                                       :>",
        r"  <:  -- ELITE ONLY.  LAMERS GET NUKED --  :>",
        r"  <:                                       :>",
        r"  <:::::::::::::::::::::::::::::::::::::::::>",
        "",
    ].join("\r\n")
}

fn underground_railroad(_bbs: &BbsEntry) -> String {
    [
        "",
        r"  =============================================",
        r"  ||                                         ||",
        r"  ||    T H E   U N D E R G R O U N D       ||",
        r"  ||                                         ||",
        r"  ||            R A I L R O A D              ||",
        r"  ||                                         ||",
        r"  ||         A Chicago BBS for the People    ||",
        r"  ||                                         ||",
        r"  =============================================",
        r"  ||                                         ||",
        r"  ||  Sysop  : ConductorX   Chicago, IL     ||",
        r"  ||  Baud   : 2400         Est. Sept. 1992  ||",
        r"  ||  Boards : Politics | News | General     ||",
        r"  ||                                         ||",
        r"  ||  'Information wants to be free'         ||",
        r"  ||                                         ||",
        r"  =============================================",
        "",
    ].join("\r\n")
}

fn midnight_rendezvous(_bbs: &BbsEntry) -> String {
    [
        "",
        r"  -=*=- -=*=- -=*=- -=*=- -=*=- -=*=- -=*=-",
        r"  *                                          *",
        r"  =     M I D N I G H T                     =",
        r"  *                                          *",
        r"  =         R E N D E Z V O U S             =",
        r"  *                                          *",
        r"  =     New York City Underground BBS        =",
        r"  *                                          *",
        r"  -=*=- -=*=- -=*=- -=*=- -=*=- -=*=- -=*=-",
        r"  *                                          *",
        r"  =  Sysop    : Pr0phet   New York, NY       =",
        r"  *  Baud     : 9600      Online: 12am-6am   *",
        r"  =  Boards   : Phreaking | Underground      =",
        r"  *               | Lounge                   *",
        r"  =  Phone    : 555-2600                     =",
        r"  *                                          *",
        r"  =  'The city never sleeps. Neither do we.' =",
        r"  *                                          *",
        r"  -=*=- -=*=- -=*=- -=*=- -=*=- -=*=- -=*=-",
        "",
    ].join("\r\n")
}

fn freq_867(_bbs: &BbsEntry) -> String {
    [
        "",
        r"  ~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~",
        r"  ~                                    ~",
        r"  ~     * * * F R E Q  8 6 7 * * *    ~",
        r"  ~                                    ~",
        r"  ~            B  B  S                 ~",
        r"  ~                                    ~",
        r"  ~    Music Is The Only Truth         ~",
        r"  ~                                    ~",
        r"  ~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~",
        r"  ~                                    ~",
        r"  ~  SysOp  :  SpinDr                 ~",
        r"  ~  Line   :  867-5309               ~",
        r"  ~  City   :  Nashville, TN           ~",
        r"  ~  Speed  :  14400 Baud              ~",
        r"  ~                                    ~",
        r"  ~  Boards :  Alt/Grunge  |  Metal    ~",
        r"  ~             Hip-Hop  |  New  |     ~",
        r"  ~             Concert Talk           ~",
        r"  ~                                    ~",
        r"  ~  'If it's too loud,               ~",
        r"  ~        you're too old.'            ~",
        r"  ~                                    ~",
        r"  ~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~-~",
        "",
    ].join("\r\n")
}

fn generic(bbs: &BbsEntry) -> String {
    let boards = bbs.boards.join("  |  ");
    let sep    = "-".repeat(44);
    format!(
        "\r\n{sep}\r\n  {name}\r\n{sep}\r\n\r\n\
         Sysop:    {sysop}\r\n\
         Location: {location}\r\n\
         Baud:     {baud}\r\n\
         Boards:   {boards}\r\n\r\n",
        sep      = sep,
        name     = bbs.name,
        sysop    = bbs.sysop,
        location = bbs.location,
        baud     = bbs.baud,
        boards   = boards,
    )
}
