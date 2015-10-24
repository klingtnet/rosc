pub enum OscError {
    BadOscPacket(String),
    BadOscAddress(String),
    BadOscBundle
}
