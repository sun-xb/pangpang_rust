use iced_native::{text::Renderer as TextRenderer, Renderer};



lazy_static::lazy_static! {
    static ref TERMINAL_MONOSPACE: Vec<u8> = iced_graphics::font::Source::new().load(&[
        iced_graphics::font::Family::Title(String::from("仿宋")),
        iced_graphics::font::Family::Title(String::from("楷体")),
        iced_graphics::font::Family::Title(String::from("黑体")),
        iced_graphics::font::Family::Title(String::from("等线")),
        iced_graphics::font::Family::Title(String::from("Cascadia Code")),
        iced_graphics::font::Family::Title(String::from("Cascadia Mono")),
        iced_graphics::font::Family::Monospace
        ]).unwrap();
}

pub struct TerminalView {
    font: iced_graphics::Font,
}

impl TerminalView {
    pub fn new() -> Self {
        Self {
            font: iced_graphics::Font::External{
                name: "terminal",
                bytes: TERMINAL_MONOSPACE.as_ref()
            }
        }
    }
}

impl<Message, B> iced_native::Widget<Message, iced_graphics::Renderer<B>> for TerminalView
where
    B: iced_graphics::Backend + iced_graphics::backend::Text,
{
    fn width(&self) -> iced::Length {
        iced::Length::Fill
    }

    fn height(&self) -> iced::Length {
        iced::Length::Fill
    }

    fn layout(
        &self,
        _renderer: &iced_graphics::Renderer<B>,
        limits: &iced_native::layout::Limits,
    ) -> iced_native::layout::Node {
        iced_native::layout::Node::new(limits.fill())
    }

    fn hash_layout(&self, _state: &mut iced_native::Hasher) {
    }

    fn draw(
        &self,
        renderer: &mut iced_graphics::Renderer<B>,
        style: &iced_native::renderer::Style,
        _layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
        viewport: &iced::Rectangle,
    ) {
        renderer.fill_quad(
            iced_native::renderer::Quad {
                bounds: *viewport,
                border_radius: 0.0,
                border_width: 0.0,
                border_color: iced::Color::TRANSPARENT
            }, style.text_color);
        let t = iced_native::text::Text{
            content: "12345678\nhijklmnm\n一三二四", 
            bounds: iced::Rectangle::new(cursor_position, viewport.size()),
            size: renderer.default_size() as f32,
            color: iced::Color::WHITE,
            font: self.font,
            horizontal_alignment: iced::alignment::Horizontal::Left, 
            vertical_alignment: iced::alignment::Vertical::Top,
        };
        renderer.fill_text(t);
    }

    fn mouse_interaction(&self, _layout: iced_native::Layout<'_>, _cursor_position: iced::Point, _viewport: &iced::Rectangle) -> iced_native::mouse::Interaction {
        iced_native::mouse::Interaction::Text
    }

    fn on_event(&mut self, event: iced_native::Event, _layout: iced_native::Layout<'_>, _cursor_position: iced::Point, _renderer: &iced_graphics::Renderer<B>, _clipboard: &mut dyn iced_native::Clipboard, _shell: &mut iced_native::Shell<'_, Message>) -> iced_native::event::Status {
        match event {
            iced_native::Event::Keyboard(_e) => {
            },
            iced_native::Event::Window(_e) => {},
            _ => {}
        }
        iced_native::event::Status::Captured
    }
}

impl <'a, Message, B> Into<iced_native::Element<'a, Message, iced_graphics::Renderer<B>>> for TerminalView
where
    B: iced_graphics::Backend + iced_graphics::backend::Text,
{
    fn into(self) -> iced_native::Element<'a, Message, iced_graphics::Renderer<B>> {
        iced_native::Element::new(self)
    }
}
