
use std::sync::Arc;

use alacritty_terminal::{ansi::Processor, config::MockConfig, term::SizeInfo, Term, grid};
use clipboard::ClipboardProvider;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::Mutex};

use crate::{session::PpPty, errors};

use super::{TerminalEventListener, Render, msg::{PpTerminalMessage, PpTerminalMessageReceiver}};




type AlacrittyTerminal = Term<TerminalEventListener>;
pub struct Terminal {
    pty: Box<dyn PpPty>,
    input: PpTerminalMessageReceiver,
    ui_render: Arc<Mutex<dyn Render>>,
    term: AlacrittyTerminal,
    processor: Processor,
    clipboard: clipboard::ClipboardContext,
}


impl Terminal {
    pub fn new(pty: Box<dyn PpPty>, input: PpTerminalMessageReceiver, ui_render: Arc<Mutex<dyn Render>>) -> Self {
        let cfg = Arc::new(MockConfig::default());
        let size = SizeInfo::new(80.0, 20.0, 1.0, 1.0, 0.0, 0.0, false);
        Self {
            pty, input, ui_render,
            term: AlacrittyTerminal::new(&cfg, size, TerminalEventListener),
            processor: Processor::new(),
            clipboard: clipboard::ClipboardProvider::new().unwrap(),
        }
    }

    pub async fn run(&mut self) -> Result<(),errors::Error> {
        loop {
            let mut buffer = [0u8; 1500];
            tokio::select! {
                n = self.pty.read(&mut buffer) => {
                    match n {
                        Ok(len) => {
                            for byte in &buffer[..len] {
                                self.processor.advance(&mut self.term, *byte);
                            }
                            self.ui_render.lock().await.draw(self.term.renderable_content());
                        }
                        Err(e) => {
                            return Err(errors::Error::ReadPtyError(format!("read pty error: {:?}", e)));
                        }
                    }
                }
                m = self.input.recv() => {
                    match m {
                        Some(msg) => {
                            self.handle_input(msg).await?
                        }
                        None => {
                            if let Err(e) = self.pty.shutdown().await {
                                return Err(errors::Error::WritePtyError(format!("close channel failed with {:?}", e)));
                            } else {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }

    async fn handle_input(&mut self, msg: PpTerminalMessage) -> Result<(), errors::Error> {
        match msg {
            PpTerminalMessage::Input(s) => {
                match self.pty.write_all(s.as_slice()).await {
                    Err(e) => Err(errors::Error::WritePtyError(format!("input to pty error: {:?}", e))),
                    _ => {
                        self.term.selection = None;
                        self.term.scroll_display(grid::Scroll::Bottom);
                        self.ui_render.lock().await.draw(self.term.renderable_content());
                        Ok(())
                    }
                }
            }
            PpTerminalMessage::ReSize(width, height) => {
                self.term.resize(SizeInfo::new(width as f32, height as f32, 1.0, 1.0, 0.0, 0.0, false));
                self.pty.resize(width, height).await
            }
            PpTerminalMessage::Scroll(delta) => {
                self.term.scroll_display(grid::Scroll::Delta(delta));
                self.ui_render.lock().await.draw(self.term.renderable_content());
                Ok(())
            }
            PpTerminalMessage::SelectionStart(line, column) => {
                self.term.selection = Some(
                    alacritty_terminal::selection::Selection::new(
                        alacritty_terminal::selection::SelectionType::Simple,
                        alacritty_terminal::index::Point {
                            line: alacritty_terminal::index::Line(line),
                            column: alacritty_terminal::index::Column(column)
                        },
                        alacritty_terminal::index::Side::Left
                    )
                );
                self.ui_render.lock().await.draw(self.term.renderable_content());
                Ok(())
            }
            PpTerminalMessage::SelectionUpdate(line, column) => {
                if let Some(sr) = &mut self.term.selection {
                    sr.update(
                        alacritty_terminal::index::Point {
                            line: alacritty_terminal::index::Line(line),
                            column: alacritty_terminal::index::Column(column)
                        },
                        alacritty_terminal::index::Side::Left
                    );
                    self.ui_render.lock().await.draw(self.term.renderable_content());
                }
                Ok(())
            }
            PpTerminalMessage::Copy(line, colune) => {
                let mut copyed = false;
                if let Some(s) = &self.term.selection {
                    if let Some(sr) = s.to_range(&self.term) {
                        if sr.contains(
                            alacritty_terminal::index::Point {
                                line: alacritty_terminal::index::Line(line),
                                column: alacritty_terminal::index::Column(colune)
                            }
                        ) {
                            if let Some(s) = self.term.selection_to_string() {
                                self.clipboard.set_contents(s).unwrap();
                                copyed = true;
                                self.term.selection = None;
                            }
                        } else {
                            self.term.selection = None;
                        }
                    }
                }
                if !copyed && self.term.selection.is_none() {
                    let mut data: Vec<u8> = Vec::new();
                    for byte in self.clipboard.get_contents().unwrap().bytes() {
                        data.push(byte);
                    }
                    if let Err(e) = self.pty.write_all(data.as_slice()).await {
                        return Err(errors::Error::WritePtyError(format!("paste to pty error: {:?}", e)));
                    }
                    self.term.selection = None;
                    self.term.scroll_display(grid::Scroll::Bottom);
                }
                self.ui_render.lock().await.draw(self.term.renderable_content());
                Ok(())
            }
        }
    }
}
