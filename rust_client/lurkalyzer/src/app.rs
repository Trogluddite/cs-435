
pub enum CurrentScreen{
    Main,
    Configuration,
    RawMode,
    LurkMode,
}


#[allow(dead_code)]          //FIXME
pub struct App {
    pub current_screen: CurrentScreen,
    pub server_address: String,
    pub server_port: u16,
    pub server_connected: bool,
    // in buffer
    // out buffer
    // other app state?
}

#[allow(dead_code)]          //FIXME
impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main,
            server_address: String::new(),
            server_port: 0,
            server_connected: false,
        }
    }

    pub fn set_server(&mut self, ip: String, port: u16) {
        self.server_address = ip;
        self.server_port = port;
        //connect here? just seetting up screens now
    }

    pub fn switch_screen(&mut self, target: CurrentScreen) {
        self.current_screen = target;
    }
}
