use std::collections::HashMap;

pub struct CharacterFlags;
#[allow(dead_code)]
impl CharacterFlags{
    pub const IS_ALIVE:     u8 = 0b10000000;
    pub const JOIN_BATTLE:  u8 = 0b01000000;
    pub const IS_MONSTER:   u8 = 0b00100000;
    pub const IS_STARTED:   u8 = 0b00010000;
    pub const IS_READY:     u8 = 0b00001000;
    pub const ALL_FLAGS_SET:u8 = 0b11111111;
    pub const NO_FLAGS_SET: u8 = 0b00000000;
}

#[derive(Clone, Debug)]
pub struct Character{
    pub name:       String,
    pub is_active:  bool,
    pub flags:      u8,
    pub attack:     u16,
    pub defense:    u16,
    pub regen:      u16,
    pub health:     i16,
    pub gold:       u16,
    pub curr_room:  u16,
    pub desc:       String,
}
impl Character{
    pub fn new(name: String, desc: String) -> Character{
        Character{
            name,
            desc,
            is_active : true,
            flags: 0xFF,
            attack: 50,
            defense: 50,
            regen: 100,
            health: 100,
            gold: 0,
            curr_room: 0,
        }
    }
    pub fn new_monster( name: String, desc:String,
                    attack:u16, defense:u16, regen:u16,
                    health:i16, gold:u16, curr_room:u16) -> Character{
        Character{
            name,
            desc,
            is_active : true,
            flags: CharacterFlags::IS_ALIVE | CharacterFlags::IS_MONSTER,
            attack,
            defense,
            regen,
            health,
            gold,
            curr_room,
        }
    }
}

// Used by game state; conversions will need to be made for Room messages
pub struct Room{
    pub id_num : u16,
    pub name : String,
    pub desc : String,
    pub connections : Vec<u16>,

}
impl Room{
    pub fn new(id_num: u16, name: String, desc: String, connections: Vec<u16>) -> Room{
        Room{id_num, name, desc, connections,}
    }
}

pub struct GameState{
    pub room_hashmap: HashMap<u16, Room>,
    pub character_map: HashMap<String, Character>,
}
impl GameState{
    pub fn new() -> GameState{
        GameState{
            room_hashmap: HashMap::new(),
            character_map: HashMap::new(),
        }
    }
    pub fn add_room(&mut self, room : Room) -> Option<Room> {
        self.room_hashmap.insert(room.id_num, room)
    }
    pub fn add_character(&mut self, character : &mut Character) {
        self.character_map.insert(String::clone(&character.name), Clone::clone(character));
    }
}

