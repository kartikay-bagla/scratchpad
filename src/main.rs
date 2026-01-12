use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

// Window
const WINDOW_SIZE: [f32; 2] = [400.0, 250.0];

// Close button
const CLOSE_BTN_SIZE: f32 = 20.0;
const CLOSE_BTN_MARGIN: f32 = 8.0;
const CLOSE_BTN_PADDING: f32 = 4.0;
const CLOSE_BTN_STROKE: f32 = 2.0;

// Resize border
const RESIZE_BORDER: f32 = 6.0;

// Textbox padding
const TEXTBOX_PADDING: f32 = 8.0;

// Checksum poll interval (seconds)
const CHECKSUM_POLL_INTERVAL_SECS: u64 = 2;

// Default config directory name
const DEFAULT_CONFIG_DIR: &str = ".scratchpad";

// Colors
const BG_COLOR: (u8, u8, u8) = (30, 30, 30);
const CLOSE_BTN_COLOR: (u8, u8, u8) = (150, 150, 150);
const CLOSE_BTN_HOVER_COLOR: (u8, u8, u8) = (255, 100, 100);
const INFO_BTN_HOVER_COLOR: (u8, u8, u8) = (100, 150, 255);
const PIN_BTN_COLOR: (u8, u8, u8) = (150, 150, 150);
const PIN_BTN_ACTIVE_COLOR: (u8, u8, u8) = (255, 255, 255);

// Status dot colors
const DOT_UNSAVED_COLOR: (u8, u8, u8) = (200, 200, 200);
const DOT_CONFLICT_COLOR: (u8, u8, u8) = (255, 80, 80);
const DOT_BACKUP_COLOR: (u8, u8, u8) = (255, 200, 80);

// Links
const GITHUB_URL: &str = "https://github.com/kartikay-bagla/scratchpad";

#[derive(Clone, Copy, PartialEq)]
enum StatusDot {
    None,
    Unsaved,
    Conflict,
    BackupFiles,
}

impl StatusDot {
    fn color(&self) -> Option<egui::Color32> {
        match self {
            StatusDot::None => None,
            StatusDot::Unsaved => Some(egui::Color32::from_rgb(
                DOT_UNSAVED_COLOR.0,
                DOT_UNSAVED_COLOR.1,
                DOT_UNSAVED_COLOR.2,
            )),
            StatusDot::Conflict => Some(egui::Color32::from_rgb(
                DOT_CONFLICT_COLOR.0,
                DOT_CONFLICT_COLOR.1,
                DOT_CONFLICT_COLOR.2,
            )),
            StatusDot::BackupFiles => Some(egui::Color32::from_rgb(
                DOT_BACKUP_COLOR.0,
                DOT_BACKUP_COLOR.1,
                DOT_BACKUP_COLOR.2,
            )),
        }
    }
}

fn get_default_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(DEFAULT_CONFIG_DIR)
}

fn get_scratchpad_file_path(config_dir: &PathBuf) -> PathBuf {
    config_dir.join("scratchpad.txt")
}

fn get_old_scratchpad_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".scratchpad.txt")
}

fn load_icon() -> Option<Arc<egui::IconData>> {
    let icon_bytes = include_bytes!("../icon.png");
    let image = image::load_from_memory(icon_bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Some(Arc::new(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    }))
}

fn compute_checksum(content: &str) -> u32 {
    crc32fast::hash(content.as_bytes())
}

fn compute_file_checksum(path: &PathBuf) -> u32 {
    match fs::read_to_string(path) {
        Ok(content) => compute_checksum(&content),
        Err(_) => 0, // File doesn't exist or can't be read
    }
}

/// Check for backup-* and *.sync-conflict-* files in the config directory
fn has_backup_or_conflict_files(config_dir: &PathBuf) -> bool {
    if let Ok(entries) = fs::read_dir(config_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("backup-") && name.ends_with("-scratchpad.txt") {
                    return true;
                }
                if name.contains(".sync-conflict-") {
                    return true;
                }
            }
        }
    }
    false
}

/// Migrate old ~/.scratchpad.txt to new config directory
fn migrate_old_scratchpad(config_dir: &PathBuf) {
    let old_path = get_old_scratchpad_path();
    let new_path = get_scratchpad_file_path(config_dir);

    // Only migrate if old file exists and new file doesn't
    if old_path.exists() && !new_path.exists() {
        // Ensure config directory exists
        if let Err(e) = fs::create_dir_all(config_dir) {
            eprintln!("Failed to create config directory: {}", e);
            return;
        }

        // Move the file
        if let Err(e) = fs::rename(&old_path, &new_path) {
            // If rename fails (cross-device), try copy + delete
            if let Ok(content) = fs::read_to_string(&old_path) {
                if fs::write(&new_path, &content).is_ok() {
                    let _ = fs::remove_file(&old_path);
                }
            } else {
                eprintln!("Failed to migrate scratchpad file: {}", e);
            }
        }
    }
}

fn parse_cli_args() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--config-dir" && i + 1 < args.len() {
            return Some(PathBuf::from(&args[i + 1]));
        }
        if args[i].starts_with("--config-dir=") {
            return Some(PathBuf::from(args[i].trim_start_matches("--config-dir=")));
        }
        i += 1;
    }
    None
}

fn create_backup_filename() -> String {
    let now = chrono::Local::now();
    format!("backup-{}-scratchpad.txt", now.format("%Y%m%d%H%M%S"))
}

fn main() -> eframe::Result<()> {
    // Parse CLI args
    let cli_config_dir = parse_cli_args();
    let config_dir_from_cli = cli_config_dir.is_some();

    // Determine config directory
    let config_dir = cli_config_dir.unwrap_or_else(get_default_config_dir);

    // Ensure config directory exists
    if let Err(e) = fs::create_dir_all(&config_dir) {
        eprintln!("Failed to create config directory: {}", e);
    }

    // Migrate old scratchpad file if needed
    migrate_old_scratchpad(&config_dir);

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(WINDOW_SIZE)
        .with_title("Scratchpad")
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(true);

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let app = ScratchpadApp::new(config_dir, config_dir_from_cli);

    eframe::run_native("Scratchpad", options, Box::new(|_cc| Ok(Box::new(app))))
}

struct ScratchpadApp {
    text_content: String,
    show_info: bool,
    always_on_top: bool,
    content_changed: bool,
    last_saved_content: String,

    // Config
    config_dir: PathBuf,
    config_dir_from_cli: bool,

    // Sync state
    file_checksum: u32,
    conflict_detected: bool,
    backup_files_exist: bool,
    last_poll_time: Instant,

    // Dialogs
    show_overwrite_dialog: bool,
    show_reload_dialog: bool,
}

impl ScratchpadApp {
    fn new(config_dir: PathBuf, config_dir_from_cli: bool) -> Self {
        let scratchpad_path = get_scratchpad_file_path(&config_dir);
        let text_content = fs::read_to_string(&scratchpad_path).unwrap_or_default();
        let file_checksum = compute_checksum(&text_content);
        let backup_files_exist = has_backup_or_conflict_files(&config_dir);

        Self {
            text_content: text_content.clone(),
            show_info: false,
            always_on_top: false,
            content_changed: false,
            last_saved_content: text_content,

            config_dir,
            config_dir_from_cli,

            file_checksum,
            conflict_detected: false,
            backup_files_exist,
            last_poll_time: Instant::now(),

            show_overwrite_dialog: false,
            show_reload_dialog: false,
        }
    }

    fn get_status_dot(&self) -> StatusDot {
        // Priority: Conflict > BackupFiles > Unsaved > None
        if self.conflict_detected {
            StatusDot::Conflict
        } else if self.backup_files_exist {
            StatusDot::BackupFiles
        } else if self.content_changed {
            StatusDot::Unsaved
        } else {
            StatusDot::None
        }
    }

    fn save_to_file(&mut self) -> bool {
        let path = get_scratchpad_file_path(&self.config_dir);

        // Compute fresh checksum to catch race condition
        let current_file_checksum = compute_file_checksum(&path);

        if current_file_checksum != self.file_checksum && current_file_checksum != 0 {
            // Conflict detected!
            self.conflict_detected = true;
            self.show_overwrite_dialog = true;
            return false;
        }

        // No conflict, save normally
        self.do_save()
    }

    fn do_save(&mut self) -> bool {
        let path = get_scratchpad_file_path(&self.config_dir);

        if let Err(e) = fs::write(&path, &self.text_content) {
            eprintln!("Failed to save scratchpad: {}", e);
            return false;
        }

        self.last_saved_content = self.text_content.clone();
        self.content_changed = false;
        self.file_checksum = compute_checksum(&self.text_content);
        self.conflict_detected = false;
        true
    }

    fn save_with_backup(&mut self) -> bool {
        let path = get_scratchpad_file_path(&self.config_dir);

        // Create backup of current file
        if path.exists() {
            let backup_name = create_backup_filename();
            let backup_path = self.config_dir.join(&backup_name);
            if let Err(e) = fs::rename(&path, &backup_path) {
                eprintln!("Failed to create backup: {}", e);
                // Try copy instead
                if let Ok(content) = fs::read_to_string(&path) {
                    let _ = fs::write(&backup_path, content);
                }
            }
        }

        // Now save
        let result = self.do_save();

        // Update backup files status
        self.backup_files_exist = has_backup_or_conflict_files(&self.config_dir);

        result
    }

    fn reload_from_file(&mut self) {
        let path = get_scratchpad_file_path(&self.config_dir);
        self.text_content = fs::read_to_string(&path).unwrap_or_default();
        self.last_saved_content = self.text_content.clone();
        self.content_changed = false;
        self.file_checksum = compute_checksum(&self.text_content);
        self.conflict_detected = false;
    }

    fn poll_for_changes(&mut self) {
        if self.last_poll_time.elapsed().as_secs() >= CHECKSUM_POLL_INTERVAL_SECS {
            let path = get_scratchpad_file_path(&self.config_dir);
            let current_checksum = compute_file_checksum(&path);

            // Only flag conflict if file exists and checksum differs
            if current_checksum != 0 && current_checksum != self.file_checksum {
                self.conflict_detected = true;
            }

            // Also check for backup files
            self.backup_files_exist = has_backup_or_conflict_files(&self.config_dir);

            self.last_poll_time = Instant::now();
        }
    }
}

impl eframe::App for ScratchpadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaints for polling
        ctx.request_repaint();

        // Poll for external file changes
        self.poll_for_changes();

        // Handle keyboard shortcuts
        let mut save_requested = false;
        let mut reload_requested = false;

        ctx.input(|i| {
            // Use `command` so it maps to Ctrl on Windows/Linux and Cmd on macOS
            if i.modifiers.command && i.key_pressed(egui::Key::S) {
                save_requested = true;
            }
            if i.modifiers.command && i.key_pressed(egui::Key::R) {
                reload_requested = true;
            }
        });

        if save_requested && !self.show_overwrite_dialog && !self.show_reload_dialog {
            self.save_to_file();
        }

        if reload_requested && !self.show_overwrite_dialog && !self.show_reload_dialog {
            self.show_reload_dialog = true;
        }

        let bg_color = egui::Color32::from_rgb(BG_COLOR.0, BG_COLOR.1, BG_COLOR.2);

        // Handle window resize from edges/corners
        if let Some(pointer_pos) = ctx.input(|i| i.pointer.hover_pos()) {
            let window_rect = ctx.screen_rect();
            let left = pointer_pos.x - window_rect.left() < RESIZE_BORDER;
            let right = window_rect.right() - pointer_pos.x < RESIZE_BORDER;
            let top = pointer_pos.y - window_rect.top() < RESIZE_BORDER;
            let bottom = window_rect.bottom() - pointer_pos.y < RESIZE_BORDER;

            let resize_direction = match (left, right, top, bottom) {
                (true, false, true, false) => Some(egui::ResizeDirection::NorthWest),
                (false, true, true, false) => Some(egui::ResizeDirection::NorthEast),
                (true, false, false, true) => Some(egui::ResizeDirection::SouthWest),
                (false, true, false, true) => Some(egui::ResizeDirection::SouthEast),
                (true, false, false, false) => Some(egui::ResizeDirection::West),
                (false, true, false, false) => Some(egui::ResizeDirection::East),
                (false, false, true, false) => Some(egui::ResizeDirection::North),
                (false, false, false, true) => Some(egui::ResizeDirection::South),
                _ => None,
            };

            if let Some(direction) = resize_direction {
                let cursor = match direction {
                    egui::ResizeDirection::NorthWest | egui::ResizeDirection::SouthEast => {
                        egui::CursorIcon::ResizeNwSe
                    }
                    egui::ResizeDirection::NorthEast | egui::ResizeDirection::SouthWest => {
                        egui::CursorIcon::ResizeNeSw
                    }
                    egui::ResizeDirection::West | egui::ResizeDirection::East => {
                        egui::CursorIcon::ResizeHorizontal
                    }
                    egui::ResizeDirection::North | egui::ResizeDirection::South => {
                        egui::CursorIcon::ResizeVertical
                    }
                };
                ctx.set_cursor_icon(cursor);

                if ctx.input(|i| i.pointer.primary_pressed()) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(direction));
                }
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(bg_color))
            .show(ctx, |ui| {
                // Title bar region for dragging
                let title_bar_height = CLOSE_BTN_SIZE + CLOSE_BTN_MARGIN * 2.0;
                let close_btn_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        ui.max_rect().right() - CLOSE_BTN_SIZE - CLOSE_BTN_MARGIN,
                        ui.max_rect().top() + CLOSE_BTN_MARGIN,
                    ),
                    egui::vec2(CLOSE_BTN_SIZE, CLOSE_BTN_SIZE),
                );
                let info_btn_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        ui.max_rect().right() - (CLOSE_BTN_SIZE + CLOSE_BTN_MARGIN) * 2.0,
                        ui.max_rect().top() + CLOSE_BTN_MARGIN,
                    ),
                    egui::vec2(CLOSE_BTN_SIZE, CLOSE_BTN_SIZE),
                );
                let pin_btn_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        ui.max_rect().left() + CLOSE_BTN_MARGIN,
                        ui.max_rect().top() + CLOSE_BTN_MARGIN,
                    ),
                    egui::vec2(CLOSE_BTN_SIZE, CLOSE_BTN_SIZE),
                );
                let title_bar_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        ui.max_rect().left() + CLOSE_BTN_SIZE + CLOSE_BTN_MARGIN * 2.0,
                        ui.max_rect().top(),
                    ),
                    egui::vec2(
                        ui.max_rect().width() - (CLOSE_BTN_SIZE + CLOSE_BTN_MARGIN) * 3.0
                            - CLOSE_BTN_MARGIN,
                        title_bar_height,
                    ),
                );

                // Drag window from title bar region (excluding buttons)
                if ui.rect_contains_pointer(title_bar_rect)
                    && ui.input(|i| i.pointer.primary_pressed())
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                // Draw pin button (circle in top-left)
                let pin_hovered = ui.rect_contains_pointer(pin_btn_rect);
                let pin_clicked = pin_hovered && ui.input(|i| i.pointer.primary_clicked());
                let pin_center = pin_btn_rect.center();
                let pin_radius = (CLOSE_BTN_SIZE - CLOSE_BTN_PADDING * 2.0) / 2.0;

                if self.always_on_top {
                    ui.painter().circle_filled(
                        pin_center,
                        pin_radius,
                        egui::Color32::from_rgb(
                            PIN_BTN_ACTIVE_COLOR.0,
                            PIN_BTN_ACTIVE_COLOR.1,
                            PIN_BTN_ACTIVE_COLOR.2,
                        ),
                    );
                } else {
                    let pin_color = if pin_hovered {
                        egui::Color32::from_rgb(
                            PIN_BTN_ACTIVE_COLOR.0,
                            PIN_BTN_ACTIVE_COLOR.1,
                            PIN_BTN_ACTIVE_COLOR.2,
                        )
                    } else {
                        egui::Color32::from_rgb(
                            PIN_BTN_COLOR.0,
                            PIN_BTN_COLOR.1,
                            PIN_BTN_COLOR.2,
                        )
                    };
                    ui.painter().circle_stroke(
                        pin_center,
                        pin_radius,
                        egui::Stroke::new(CLOSE_BTN_STROKE, pin_color),
                    );
                }

                if pin_clicked {
                    self.always_on_top = !self.always_on_top;
                    let level = if self.always_on_top {
                        egui::WindowLevel::AlwaysOnTop
                    } else {
                        egui::WindowLevel::Normal
                    };
                    ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                }

                // Draw info button (i)
                let info_hovered = ui.rect_contains_pointer(info_btn_rect);
                let info_clicked = info_hovered && ui.input(|i| i.pointer.primary_clicked());
                let info_color = if info_hovered {
                    egui::Color32::from_rgb(
                        INFO_BTN_HOVER_COLOR.0,
                        INFO_BTN_HOVER_COLOR.1,
                        INFO_BTN_HOVER_COLOR.2,
                    )
                } else {
                    egui::Color32::from_rgb(
                        CLOSE_BTN_COLOR.0,
                        CLOSE_BTN_COLOR.1,
                        CLOSE_BTN_COLOR.2,
                    )
                };
                let info_center = info_btn_rect.center();
                ui.painter().text(
                    info_center,
                    egui::Align2::CENTER_CENTER,
                    "i",
                    egui::FontId::proportional(14.0),
                    info_color,
                );
                if info_clicked {
                    self.show_info = !self.show_info;
                }

                // Draw close button (x)
                let close_hovered = ui.rect_contains_pointer(close_btn_rect);
                let close_clicked = close_hovered && ui.input(|i| i.pointer.primary_clicked());
                let close_color = if close_hovered {
                    egui::Color32::from_rgb(
                        CLOSE_BTN_HOVER_COLOR.0,
                        CLOSE_BTN_HOVER_COLOR.1,
                        CLOSE_BTN_HOVER_COLOR.2,
                    )
                } else {
                    egui::Color32::from_rgb(
                        CLOSE_BTN_COLOR.0,
                        CLOSE_BTN_COLOR.1,
                        CLOSE_BTN_COLOR.2,
                    )
                };
                ui.painter().line_segment(
                    [
                        close_btn_rect.left_top()
                            + egui::vec2(CLOSE_BTN_PADDING, CLOSE_BTN_PADDING),
                        close_btn_rect.right_bottom()
                            - egui::vec2(CLOSE_BTN_PADDING, CLOSE_BTN_PADDING),
                    ],
                    egui::Stroke::new(CLOSE_BTN_STROKE, close_color),
                );
                ui.painter().line_segment(
                    [
                        close_btn_rect.right_top()
                            + egui::vec2(-CLOSE_BTN_PADDING, CLOSE_BTN_PADDING),
                        close_btn_rect.left_bottom()
                            + egui::vec2(CLOSE_BTN_PADDING, -CLOSE_BTN_PADDING),
                    ],
                    egui::Stroke::new(CLOSE_BTN_STROKE, close_color),
                );
                if close_clicked {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                // Info popup
                if self.show_info {
                    egui::Window::new("About")
                        .collapsible(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                        .show(ctx, |ui| {
                            ui.label("Scratchpad");
                            ui.add_space(5.0);

                            // Show config directory
                            ui.horizontal(|ui| {
                                ui.label("Folder:");
                                let folder_label = if self.config_dir_from_cli {
                                    format!("{} (CLI)", self.config_dir.display())
                                } else {
                                    self.config_dir.display().to_string()
                                };
                                ui.label(folder_label);
                            });

                            // Show backup file warning
                            if self.backup_files_exist {
                                ui.add_space(5.0);
                                ui.colored_label(
                                    egui::Color32::from_rgb(255, 200, 80),
                                    "Backup/conflict files exist in folder",
                                );
                            }

                            ui.add_space(5.0);
                            ui.label("Ctrl+S to save, Ctrl+R to reload");

                            ui.add_space(10.0);
                            if ui.button("View on GitHub").clicked() {
                                let _ = open::that(GITHUB_URL);
                                self.show_info = false;
                            }
                            ui.add_space(10.0);
                            if ui.button("Close").clicked() {
                                self.show_info = false;
                            }
                        });
                }

                // Overwrite confirmation dialog
                if self.show_overwrite_dialog {
                    egui::Window::new("Conflict Detected")
                        .collapsible(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                        .show(ctx, |ui| {
                            ui.label("File changed externally.");
                            ui.label("Overwrite with your changes?");
                            ui.label("(Current file will be backed up)");
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                if ui.button("Yes").clicked() {
                                    self.save_with_backup();
                                    self.show_overwrite_dialog = false;
                                }
                                if ui.button("No").clicked() {
                                    self.show_overwrite_dialog = false;
                                }
                            });
                        });
                }

                // Reload confirmation dialog
                if self.show_reload_dialog {
                    egui::Window::new("Reload")
                        .collapsible(false)
                        .resizable(false)
                        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                        .show(ctx, |ui| {
                            ui.label("Discard local changes and reload?");
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                if ui.button("Yes").clicked() {
                                    self.reload_from_file();
                                    self.show_reload_dialog = false;
                                }
                                if ui.button("No").clicked() {
                                    self.show_reload_dialog = false;
                                }
                            });
                        });
                }

                // Draw status dot and title text
                let title_center_x = ui.max_rect().center().x;
                let title_y = ui.max_rect().top() + title_bar_height / 2.0;
                let status_dot = self.get_status_dot();

                // Draw dot before title if needed
                if let Some(dot_color) = status_dot.color() {
                    let dot_radius = 4.0;
                    let dot_x = title_center_x - 45.0; // Position before "Scratchpad" text
                    ui.painter().circle_filled(
                        egui::pos2(dot_x, title_y),
                        dot_radius,
                        dot_color,
                    );
                }

                ui.painter().text(
                    egui::pos2(title_center_x, title_y),
                    egui::Align2::CENTER_CENTER,
                    "Scratchpad",
                    egui::FontId::proportional(14.0),
                    egui::Color32::from_rgb(CLOSE_BTN_COLOR.0, CLOSE_BTN_COLOR.1, CLOSE_BTN_COLOR.2),
                );

                // Add space for title bar
                ui.add_space(title_bar_height);

                // Text area filling remaining space with padding on all sides
                let available_rect = ui.available_rect_before_wrap();
                let padded_rect = available_rect.shrink(TEXTBOX_PADDING);

                let response = ui
                    .allocate_new_ui(egui::UiBuilder::new().max_rect(padded_rect), |ui| {
                        egui::ScrollArea::vertical()
                            .show(ui, |ui| {
                                let text_edit = egui::TextEdit::multiline(&mut self.text_content)
                                    .desired_width(f32::INFINITY)
                                    .frame(false)
                                    .font(egui::FontId::monospace(14.0))
                                    .text_color(egui::Color32::WHITE);

                                ui.add_sized(ui.available_size(), text_edit)
                            })
                            .inner
                    })
                    .inner;

                // Track content changes
                if response.changed() {
                    self.content_changed = self.text_content != self.last_saved_content;
                }
            });
    }
}
