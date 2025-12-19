use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

// Window (slightly taller for text area)
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

// Autosave interval (3 seconds)
const AUTOSAVE_INTERVAL_SECS: u64 = 3;

// Colors
const BG_COLOR: (u8, u8, u8) = (30, 30, 30);
const CLOSE_BTN_COLOR: (u8, u8, u8) = (150, 150, 150);
const CLOSE_BTN_HOVER_COLOR: (u8, u8, u8) = (255, 100, 100);
const INFO_BTN_HOVER_COLOR: (u8, u8, u8) = (100, 150, 255);
const PIN_BTN_COLOR: (u8, u8, u8) = (150, 150, 150);
const PIN_BTN_ACTIVE_COLOR: (u8, u8, u8) = (255, 255, 255);

// Links
const GITHUB_URL: &str = "https://github.com/kartikay-bagla/scratchpad";

fn get_scratchpad_path() -> PathBuf {
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

fn main() -> eframe::Result<()> {
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

    eframe::run_native(
        "Scratchpad",
        options,
        Box::new(|_cc| Ok(Box::new(ScratchpadApp::default()))),
    )
}

struct ScratchpadApp {
    text_content: String,
    show_info: bool,
    always_on_top: bool,
    last_save_time: Instant,
    content_changed: bool,
    last_saved_content: String,
}

impl Default for ScratchpadApp {
    fn default() -> Self {
        let path = get_scratchpad_path();
        let text_content = fs::read_to_string(&path).unwrap_or_default();
        let last_saved_content = text_content.clone();

        Self {
            text_content,
            show_info: false,
            always_on_top: false,
            last_save_time: Instant::now(),
            content_changed: false,
            last_saved_content,
        }
    }
}

impl ScratchpadApp {
    fn save_to_file(&mut self) {
        let path = get_scratchpad_path();
        if let Err(e) = fs::write(&path, &self.text_content) {
            eprintln!("Failed to save scratchpad: {}", e);
        } else {
            self.last_saved_content = self.text_content.clone();
            self.content_changed = false;
            self.last_save_time = Instant::now();
        }
    }
}

impl eframe::App for ScratchpadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaints for autosave checking
        ctx.request_repaint();

        // Check for autosave
        if self.content_changed
            && self.last_save_time.elapsed().as_secs() >= AUTOSAVE_INTERVAL_SECS
        {
            self.save_to_file();
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
                // Set appropriate cursor icon for resize direction
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
                    // Filled white circle when pinned
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
                    // Unfilled circle (just stroke)
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
                    // Save before closing
                    if self.content_changed {
                        self.save_to_file();
                    }
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
                            ui.label("Autosaves to ~/.scratchpad.txt");
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

                // Draw title text in the center of the title bar
                ui.painter().text(
                    egui::pos2(ui.max_rect().center().x, ui.max_rect().top() + title_bar_height / 2.0),
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

                let response = ui.allocate_new_ui(egui::UiBuilder::new().max_rect(padded_rect), |ui| {
                    let text_edit = egui::TextEdit::multiline(&mut self.text_content)
                        .desired_width(f32::INFINITY)
                        .frame(false)
                        .font(egui::FontId::monospace(14.0))
                        .text_color(egui::Color32::WHITE);

                    ui.add_sized(ui.available_size(), text_edit)
                }).inner;

                // Track content changes
                if response.changed() {
                    self.content_changed = self.text_content != self.last_saved_content;
                }
            });
    }
}
