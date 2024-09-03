mod uci;

pub fn start() {
    uci::UCI::new().run();
}
