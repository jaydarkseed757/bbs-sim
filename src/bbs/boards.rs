use crate::bbs::data::{Board, Message, Thread};

pub struct BoardStore {
    pub boards: Vec<Board>,
}

impl BoardStore {
    pub fn new() -> Self {
        Self { boards: vec![] }
    }

    pub fn get_board(&self, id: &str) -> Option<&Board> {
        self.boards.iter().find(|b| b.id == id)
    }
}

pub fn hardcoded_boards() -> Vec<Board> {
    vec![
        Board {
            id: "general".into(),
            name: "General".into(),
            threads: vec![
                Thread {
                    id: 1,
                    subject: "Welcome to The Rusty Nail!".into(),
                    posts: vec![
                        Message {
                            id: 1,
                            author: "ShadowByte".into(),
                            subject: "Welcome to The Rusty Nail!".into(),
                            body: "Hey everyone, welcome aboard The Rusty Nail BBS!\n\
                                   Rules are simple: be cool, no lamers, no warez drama.\n\
                                   This is your home away from home in cyberspace.".into(),
                            timestamp: "04/01/93 09:14".into(),
                        },
                        Message {
                            id: 2,
                            author: "PhreakMaster".into(),
                            subject: "Re: Welcome to The Rusty Nail!".into(),
                            body: "Thanks! Glad to be here. This place looks rad.\n\
                                   Anyone else running a C64 out there? Just got my SwiftLink.".into(),
                            timestamp: "04/01/93 11:42".into(),
                        },
                        Message {
                            id: 3,
                            author: "NeonByte".into(),
                            subject: "Re: Welcome to The Rusty Nail!".into(),
                            body: "C64 gang represent! SwiftLink + 38400 baud = smooth as butter.\n\
                                   Glad the sysop finally got this place running again.".into(),
                            timestamp: "04/02/93 02:17".into(),
                        },
                    ],
                },
                Thread {
                    id: 2,
                    subject: "What modem are you running?".into(),
                    posts: vec![
                        Message {
                            id: 1,
                            author: "DataSurfer".into(),
                            subject: "What modem are you running?".into(),
                            body: "Just picked up a USRobotics Sportster 14.4k last month. Incredible speed.\n\
                                   Worth every penny of the $299. What are you all running?".into(),
                            timestamp: "04/10/93 18:05".into(),
                        },
                        Message {
                            id: 2,
                            author: "ShadowByte".into(),
                            subject: "Re: What modem are you running?".into(),
                            body: "Still rocking the 2400 Hayes here. Saving up for that USR.\n\
                                   Maybe by summer if I can convince my parents it's educational.".into(),
                            timestamp: "04/10/93 20:33".into(),
                        },
                        Message {
                            id: 3,
                            author: "DataSurfer".into(),
                            subject: "Re: What modem are you running?".into(),
                            body: "Ha! I told mine it was for homework research. Worked like a charm.\n\
                                   Seriously though the 14.4 is a gamechanger. Z-modem at full speed is wild.".into(),
                            timestamp: "04/11/93 09:01".into(),
                        },
                    ],
                },
                Thread {
                    id: 3,
                    subject: "Anyone going to Defcon this summer?".into(),
                    posts: vec![
                        Message {
                            id: 1,
                            author: "H4x0rZone".into(),
                            subject: "Anyone going to Defcon this summer?".into(),
                            body: "First Defcon is happening in July out in Las Vegas.\n\
                                   Dark Tangent is organizing it. Who's going?\n\
                                   I hear it's gonna be the real deal — not some lame expo.".into(),
                            timestamp: "04/20/93 14:44".into(),
                        },
                    ],
                },
            ],
        },
        Board {
            id: "tech".into(),
            name: "Tech Talk".into(),
            threads: vec![
                Thread {
                    id: 1,
                    subject: "DOS 6.0 -- worth upgrading?".into(),
                    posts: vec![
                        Message {
                            id: 1,
                            author: "PcJockey".into(),
                            subject: "DOS 6.0 -- worth upgrading?".into(),
                            body: "Microsoft just dropped DOS 6.0. DoubleSpace looks interesting\n\
                                   but I've heard scary stories about it eating whole drives.\n\
                                   Anyone tried it yet? Worth the $50 upgrade?".into(),
                            timestamp: "03/31/93 10:00".into(),
                        },
                        Message {
                            id: 2,
                            author: "TechWiz".into(),
                            subject: "Re: DOS 6.0 -- worth upgrading?".into(),
                            body: "DoubleSpace is risky. Stick with Stacker if you need compression.\n\
                                   MemMaker is legit though -- freed up 47k of conventional RAM for me.\n\
                                   MultiConfig boot menus alone make it worth it.".into(),
                            timestamp: "04/01/93 08:21".into(),
                        },
                        Message {
                            id: 3,
                            author: "NeonByte".into(),
                            subject: "Re: DOS 6.0 -- worth upgrading?".into(),
                            body: "MultiConfig in CONFIG.SYS is the killer feature imo.\n\
                                   Boot menu between DOS gaming config and Windows config? Yes please.\n\
                                   Finally I can have HIMEM and EMM386 only when I need them.".into(),
                            timestamp: "04/02/93 16:09".into(),
                        },
                    ],
                },
                Thread {
                    id: 2,
                    subject: "VGA vs SVGA -- real difference?".into(),
                    posts: vec![
                        Message {
                            id: 1,
                            author: "PixelPusher".into(),
                            subject: "VGA vs SVGA -- real difference?".into(),
                            body: "My friend says SVGA is just marketing hype.\n\
                                   Is 800x600 actually noticeably better or is 320x200 good enough?".into(),
                            timestamp: "04/15/93 21:00".into(),
                        },
                        Message {
                            id: 2,
                            author: "GfxGuru".into(),
                            subject: "Re: VGA vs SVGA -- real difference?".into(),
                            body: "For games? 320x200 Mode 13h is king -- palette tricks look amazing.\n\
                                   For productivity and Windows though? 800x600 is a huge difference.\n\
                                   Get a Diamond Stealth VGA with 1MB VRAM and you can do both.".into(),
                            timestamp: "04/16/93 00:15".into(),
                        },
                    ],
                },
            ],
        },
    ]
}
