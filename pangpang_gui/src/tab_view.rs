
use eframe::egui;



type TabViewData = Vec<(String, crate::terminal_view::TerminalView)>;


pub struct TabView {
    items: TabViewData,
    selected: usize,
}

impl TabView {
    pub fn new() -> Self {
        Self {
            items: TabViewData::new(),
            selected: 0,
        }
    }

    pub fn insert(&mut self, title: String, term: crate::terminal_view::TerminalView) {
        self.items.push((title, term));
        self.selected = self.items.len() - 1;
    }

    fn paint_tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.fonts().row_height(egui::TextStyle::Heading)),
            egui::Layout::left_to_right(),
            |ui| {
                let mut remove: Option<usize> = None;
                for (i, item) in self.items.iter().enumerate() {
                    let mut close = false;
                    let mut click = false;
                    ui.add(TabItem::new(&item.0, i == self.selected, &mut close, &mut click));
                    if close {
                        remove = Some(i);
                    }
                    if click {
                        self.selected = i;
                    }
                }
                if let Some(i) = remove {
                    self.items.remove(i);
                    if self.selected >= self.items.len() && self.items.len() > 0 {
                        self.selected = self.items.len() - 1;
                    }
                }
            }
        );
    }
}

impl egui::Widget for &mut TabView {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.allocate_ui_with_layout(
            ui.available_size(),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                self.paint_tab_bar(ui);
                if let Some(item) = self.items.get_mut(self.selected) {
                    ui.add(&mut item.1);
                }
            }
        ).response
    }
}


struct TabItem<'a> {
    title: &'a String,
    selected: bool,
    close: &'a mut bool,
    click: &'a mut bool,
}

impl<'a> TabItem<'a> {
    pub fn new(title: &'a String, selected: bool, close: &'a mut bool, click: &'a mut bool) -> Self {
        Self {
            title,
            selected,
            close,
            click,
        }
    }
}

impl<'a> egui::Widget for TabItem<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let width = ui.available_width().min(100.0);
        ui.allocate_ui_with_layout(
            egui::vec2(width, ui.available_height()),
            egui::Layout::right_to_left(),
            |ui| {
                if self.selected {
                    ui.painter().rect_filled(ui.available_rect_before_wrap(), 0.0, egui::Color32::DARK_GREEN);
                }
                let btn = egui::Button::new(char::from_u32(0x1f5d9).unwrap())
                    .text_style(egui::TextStyle::Small)
                    .frame(false);
                if ui.add(btn).clicked() {
                    *self.close = true;
                }
                ui.with_layout(
                    egui::Layout::left_to_right(),
                    |ui| {
                        let rect = ui.available_rect_before_wrap();
                        ui.painter().text(
                            rect.left_center(),
                            egui::Align2::LEFT_CENTER,
                            self.title,
                            egui::TextStyle::Button,
                            ui.style().visuals.text_color()
                        );
                        let response = ui.allocate_rect(rect, egui::Sense::click());
                        if response.clicked() {
                            *self.click = true;
                        }
                    }
                );
            }
        ).response
    }
}

