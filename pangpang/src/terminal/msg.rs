



pub type PpTerminalMessageReceiver = tokio::sync::mpsc::Receiver<PpTerminalMessage>;
pub type PpTerminalMessageSender = tokio::sync::mpsc::Sender<PpTerminalMessage>;

#[derive(Debug)]
pub enum PpTerminalMessage {
    Input(Vec<u8>),
    ReSize(usize, usize),
    Scroll(i32),
    Clicked(i32, usize, bool),
    MouseMove(i32, usize),
}

