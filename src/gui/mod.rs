mod map_grid;

use gtk::Inhibit;
use gtk::{ButtonExt, GridExt, GtkWindowExt, WidgetExt};
use relm::Widget;
use relm_attributes::widget;

use self::map_grid::MapGrid;

//type Result<T> = std::result::Result<T, ::failure::Error>;

pub struct Model {
    _counter: i32,
}

#[derive(Msg)]
pub enum Msg {
    Path,
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model { _counter: 0 }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Path => {
                if self.path_button.get_label().unwrap() == "Поиск пути" {
                    self.path_button.set_label("Очистить путь");
                    self.map_grid.emit(self::map_grid::Msg::FindPath);
                } else {
                    self.path_button.set_label("Поиск пути");
                    self.map_grid.emit(self::map_grid::Msg::ClearPath);
                }
            }
            Msg::Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            title: "Поиск кратчайшего пути",
            gtk::Grid {
                column_homogeneous: true,
                row_homogeneous: true,
                column_spacing: 4,
                row_spacing: 2,
                #[name="map_grid"]
                MapGrid((18, 32)) {
                    cell: {
                        left_attach: 0,
                        top_attach: 0,
                        width: 32,
                        height: 18,
                    },
                },
                gtk::Button {
                    label: "Сохранить карту",
                    cell: {
                        left_attach: 0,
                        top_attach: 18,
                        width: 4,
                        height: 1,
                    },
                },
                gtk::Button {
                    label: "Загрузить карту",
                    cell: {
                        left_attach: 0,
                        top_attach: 19,
                        width: 4,
                        height: 1,
                    },
                },
                #[name="path_button"]
                gtk::Button {
                    label: "Поиск пути",
                    cell: {
                        left_attach: 28,
                        top_attach: 18,
                        width: 4,
                        height: 1,
                    },
                    clicked => Msg::Path,
                },
            },
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}
