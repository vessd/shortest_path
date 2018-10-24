mod map_grid;

use gtk::Inhibit;
use gtk::{ButtonExt, GridExt, GtkWindowExt, WidgetExt};
use relm::Widget;
use relm_attributes::widget;

use self::map_grid::MapGrid;
use self::Msg::Quit;

//type Result<T> = std::result::Result<T, ::failure::Error>;

pub struct Model {
    _counter: i32,
}

#[derive(Msg)]
pub enum Msg {
    Quit,
}

#[widget]
impl Widget for Win {
    fn model() -> Model {
        Model { _counter: 0 }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window {
            title: "Поиск кратчайшего пути",
            gtk::Grid {
                column_homogeneous: true,
                row_homogeneous: true,
                MapGrid((32,18)) {
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
            },
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}
