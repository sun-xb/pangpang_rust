


pub struct TerminalView;

impl<Message, Renderer> iced_native::Widget<Message, Renderer> for TerminalView
where
    Renderer: iced_native::renderer::Renderer
{
    fn width(&self) -> iced::Length {
        iced::Length::Shrink
    }

    fn height(&self) -> iced::Length {
        iced::Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &iced_native::layout::Limits,
    ) -> iced_native::layout::Node {
        iced_native::layout::Node::new(limits.fill())
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
        viewport: &iced::Rectangle,
    ) -> Renderer::Output {
        todo!()
    }

    fn hash_layout(&self, state: &mut iced_native::Hasher) {
        use std::hash::Hash;

        (100.0_f32).to_bits().hash(state);
    }
}