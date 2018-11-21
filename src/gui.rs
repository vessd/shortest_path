use bincode::{deserialize, serialize};
use gdk::EventMask;
use gtk::ContainerExt;
use gtk::{ButtonExt, ComboBoxExt, ComboBoxTextExt, DialogExt};
use gtk::{DrawingArea, FileChooserExt, GridExt, GtkWindowExt, Inhibit};
use gtk::{LabelExt, NativeDialogExt, NotebookExtManual, TextBufferExt, WidgetExt};
use map::{Algorithm, Cell, Map, MapPos, SearchStatus, ShortestPath};
use relm::{interval, DrawHandler, Relm, Widget};
use relm_attributes::widget;
use std::fs;

// макрос для распаковки Result или вывода окна с ошибкой
macro_rules! try_message {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                Win::error_message(&err.to_string());
                return;
            }
        }
    };
}

#[derive(Debug, Clone)]
struct Color {
    red: f64,
    green: f64,
    blue: f64,
}

impl Color {
    fn white() -> Self {
        Self {
            red: 1f64,
            green: 1f64,
            blue: 1f64,
        }
    }

    fn red() -> Self {
        Self {
            red: 1f64,
            green: 0f64,
            blue: 0f64,
        }
    }

    fn green() -> Self {
        Self {
            red: 0f64,
            green: 1f64,
            blue: 0f64,
        }
    }

    fn grey() -> Self {
        Self {
            red: 0.5f64,
            green: 0.5f64,
            blue: 0.5f64,
        }
    }

    fn yellow() -> Self {
        Self {
            red: 1f64,
            green: 1f64,
            blue: 0f64,
        }
    }

    fn pale_blue() -> Self {
        Self {
            red: 0.68359375f64,
            green: 0.9296875f64,
            blue: 0.9296875f64,
        }
    }

    fn pale_green() -> Self {
        Self {
            red: 0.59375f64,
            green: 0.98046875f64,
            blue: 0.59375f64,
        }
    }
}

// состояние указателя мыши
struct Cursor {
    position: (f64, f64),
    button_pressed: bool,
    cell: Cell,
}

// модель виджета
pub struct Model {
    draw_handler: DrawHandler<DrawingArea>,
    search: ShortestPath,
    status: SearchStatus,
    path: Option<Vec<MapPos>>,
    cursor: Cursor,
}

// сообщения, которые можно отправлять виджету
#[derive(Msg)]
pub enum Msg {
    About,
    AlgorithmChange,
    ButtonPress,
    ButtonRelease,
    Clear,
    ClearPath,
    FindPath,
    MoveCursor((f64, f64)),
    Next,
    Open,
    Quit,
    Save,
    UpdateDrawBuffer,
}

impl Win {
    // возвращает координаты клетки на которую указывает указатель мыши
    fn get_cursor_pos(&self) -> MapPos {
        // размер отображаемой карты
        let allocation = self.drawing_area.get_allocation();
        let x = match self.model.cursor.position.1.round() {
            x if x < 0f64 => 0,
            x if x >= f64::from(allocation.height) => self.model.search.map.rows() - 1,
            x => (x / f64::from(allocation.height) * self.model.search.map.rows() as f64) as usize,
        };
        let y = match self.model.cursor.position.0.round() {
            y if y < 0f64 => 0,
            y if y >= f64::from(allocation.width) => self.model.search.map.cols() - 1,
            y => (y / f64::from(allocation.width) * self.model.search.map.cols() as f64) as usize,
        };
        MapPos::new(x, y)
    }

    // выводит сообщение об успехе
    fn success_message(&self, message: &str) {
        let dialog = gtk::MessageDialog::new(
            Some(&self.window),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Info,
            gtk::ButtonsType::Ok,
            message,
        );
        dialog.run();
        dialog.destroy();
    }

    // выводит сообщение об ошибке
    fn error_message(message: &str) {
        let dialog = gtk::MessageDialog::new(
            None::<&gtk::Window>,
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Error,
            gtk::ButtonsType::Close,
            message,
        );
        dialog.run();
        dialog.destroy();
    }
}

#[widget]
impl Widget for Win {
    // инициализация элементов виджета
    fn init_view(&mut self) {
        self.combo_box.append_text("Поиск в ширину");
        self.combo_box
            .append_text("Алгоритм Дейкстры");
        self.combo_box.append_text("А*");
        self.combo_box.set_active(2);

        self.model.draw_handler.init(&self.drawing_area);
        self.drawing_area.add_events(
            (EventMask::BUTTON_PRESS_MASK
                | EventMask::BUTTON_RELEASE_MASK
                | EventMask::POINTER_MOTION_MASK)
                .bits() as i32,
        );
    }

    // инициализация модели виджета
    fn model(size: (usize, usize)) -> Model {
        Model {
            draw_handler: DrawHandler::new().expect("draw handler"),
            search: ShortestPath::new(Map::new(size.0, size.1), Algorithm::AStar),
            status: SearchStatus::NotFound,
            path: None,
            cursor: Cursor {
                position: (0f64, 0f64),
                button_pressed: false,
                cell: Cell::Passable,
            },
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        // отпровляет сообщение виджету каждые 16 мс
        interval(relm.stream(), 16, || Msg::Next);
    }

    // обработка сообщений виджета
    fn update(&mut self, event: Msg) {
        match event {
            Msg::About => {
                let text_instruction = gtk::TextBuffer::new(None);
                text_instruction.set_text(
                    "Для добавления препятствий нажите на белую клетку и\n\
                     не отпуская двигайте мышкой в нужном направлении.\n\n\
                     Для удаление препятствий нажите на серую клетку и\n\
                     не отпуская двигайте мышкой по препятствиям.\n\n\
                     Перетащите зеленую клетку, чтобы установить начальную позицию.\n\n\
                     Перетащите красную клетку, чтобы установить конечную позицию."
                );
                let view_instruction = gtk::TextView::new_with_buffer(&text_instruction);

                let text_algorithms = gtk::TextBuffer::new(None);
                text_algorithms.set_text(
                    "Обход в ширину\n\
                     Один из простейших алгоритмов обхода графа, являющийся основой\n\
                     для многих важных алгоритмов для работы с графами.\n\n\
                     Алгоритм Дейкстры\n\
                     В ориентированном взвешенном графе G=(V,E), вес рёбер которого\n\
                     неотрицателен и определяется весовой функцией W:E → R, алгоритмт\n\
                     Дейкстры находит длины кратчайших путей из заданной вершины S\n\
                     до всех остальных.\n\n\
                     А*\n\
                     Алгоритм поиска, который находит во взвешенном графе маршрут\n\
                     наименьшей стоимости от начальной вершины до выбранной конечной.",
                );
                let view_algorithms = gtk::TextView::new_with_buffer(&text_algorithms);

                let text_about = gtk::TextBuffer::new(None);
                text_about.set_text(
                    "Программа «Поиск кратчайшего пути» предназначена для\n\
                     демонстрации работы алгоритмов поиска кратчашего пути\n\
                     в известных дискретных средах.\n\n\
                     Автор: студент группы ИП-16-3 Веселков C.Д."
                );
                let view_about = gtk::TextView::new_with_buffer(&text_about);

                let notebook = gtk::Notebook::new();
                notebook.append_page(
                    &view_about,
                    Some(&gtk::Label::new(Some("О программе"))),
                );
                notebook.append_page(
                    &view_instruction,
                    Some(&gtk::Label::new(Some("Инструкция"))),
                );
                notebook.append_page(
                    &view_algorithms,
                    Some(&gtk::Label::new(Some("Алгоритмы"))),
                );
                let dialog = gtk::Dialog::new_with_buttons(
                    Some("Справка"),
                    Some(&self.window),
                    gtk::DialogFlags::MODAL,
                    &[("Закрыть", gtk::ResponseType::Close.into())],
                );
                dialog.set_default_response(gtk::ResponseType::Close.into());
                dialog.connect_response(|dialog, _| dialog.destroy());

                let dialog_box = dialog.get_content_area();
                dialog_box.add(&notebook);
                dialog.show_all();
            }
            Msg::AlgorithmChange => {
                let map = self.model.search.map.clone();
                match self.combo_box.get_active() {
                    0 => self.model.search = ShortestPath::new(map, Algorithm::BreadthFirstSearch),
                    1 => self.model.search = ShortestPath::new(map, Algorithm::Dijkstra),
                    _ => self.model.search = ShortestPath::new(map, Algorithm::AStar),
                }
            }
            Msg::ButtonPress => {
                self.model.cursor.button_pressed = true;
                let pos = self.get_cursor_pos();
                self.model.cursor.cell = match self.model.search.map[pos.x][pos.y] {
                    Cell::Passable => Cell::Impassable,
                    Cell::Impassable => Cell::Passable,
                    c => c,
                };
                self.model.search.map[pos.x][pos.y] = self.model.cursor.cell;
            }
            Msg::ButtonRelease => self.model.cursor.button_pressed = false,
            Msg::Clear => {
                self.model.search.map.clear();
            }
            Msg::ClearPath => {
                self.search_path_button.show();
                self.clear_path_button.hide();
                self.drawing_area.set_sensitive(true);
                self.combo_box.set_sensitive(true);
                self.save_button.set_sensitive(true);
                self.open_button.set_sensitive(true);
                self.clear_button.set_sensitive(true);
                self.model.path = None;
                self.model.search.map.clear_path();
                // сообщения Msg::Next не будут обрабатываться
                self.model.status = SearchStatus::NotFound;
                self.label.set_text("Длина пути:");
            }
            Msg::FindPath => {
                // инициализация поиска
                self.model.search.init();
                self.search_path_button.hide();
                self.clear_path_button.show();
                self.drawing_area.set_sensitive(false);
                self.combo_box.set_sensitive(false);
                self.save_button.set_sensitive(false);
                self.open_button.set_sensitive(false);
                self.clear_button.set_sensitive(false);
                // сообщения Msg::Next будут обрабатываться в соотвествии subscriptions
                self.model.status = SearchStatus::Searching;
            }
            Msg::MoveCursor(pos) => {
                self.model.cursor.position = pos;
                if self.model.cursor.button_pressed {
                    let pos = self.get_cursor_pos();
                    self.model.search.map.set_cell(self.model.cursor.cell, pos);
                }
            }
            Msg::Next => {
                if self.model.status == SearchStatus::Searching {
                    match self.model.search.next() {
                        SearchStatus::Found(len) => {
                            self.model.status = SearchStatus::Found(len);
                            self.model.path = self.model.search.path();
                            self.label
                                .set_text(format!("Длина пути: {:.2}", len).as_str());
                        }
                        status => self.model.status = status,
                    }
                }
            }
            Msg::Open => {
                let file_chooser = gtk::FileChooserNative::new(
                    Some("Загрузить карту"),
                    Some(&self.window),
                    gtk::FileChooserAction::Open,
                    Some("Открыть"),
                    Some("Отменить"),
                );
                if file_chooser.run() == gtk::ResponseType::Accept.into() {
                    let vec = try_message!(fs::read(file_chooser.get_filename().unwrap()));
                    self.model
                        .search
                        .map
                        .replace_from(&try_message!(deserialize(&vec)));
                    self.success_message("Карта загружена");
                }
            }
            Msg::Quit => gtk::main_quit(),
            Msg::Save => {
                let file_chooser = gtk::FileChooserNative::new(
                    Some("Сохранить карту"),
                    Some(&self.window),
                    gtk::FileChooserAction::Save,
                    Some("Сохранить"),
                    Some("Отменить"),
                );
                if file_chooser.run() == gtk::ResponseType::Accept.into() {
                    let vec = try_message!(serialize(&self.model.search.map));
                    try_message!(fs::write(file_chooser.get_filename().unwrap(), vec));
                    self.success_message("Карта сохранена");
                }
            }
            // сообщение отрисовки
            Msg::UpdateDrawBuffer => {
                // размер карты
                let allocation = self.drawing_area.get_allocation();
                // контекст для рисования
                let context = self.model.draw_handler.get_context();
                context.rectangle(
                    0f64,
                    0f64,
                    f64::from(allocation.width),
                    f64::from(allocation.height),
                );
                context.set_source_rgb(0.0, 0.0, 0.0);
                context.fill();

                let cell_width = f64::from(allocation.width) / self.model.search.map.cols() as f64;
                let cell_height =
                    f64::from(allocation.height) / self.model.search.map.rows() as f64;

                // отрисовка карты
                let border = 1f64;
                for i in 0..self.model.search.map.rows() {
                    for j in 0..self.model.search.map.cols() {
                        let color = match self.model.search.map[i][j] {
                            Cell::Passable => Color::white(),
                            Cell::Impassable => Color::grey(),
                            Cell::Start => Color::green(),
                            Cell::Finish => Color::red(),
                            Cell::Visited => Color::pale_blue(),
                            Cell::InQueue => Color::pale_green(),
                        };
                        context.set_source_rgb(color.red, color.green, color.blue);
                        context.rectangle(
                            j as f64 * cell_width + border,
                            i as f64 * cell_height + border,
                            cell_width - 2f64 * border,
                            cell_height - 2f64 * border,
                        );
                        context.fill();
                    }
                }

                // отрисовка пути
                if let Some(ref path) = self.model.path {
                    if path.len() > 1 {
                        let color = Color::yellow();
                        context.set_line_width(3f64);
                        context.set_source_rgb(color.red, color.green, color.blue);
                        context.move_to(
                            (path[0].y as f64 + 0.5f64) * cell_width,
                            (path[0].x as f64 + 0.5f64) * cell_height,
                        );
                        for c in path.iter().skip(1) {
                            context.line_to(
                                (c.y as f64 + 0.5f64) * cell_width,
                                (c.x as f64 + 0.5f64) * cell_height,
                            );
                        }
                        context.stroke();
                    }
                }
            }
        }
    }

    view! {
        // главное окно
        #[name="window"]
        gtk::Window {
            title: "Поиск кратчайшего пути",
            // сетка, в которой размещены остальыне виджеты
            gtk::Grid {
                column_homogeneous: true,
                row_homogeneous: true,
                column_spacing: 4,
                row_spacing: 2,
                // область для рисования карты
                #[name="drawing_area"]
                gtk::DrawingArea {
                    cell: {
                        left_attach: 0,
                        top_attach: 0,
                        width: 32,
                        height: 18,
                    },
                    // обработка событий области
                    draw(_, _) => (Msg::UpdateDrawBuffer, Inhibit(false)),
                    motion_notify_event(_, event) => (Msg::MoveCursor(event.get_position()), Inhibit(false)),
                    button_press_event(_, _) => (Msg::ButtonPress, Inhibit(false)),
                    button_release_event(_, _) => (Msg::ButtonRelease, Inhibit(false)),
                },
                #[name="save_button"]
                gtk::Button {
                    label: "Сохранить карту",
                    cell: {
                        left_attach: 0,
                        top_attach: 18,
                        width: 4,
                        height: 1,
                    },
                    clicked => Msg::Save,
                },
                #[name="open_button"]
                gtk::Button {
                    label: "Загрузить карту",
                    cell: {
                        left_attach: 0,
                        top_attach: 19,
                        width: 4,
                        height: 1,
                    },
                    clicked => Msg::Open,
                },
                #[name="clear_button"]
                gtk::Button {
                    label: "Очистить карту",
                    cell: {
                        left_attach: 4,
                        top_attach: 18,
                        width: 4,
                        height: 1,
                    },
                    clicked => Msg::Clear,
                },
                gtk::Button {
                    label: "?",
                    cell: {
                        left_attach: 21,
                        top_attach: 18,
                        width: 2,
                        height: 1,
                    },
                    clicked => Msg::About,
                },
                #[name="combo_box"]
                gtk::ComboBoxText {
                    cell: {
                        left_attach: 23,
                        top_attach: 18,
                        width: 5,
                        height: 1,
                    },
                    changed => Msg::AlgorithmChange,
                },
                #[name="search_path_button"]
                gtk::Button {
                    label: "Поиск пути",
                    cell: {
                        left_attach: 28,
                        top_attach: 18,
                        width: 4,
                        height: 1,
                    },
                    clicked => Msg::FindPath,
                },
                #[name="clear_path_button"]
                gtk::Button {
                    label: "Очистить путь",
                    cell: {
                        left_attach: 28,
                        top_attach: 18,
                        width: 4,
                        height: 1,
                    },
                    visible: false,
                    clicked => Msg::ClearPath,
                },
                #[name="label"]
                gtk::Label {
                    text: "Длина пути:",
                    xalign: 0f32,
                    cell: {
                        left_attach: 23,
                        top_attach: 19,
                        width: 9,
                        height: 1,
                    },
                }
            },
            delete_event(_, _) => (Msg::Quit, Inhibit(false)),
        }
    }
}
