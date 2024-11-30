use std::sync::Arc;
use std::net::TcpStream;

pub struct MessageType;
impl MessageType{
    pub const ACCEPT:       u8 = 8;
    pub const CHANGEROOM:   u8 = 2;
    pub const CHARACTER:    u8 = 10;
    pub const CONNECTION:   u8 = 13;
    pub const ERROR:        u8 = 7;
    pub const FIGHT:        u8 = 3;
    pub const GAME:         u8 = 11;
    pub const LEAVE:        u8 = 12;
    pub const LOOT:         u8 = 5;
    pub const MESSAGE:      u8 = 1;
    pub const ROOM:         u8 = 9;
    pub const START:        u8 = 6;
    pub const PVPFIGHT:     u8 = 4;
    pub const VERSION:      u8 = 14;
}

pub struct ErrorType;
#[allow(dead_code)]
impl ErrorType{
    pub const OTHER:        u8 = 0;
    pub const BAD_ROOM:     u8 = 1;
    pub const PLAYER_EXISTS:u8 = 2;
    pub const BAD_MONSTER:  u8 = 3;
    pub const STAT_ERROR:   u8 = 4;
    pub const NOT_READY:    u8 = 5;
    pub const NO_TARGET:    u8 = 6;
    pub const NO_FIGHT:     u8 = 7;
    pub const PVP_DISABLED: u8 = 8;
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Message{
    Accept{
        author:         Arc<TcpStream>,
        message_type:   u8,
        accepted_type:  u8,
    },
    ChangeRoom{
        author:         Arc<TcpStream>,
        message_type:   u8,
        target_room:    u16,
    },
    Character{
        author:         Arc<TcpStream>,
        message_type:   u8,
        character_name: [u8; 32], //expect exactly 32 characters, null-terminated
        flags:          u8,
        attack:         u16,
        defense:        u16,
        regen:          u16,
        health:         i16,
        gold:           u16,
        curr_room:      u16,
        desc_len:       u16,
        desc:           Vec<u8>,
    },
    Connection{
        author:         Arc<TcpStream>,
        message_type:   u8,
        room_number:    u16,
        room_name:      [u8; 32],
        desc_len:       u16,
        room_desc:      Vec<u8>,
    },
    Error{
        author:         Arc<TcpStream>,
        message_type:   u8,
        error_code:     u8,
        message_len:   u16,
        message:        Vec<u8>,
    },
    Fight{
        author:         Arc<TcpStream>,
        message_type:   u8,
    },
    Game{
        author:         Arc<TcpStream>,
        message_type:   u8,
        initial_points: u16,
        stat_limit:     u16,
        desc_len:       u16,
        game_desc:      Vec<u8>,  //to be treated as characters
    },
    Leave{
        author:         Arc<TcpStream>,
        message_type:   u8,
    },
    Loot{
        author:         Arc<TcpStream>,
        message_type:   u8,
        target_name:    Vec<u8>,
    },
    Message{
        author:         Arc<TcpStream>,
        message_type:   u8,
        message_len:    u16,
        recipient_name: [u8; 32],
        sender_name:    [u8; 30],
        end_marker:     u16,
        message:        Vec<u8>,
    },
    Room{
        author:         Arc<TcpStream>,
        message_type:   u8,
        room_number:    u16,
        room_name:      [u8; 32],
        desc_len:       u16,
        room_desc:      Vec<u8>,
    },
    Start{
        author:         Arc<TcpStream>,
        message_type:   u8,
    },
    PVPFight{
        author:         Arc<TcpStream>,
        message_type:   u8,
        target_name:    [u8; 32],
    },
    Version{
        author:         Arc<TcpStream>,
        message_type:   u8,
        major_revision: u8,
        minor_revision: u8,
        ext_len:        u16,
        ext_list:       Vec<u8>,
    },
}
