use druid::WidgetExt;





pub(crate) fn main_window() -> impl druid::Widget<crate::PpState> {
    druid::widget::Split::columns(
        druid::widget::Slider::new().lens(crate::PpState::slider_test),
        crate::widgets::TerminalView::default().lens(crate::PpState::terminal_grid)
    )
    .draggable(true)
    .bar_size(3.0)
    .split_point(0.2)
}

