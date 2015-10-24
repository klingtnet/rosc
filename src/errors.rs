pub enum OscError {
    BadOscPacket(String),
    BadOscAddress(String),
    BadOscMessage(String),
    BadOscBundle,
}
