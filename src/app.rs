// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[serde(skip)]
    debug_label: String,

    #[serde(skip)]
    selected_process_index: i8,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            debug_label: "Hello World!".to_owned(),
            selected_process_index: -1,
            value: 2.7,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            egui::Grid::new("grid_primary").min_col_width(0.0).show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Volume control");
                    ui.style_mut().spacing.item_spacing = egui::vec2(7.5, 8.0);
                    ui.add(egui::Slider::new(&mut self.value, 0.01..=1.00).max_decimals(2).text("Not in game"));
                    ui.add(egui::Slider::new(&mut self.value, 0.00..=1.00).max_decimals(2).text("In game: Buy phase"));
                    ui.add(egui::Slider::new(&mut self.value, 0.00..=1.00).max_decimals(2).text("In game: Playing"));
                    ui.add(egui::Slider::new(&mut self.value, 0.00..=1.00).max_decimals(2).text("In game: Dead"));
                });
                ui.add(egui::Separator::default().vertical());
                ui.vertical(|ui| {
                    ui.heading("Process selection");

                    ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 10.0);
                    egui_extras::TableBuilder::new(ui)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(egui_extras::Column::initial(250.0))
                    .striped(true)
                    .sense(egui::Sense::click())
                    .min_scrolled_height(0.0)
                    .max_scroll_height(75.0)
                    .body(|mut body| {
                        let number_of_processes = 30;
                        for i in 0..number_of_processes {
                            body.row(25.0, |mut row| {
                                let mut label_clicked = false;

                                row.set_selected(self.selected_process_index == i);
                                row.col(|ui| {
                                    let label = ui.add_sized(ui.available_size(), egui::Label::new("prueba123456789012345678901234567890").sense(egui::Sense::click()));
                                    label_clicked = label.clicked();
                                    label.on_hover_cursor(egui::CursorIcon::PointingHand);
                                });
                                
                                row.response().on_hover_cursor(egui::CursorIcon::PointingHand);
                                if row.response().clicked() || label_clicked {
                                    self.debug_label = format!("TODO: Row {} clicked", i);
                                    self.selected_process_index = i;
                                }
                            });
                        }
                    });

                    if ui.add_sized((ui.available_width(), 0.0), egui::Button::new("Update process list")).clicked() {
                        self.debug_label = "TODO: Update process list".to_string();
                    }
                    
                });
                ui.end_row();
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label(self.debug_label.clone());
                ui.checkbox(&mut false, "Simulate test");
                if ui.add_sized((ui.available_width(), 30.0), egui::Button::new("Activate")).clicked() {
                    self.debug_label = "TODO: Button activated".to_string();
                }
            });
        });
    }
}

/* TODO list:
 * - [X] Aplicar el diseño pensado
 * - [ ] Adaptar el código del programa CLI a la interfaz gráfica
 *     - [ ] El proceso ahora se obtiene de la lista de procesos, al seleccionar uno, el botón de activar programa se pone enabled (disabled por defecto)
 *         - [ ] Hay un botón para actualizar la lista de procesos
 *     - [ ] El handler de CTRL+C ahora se elimina y el código que tenía ahora se ejecuta al salir del programa
 *     - [ ] La funcionalidad principal ahora se ejecuta al hacer click en el botón de activar programa, no al abrir la GUI
 *         - [ ] Al activar el programa, se hace uso de un nuevo hilo para ejecutar el programa ya que es un loop infinito
 *         - [ ] El texto del botón cambia a "Desactivar programa" y al pulsarlo, el hilo nuevo se muere
 *         - [ ] No se puede cambiar de proceso ni cambiar los volúmenes mientras el programa está activo
 *     - [ ] Los volúmenes ahora son editables, recordar que el volumen "NOT_IN_GAME" no puede ser cero
 *     - [ ] El label de los volúmenes a ser posible que se muestre como porcentaje
 * - [ ] El test utilizado en el CLI se implementa como comprobador de que el programa funciona correctamente con una checkbox debajo del botón de activar (o en el menú de opciones)
 *     - [ ] Al pulsar la checkbox, el botón ahora pone simular en vez de activar y al pulsarlo, se ejecuta el test y no se puede detener manualmente por lo que se desactiva el botón
 *     - [ ] En el nombre del botón, mientras se está simulando, se explica en qué estado se está (por ejemplo, "Simulando: Fase de compra")
 *     - [ ] Al terminar el test, sale un dialog explicando que si se ha escuchado como cambiaba el volumen y si se ha pausado/reaunudado correctamente el video/música al tener el volumen a 0, está todo correcto
 * - [ ] En el menú de opciones, añadir una opción para ver los créditos e información (sección "Help")
 * - [ ] Cambiar el icono del programa
 */