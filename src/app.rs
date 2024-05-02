use std::{fs::{self, File}, io::{self, BufReader}, process::{Command}, collections::HashMap};
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
    //mp3: Vec<String>,
    mp3: HashMap<String, Vec<String>>,
    project: Vec<String>
}
impl Files {
    fn clear(&mut self) {
        //self.mp3.clear();
        self.mp3.clear();
        self.project.clear();
    }
    fn append_tag(&mut self, key:&String, value:String) {
        if let Some(vec) = self.mp3.get_mut(key) {
            // Append the value to the vector
            vec.push(value.to_string());
        }
    }
    fn get_tags(&self) -> Vec<String> {
        let all_tags = self.mp3.values().cloned().flatten().collect::<Vec<String>>();
        let mut tags:Vec<String> = vec![];
        for tag in all_tags {
            if !tags.contains(&tag) && tag != "" {
                tags.push(tag);
            }
        }
        tags
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MainApp {
    path: String,
    files: Files,
    global_tags: Vec<String>,
    /// Additional windows
    #[serde(skip)]
    show_error: (bool, String ),
    /// Track Variables
    #[serde(skip)]
    files_shown: Vec<String>,
    #[serde(skip)]
    now_playing: String,
    #[serde(skip)]
    now_tags: String,
    // Track options
    _scan_on_open: bool,
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
                //mp3: vec![],
                mp3: HashMap::new(),
                project: vec![]
            },
            // Initialize additional windows
            show_error: (false, "".to_string()),
            // Files Shown
            files_shown: vec![],
            now_playing: String::from(""),
            now_tags: String::from(""),
            global_tags: vec![],
            // Settings
            _scan_on_open: true,
            // Music Playing sounds || Doesn't matter, gets updated in the new function
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
        }
        // Initialize the global sink for playing sounds
        (app.stream, app.stream_handle) = OutputStream::try_default().unwrap();
        app.sink = Sink::try_new(&app.stream_handle).unwrap();
        /// Whenever the app opens, re-scan the files in the directory, if the option is set
        if app._scan_on_open {
            get_files(&app.path, &mut app.files, &mut app.files_shown).unwrap_or_else(|e| {
                match e {
                    Error::FsError(e) => {
                        app.show_error.0 = true;
                        app.show_error.1 = e.to_string();
                    },
                    Error::Other(e) => println!("{}", e),
                    _ => {}
                }
            });
            app.global_tags = app.files.get_tags();
        }
        // Initialize the list of mp3's
        //app.files_shown = app.files.mp3.clone();
        println!("tags: {:?}", app.global_tags);
        app.files_shown = app.files.mp3.keys().map(|s| s.to_string()).collect();
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
                        }
                        /*else if ui.button("Scan Files").clicked() {
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
                            self.now_playing = String::from("");
                        }
                    });
                }
            });
        });
        egui::SidePanel::right("SideBar").exact_width(168.0).show(ctx, |ui| {
            let mut title_track = "";
            if &self.now_playing != "" {
                title_track = &self.now_playing;
            } else {
                title_track = "No Track";
            }
            ui.heading(title_track);
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("See MP3").clicked() {
                    if self.now_playing != "" {
                        let file_path = format!("{}/{}.mp3", self.path, &self.now_playing);
                        match Command::new("open").arg("-R").arg(file_path).output() {
                            Ok(_) => {},
                            Err(e) => panic!("Fuck")
                        };
                    }
                };
                if ui.button("Open project").clicked() {
                    if self.now_playing != "" {
                        let mut project:Option<String> = None;
                        for file in &self.files.project {
                            if file.contains(&format!("{}.", &self.now_playing)) {
                                project = Some(String::from(file));
                                break;
                            }
                        }
                        if let Some(file) = project {
                            let file_path = format!("{}/{}", self.path, file);
                            match Command::new("open").arg("-R").arg(file_path).output() {
                                Ok(_) => {},
                                Err(e) => panic!("Fuck")
                            };
                        } else {
                            // Prompt the error box
                            panic!("No project file for that")
                        }
                    }
                };
            });
            ui.add_space(4.0);
            ui.heading("Tags:");
            let tag_box = ui.add(egui::TextEdit::multiline(&mut self.now_tags)
                .desired_width(145.0));
            if tag_box.lost_focus() && self.now_playing != "" {
                //self.files.append_tag(&self.now_playing, )
                let mut tag_list:Vec<String> = self.now_tags.split("\n").map(|s| s.to_string()).collect();
                for mut tag in &tag_list {
                    tag = &tag.trim().to_string();
                }
                self.files.mp3.insert(String::from(&self.now_playing), tag_list);
                self.global_tags = self.files.get_tags();
            }
            /*ui.text_edit_singleline(&mut self.path);
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
            ui.separator();*/

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
                                        self.now_playing = String::from(&self.files_shown[i-1]);
                                        self.now_tags = self.files.mp3.get(&self.files_shown[i-1]).unwrap().join("\n");
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
    //file_list.clear();
    if path == "" {
        return Err(Error::Other("Directory is empty"));
    }
    // Possible fail if this fails due to the error propagation here because I'm returning my custom error type
    let files = fs::read_dir(path).map_err(Error::FsError)?;
    let mut file_vector:Vec<String> = vec![];
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
                    file_vector.push(String::from(name));
                    match ext {
                        "mp3" => if !file_list.mp3.contains_key(&name.to_string()) {
                                file_list.mp3.insert(name.to_string(), vec![]);
                            },
                        // Logic | FL Studio | Ableton | Musescore | Reaper | Cubase | Pro Tools
                        "logicx" | "flp" | "als" | "mscz" | "rpp" | "cpr" | "ptx" => file_list.project.push(file_name),
                        _ => {}
                    }
                }
            }
        }
    }
    file_list.mp3.retain(|title, _| file_vector.contains(title));
    *files_shown = file_list.mp3.keys().map(|s| String::from(s)).collect();
    Ok(())
}