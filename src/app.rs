use std::{fs::{self, File}, io::{self, BufReader}};
use rodio::{Decoder, OutputStream, Sink, OutputStreamHandle};
use std::thread;
use egui::vec2;
use rodio::decoder::DecoderError;

enum Error<'e> {
    FsError(io::Error),
    PlayError,
    Other(& 'e str)
}
#[derive(serde::Deserialize, serde::Serialize)]
struct Files {
    mp3: Vec<String>,
    project: Vec<String>
}
impl Files {
    fn clear(&mut self) {
        self.mp3.clear();
        self.project.clear();
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MainApp {
    path: String,
    files: Files,
    /// Additional windows
    #[serde(skip)]
    show_error: (bool, String ),
    /// Track Variables
    #[serde(skip)]
    files_shown: Vec<String>,
    /// Audio Playback
    #[serde(skip)]
    stream: OutputStream,
    #[serde(skip)]
    stream_handle: OutputStreamHandle,
    #[serde(skip)]
    sink: Sink
}
impl Default for MainApp {
    fn default() -> Self {
        Self {
            path: String::from(""),
            files: Files {
                mp3: vec![],
                project: vec![]
            },
            // Initialize additional windows
            show_error: (false, "".to_string()),
            // Files Shown
            files_shown: vec![],
            stream: {
                let (stream, _) = OutputStream::try_default().unwrap();
                stream
            },
            stream_handle: {
                let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                stream_handle
            },
            sink: {
                let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                let sink = Sink::try_new(&stream_handle).unwrap();
                sink
            }
        }
    }
}
const IS_WEB: bool = cfg!(target_arch = "wasm32");
impl MainApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app:Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        if app.path.ends_with("/") {
            app.path = String::from(&app.path[0..app.path.len()-1]);
            println!("{}", app.path);
        }
        //let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        (app.stream, app.stream_handle) = OutputStream::try_default().unwrap();
        app.sink = Sink::try_new(&app.stream_handle).unwrap();
        // Initialize the list of mp3's
        app.files_shown = app.files.mp3.clone();
        app
    }
}
impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if !IS_WEB {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        } /*else if ui.button("Scan Files").clicked() {
                            get_files(&self.path, &mut self.files, &mut self.files_shown).unwrap_or_else(|e| {
                                match e {
                                    Error::FsError(e) => {
                                        self.show_error.0 = true;
                                        self.show_error.1 = e.to_string();
                                    },
                                    Error::Other(e) => println!("{}", e),
                                    _ => {}
                                }
                            });
                        }*/
                    });
                    ui.menu_button("Sound", |ui| {
                        if ui.button("Play").clicked() {
                            self.sink.play();
                        } else if ui.button("Pause").clicked() {
                           self.sink.pause();
                       } else if ui.button("Clear").clicked() {
                            self.sink.clear();
                        }
                    });
                }
            });
        });
        egui::SidePanel::left("SideBar").exact_width(150.0).show(ctx, |ui| {
            ui.label("Directory");
            ui.text_edit_singleline(&mut self.path);
            if ui.add(egui::Button::new("Scan").min_size(vec2(150.0, 20.0))).clicked() {
                get_files(&self.path, &mut self.files, &mut self.files_shown).unwrap_or_else(|e| {
                    match e {
                        Error::FsError(e) => {
                            self.show_error.0 = true;
                            self.show_error.1 = e.to_string();
                        },
                        Error::Other(e) => println!("{}", e),
                        _ => {}
                    }
                });
            }
            ui.separator();
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("MainGrid").num_columns(5).min_col_width(104.5).max_col_width(104.5)
                .min_row_height(104.5).spacing(vec2(8.0, 8.0))
                .show(ui, |ui| {
                for i in 1..self.files_shown.len()+1 {
                    ui.vertical(|ui| {
                        let mut play = ui.add(egui::Button::image(egui::Image::new(egui::include_image!("../assets/Audio wave icon.png"))));
                        if play.clicked() {
                            match File::open(format!("{}/{}.mp3", self.path, &self.files_shown[i-1])) {
                                Ok(file) => match Decoder::new(file) {
                                    Ok(source) => {
                                        self.sink.clear();
                                        self.sink.append(source);
                                        self.sink.play();
                                    }
                                    Err(e) => self.show_error = (true, e.to_string())
                                },
                                Err(e) => self.show_error = (true, e.to_string())
                            }
                        }
                        ui.label(&self.files_shown[i-1]);
                    });
                    if i%5 == 0{
                        ui.end_row();
                    }
                }
            });
        });
        // Show the error window with its error message
        if self.show_error.0 {
            self.files.clear();
            self.files_shown.clear();
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("immediate_viewport"),
                egui::ViewportBuilder::default()
                    .with_title("Error")
                    .with_inner_size([200.0, 100.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "This egui backend doesn't support multiple viewports"
                    );
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let error_msg = self.show_error.1.as_str();
                        ui.label(error_msg);
                    });
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_error.0 = false;
                    }
                },
            );
        }
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

fn get_files<'e>(path:&str, file_list: &mut Files, files_shown: &mut Vec<String>) -> Result<(),Error<'e>> {
    file_list.clear();
    if path == "" {
        return Err(Error::Other("Directory is empty"));
    }
    // Possible fail if this fails due to the error propagation here because I'm returning my custom error type
    let files = fs::read_dir(path).map_err(Error::FsError)?;
    for file in files {
        if let Ok(file) = file {
            if let Ok(file_name) = file.file_name().into_string() {
                if !file_name.starts_with(".") { // Don't show hidden files
                    let (name, ext) = match file_name.rsplitn(2, ".") {
                        mut split_iter => {
                            let ext = split_iter.next().unwrap_or("");
                            let name = split_iter.next().unwrap_or("");
                            (name, ext)
                        }
                    };
                    match ext {
                        "mp3" => file_list.mp3.push(String::from(name)),
                        // Logic | FL Studio | Ableton | Musescore | Reaper | Cubase | Pro Tools
                        "logicx" | "flp" | "als" | "mscz" | "rpp" | "cpr" | "ptx" => file_list.project.push(file_name),
                        _ => {}
                    }
                }
            }
        }
    }
    *files_shown = file_list.mp3.clone();
    Ok(())
}