use std::{sync::mpsc, thread};

use windows_volume_control::{AudioController, CoinitMode};
use stoppable_thread;
mod functions;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ValorantMusicControllerApp {
    #[serde(skip)]
    audio_controller: AudioController,
    #[serde(skip)]
    process_list: Vec<String>,
    #[serde(skip)]
    selected_process_index: i8,
    #[serde(skip)]
    initial_process_volume: f32,

    #[serde(skip)]
    button_enabled: bool,
    #[serde(skip)]
    button_label: String,
    #[serde(skip)]
    program_active: bool,
    #[serde(skip)]
    simulation_checked: bool,

    #[serde(skip)]
    about_clicked: bool,
    #[serde(skip)]
    instructions_clicked: bool,

    #[serde(skip)]
    program_thread: Option<stoppable_thread::StoppableHandle<()>>,
    #[serde(skip)]
    receiver: Option<mpsc::Receiver<()>>,

    volumes: [u8; 3],
}

impl Default for ValorantMusicControllerApp {
    fn default() -> Self {
        Self {
            audio_controller: unsafe {
                let mut controller = AudioController::init(Some(CoinitMode::ApartmentThreaded));
                controller.GetSessions();
                controller.GetDefaultAudioEnpointVolumeControl();
                controller.GetAllProcessSessions();
                controller
            },
            process_list: unsafe {
                get_process_list(&mut AudioController::init(Some(CoinitMode::ApartmentThreaded)))
            },
            selected_process_index: -1,
            initial_process_volume: 0.0,

            button_enabled: false,
            button_label: "Select a process".to_owned(),
            program_active: false,
            simulation_checked: false,

            about_clicked: false,
            instructions_clicked: false,

            program_thread: None,
            receiver: None,

            volumes: [100, 50, 0],
        }
    }
}

impl ValorantMusicControllerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for ValorantMusicControllerApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let sim_dialog = create_dialog(
            ctx,
            "dialog_simulation".to_owned(),
            "Simulation finished".to_owned(),
            "The simulation has finished. Please, check if the volume has changed correctly and if the media has been paused/resumed correctly.".to_owned()
        );
        let how_sim_works_dialog = create_dialog(
            ctx,
            "dialog_how_simulation_works".to_owned(),
            "How simulation works?".to_owned(),
            "Simulating a match does not require Valorant to be opened. Every second, the simulation will change the volume using this state template: Not in game -> Buy phase -> Round started -> Buy phase -> Match ended.".to_owned()
        );

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("How to use?").clicked() {
                            self.instructions_clicked = true;
                        }
                        if ui.button("About").clicked() {
                            self.about_clicked = true;   
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_switch(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("grid_primary").min_col_width(0.0).show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 10.0);
                    ui.heading("Volume control");
                    ui.style_mut().spacing.item_spacing = egui::vec2(7.5, 8.0);
                    ui.add_enabled(!self.program_active, egui::Slider::new(&mut self.volumes[0], 1..=100).max_decimals(0).text("Not in game").custom_formatter(|value, _| format!("{}%", value)));
                    ui.add_enabled(!self.program_active, egui::Slider::new(&mut self.volumes[1], 0..=100).max_decimals(0).text("In game: Buy phase").custom_formatter(|value, _| format!("{}%", value)));
                    ui.add_enabled(!self.program_active, egui::Slider::new(&mut self.volumes[2], 0..=100).max_decimals(0).text("In game: Playing").custom_formatter(|value, _| format!("{}%", value)));
                });
                ui.add(egui::Separator::default().vertical());
                ui.vertical(|ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 10.0);
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
                        for i in 0..self.process_list.len() as i8 {
                            body.row(20.0, |mut row| {
                                let mut label_clicked = false;
                                row.set_selected(self.selected_process_index == i);
                                row.col(|ui| {
                                    let label = ui.add_sized(ui.available_size(), egui::Label::new(self.process_list[i as usize].clone()).sense(egui::Sense::click()));
                                    label_clicked = label.clicked();
                                    label.on_hover_cursor(egui::CursorIcon::PointingHand);
                                });
                                
                                row.response().on_hover_cursor(egui::CursorIcon::PointingHand);
                                if row.response().clicked() || label_clicked {
                                    if !self.program_active {
                                        self.selected_process_index = i;
                                        self.button_enabled = true;
                                        self.button_label = get_activate_button_label(self.simulation_checked);
                                        self.initial_process_volume = unsafe { self.audio_controller.get_session_by_name(self.process_list[i as usize].clone()).unwrap().getVolume() };
                                    }
                                }
                            });
                        }
                    });

                    if ui.add_sized((ui.available_width(), 0.0), egui::Button::new("Update process list")).clicked() {
                        if !self.program_active {
                            self.audio_controller = unsafe { AudioController::init(Some(CoinitMode::ApartmentThreaded)) };
                            self.process_list = get_process_list(&mut self.audio_controller);
                            self.selected_process_index = -1;
                            self.button_enabled = false;
                            self.button_label = "Select a process".to_owned();
                        }
                    }
                });
                ui.end_row();
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                let sim_label = ui.add(egui::Label::new("How simulation works?").sense(egui::Sense::click()));
                sim_label.clone().on_hover_cursor(egui::CursorIcon::PointingHand);
                if sim_label.clicked() {
                    how_sim_works_dialog.open();
                }

                if ui.checkbox(&mut self.simulation_checked, "Simulate test").clicked() {
                    // Si el programa está preparado para simular y se pulsa sobre la checkbox, se fuerza el cambio del texto del botón
                    if self.button_label == "Activate program" || self.button_label == "Simulate a match" {
                        self.button_label = get_activate_button_label(self.simulation_checked);
                    }
                }
                
                if ui.add_enabled(self.button_enabled, egui::Button::new(self.button_label.clone()).min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                    self.program_active = !self.program_active;
                    if self.program_active { // Se activó el programa
                        if self.simulation_checked {
                            let process_name = self.process_list[self.selected_process_index as usize].clone();
                            let volumes = self.volumes.clone();
                            
                            let (tx, rx) = mpsc::channel(); // Canal para comunicarse con el hilo secundario
                            self.receiver = Some(rx); // Guardamos el receptor

                            self.button_label = "Simulating...".to_owned();
                            self.button_enabled = false;

                            thread::spawn(move || {
                                functions::simulate_match(process_name, volumes);
                                tx.send(()).unwrap();
                            });
                        }
                        else {
                            let process_name = self.process_list[self.selected_process_index as usize].clone();
                            let volumes = self.volumes.clone();
                            self.program_thread = Some(stoppable_thread::spawn(move |should_stop| { // Crear un nuevo hilo para ejecutar el programa
                                functions::main_function(should_stop, process_name, volumes);
                            }));
                            self.button_label = "Stop program".to_owned();
                        }
                    }
                    else { // Se desactivó el programa
                        self.button_label = "Stopping program...".to_owned();
                        self.button_enabled = false;
                        self.program_thread.take().unwrap().stop().join().unwrap(); // Esperar a que el hilo termine
                        self.button_label = get_activate_button_label(self.simulation_checked);
                        self.button_enabled = true;
                        unsafe { // Restaurar el volumen del proceso
                            self.audio_controller.get_session_by_name(self.process_list[self.selected_process_index as usize].clone()).unwrap().setVolume(self.initial_process_volume);
                        }
                    }
                }
            });
        });

        egui::Window::new("About")
        .collapsible(false)
        .resizable(false)
        .open(&mut self.about_clicked)
        .show(ctx, |ui| {
            ui.label("Valorant Music Controller - By Sauleteh");
            ui.add_space(8.0);
            ui.label("Automatically pause/play and control the volume of your music depending on the state of the game you are in on Valorant.");
            ui.add_space(8.0);
            ui.add(egui::Hyperlink::from_label_and_url(
                "Source code (GitHub)",
                "https://github.com/Sauleteh/valorant-music-controller-gui",
            ));
        });

        egui::Window::new("How to use?")
        .collapsible(false)
        .resizable(false)
        .open(&mut self.instructions_clicked)
        .show(ctx, |ui| {
            ui.label("1. Select the process of the media player you are using (Firefox, Spotify...).");
            ui.label("2. Adjust the volume to your liking for each state of the game.");
            ui.label("3. Activate the program using the main button.");
            ui.label("4. If already not playing, start playing a video or music.");
            ui.add_space(8.0);
            ui.label("Note: Setting a volume to 0 on a state will pause the media player when this state is reached and will resume it when exiting this state.");
        });

        // Receptor de mensajes del hilo secundario
        if let Some(ref rx) = self.receiver {
            if let Ok(_) = rx.try_recv() { // Si el hilo secundario terminó la simulación...
                self.button_label = get_activate_button_label(self.simulation_checked);
                self.button_enabled = true;
                self.program_active = false;
                unsafe { self.audio_controller.get_session_by_name(self.process_list[self.selected_process_index as usize].clone()).unwrap().setVolume(self.initial_process_volume); }
                sim_dialog.open();
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if self.selected_process_index != -1 {
            unsafe { // Si se cierra la app con el programa en ejecución, se para el programa y se restaura el volumen del proceso
                if let Some(_) = self.program_thread {
                    self.button_label = "Stopping program...".to_owned();
                    self.button_enabled = false;
                    self.program_thread.take().unwrap().stop().join().unwrap(); // Esperar a que el hilo termine
                }
                self.audio_controller.get_session_by_name(self.process_list[self.selected_process_index as usize].clone()).unwrap().setVolume(self.initial_process_volume);
            }
        }
    }
}

fn get_process_list(controller: &mut AudioController) -> Vec<String> {
    unsafe {
        controller.GetSessions();
        controller.GetDefaultAudioEnpointVolumeControl();
        controller.GetAllProcessSessions();
        return controller.get_all_session_names();
    }
}

// Retorna el nombre del botón cuando está disponible para ser activado; sin embargo, tiene dos posibles nombres, dependiendo de si se quiere simular o no
fn get_activate_button_label(simulating: bool) -> String {
    if simulating { return "Simulate a match".to_owned(); }
    else { return "Activate program".to_owned(); }
}

fn create_dialog(ctx: &egui::Context, id: String, title: String, body: String) -> egui_modal::Modal {
    let dialog = egui_modal::Modal::new(ctx, id);
    dialog.show(|ui| {
        dialog.title(ui, title);
        dialog.frame(ui, |ui| {
            dialog.body(ui, body);
        });
        dialog.buttons(ui, |ui| {
            dialog.button(ui, "Close");
        });
    });

    return dialog;
}