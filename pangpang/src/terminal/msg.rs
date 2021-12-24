



pub type PpTerminalMessageReceiver = tokio::sync::mpsc::Receiver<PpTerminalMessage>;
pub type PpTerminalMessageSender = tokio::sync::mpsc::Sender<PpTerminalMessage>;


pub enum PpTerminalMessage {
    Input(Vec<u8>),
    ReSize(usize, usize),
    Scroll(i32),
    SelectionStart(i32, usize),
    SelectionUpdate(i32, usize),
    Copy(i32, usize)
}
