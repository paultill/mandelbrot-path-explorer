use eframe::{egui, App, CreationContext};

struct MandelbrotApp {
    mandelbrot_texture: egui::TextureHandle,
    last_size: [usize; 2],
    last_click: Option<(usize, usize)>,
    last_path: Vec<(f64, f64)>,
}

impl MandelbrotApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        let size = [800, 600];
        let image = render_mandelbrot(size[0], size[1]);
        let mandelbrot_texture = cc.egui_ctx.load_texture(
            "mandelbrot",
            image,
            egui::TextureOptions::default(),
        );
        Self {
            mandelbrot_texture,
            last_size: size,
            last_click: None,
            last_path: Vec::new(),
        }
    }
}

impl App for MandelbrotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mandelbrot Explorer");
            let available = ui.available_size();
            let side = available.x.min(available.y).max(100.0).round() as usize;
            let size = [side, side];
            if size != self.last_size {
                let image = render_mandelbrot(side, side);
                self.mandelbrot_texture.set(image, egui::TextureOptions::default());
                self.last_size = size;
            }
            let image_size = egui::Vec2::new(side as f32, side as f32);
            let offset_x = (available.x - image_size.x) / 2.0;
            let offset_y = (available.y - image_size.y) / 2.0;
            ui.add_space(offset_y.max(0.0));
            ui.horizontal_centered(|ui| {
                ui.add_space(offset_x.max(0.0));
                let image_response = ui.image(&self.mandelbrot_texture)
                    .interact(egui::Sense::click_and_drag());
                // Handle click or drag
                let pointer_pos = if image_response.dragged() || image_response.clicked() {
                    image_response.interact_pointer_pos()
                } else {
                    None
                };
                if let Some(pos) = pointer_pos {
                    let px = (pos.x - offset_x.max(0.0)).clamp(0.0, side as f32 - 1.0) as usize;
                    let py = (pos.y - offset_y.max(0.0)).clamp(0.0, side as f32 - 1.0) as usize;
                    let path = mandelbrot_path(px, py, side, side);
                    self.last_click = Some((px, py));
                    self.last_path = path;
                }
                // Draw the path if available
                if !self.last_path.is_empty() {
                    let painter = ui.painter();
                    let scale = 3.0 / side as f64;
                    let to_screen = |zx: f64, zy: f64| -> egui::Pos2 {
                        let x = ((zx + 2.0) / scale) as f32;
                        let y = ((zy + 1.5) / scale) as f32;
                        egui::pos2(
                            image_response.rect.left() + x,
                            image_response.rect.top() + y,
                        )
                    };
                    for w in self.last_path.windows(2) {
                        let p0 = to_screen(w[0].0, w[0].1);
                        let p1 = to_screen(w[1].0, w[1].1);
                        painter.line_segment([
                            p0,
                            p1,
                        ], egui::Stroke::new(2.0, egui::Color32::YELLOW));
                    }
                }
            });
        });
    }
}

fn render_mandelbrot(width: usize, height: usize) -> egui::ColorImage {
    let mut pixels = Vec::with_capacity(width * height);
    let max_iter = 100;
    // For 1:1 aspect ratio, use a square region in the complex plane
    let scale = 3.0 / width as f64; // covers -2.0..1.0 horizontally, -1.5..1.5 vertically
    for y in 0..height {
        for x in 0..width {
            let cx = x as f64 * scale - 2.0;
            let cy = y as f64 * scale - 1.5;
            let mut zx = 0.0;
            let mut zy = 0.0;
            let mut iter = 0;
            while zx * zx + zy * zy < 4.0 && iter < max_iter {
                let tmp = zx * zx - zy * zy + cx;
                zy = 2.0 * zx * zy + cy;
                zx = tmp;
                iter += 1;
            }
            let color = if iter == max_iter {
                egui::Color32::BLACK
            } else {
                let c = (255.0 * iter as f32 / max_iter as f32) as u8;
                egui::Color32::from_rgb(c, 0, 255 - c)
            };
            pixels.push(color);
        }
    }
    egui::ColorImage {
        size: [width, height],
        pixels,
    }
}

fn mandelbrot_path(px: usize, py: usize, width: usize, height: usize) -> Vec<(f64, f64)> {
    let mut path = Vec::new();
    let scale = 3.0 / width as f64;
    let cx = px as f64 * scale - 2.0;
    let cy = py as f64 * scale - 1.5;
    let mut zx = 0.0;
    let mut zy = 0.0;
    let max_iter = 100;
    for _ in 0..max_iter {
        path.push((zx, zy));
        if zx * zx + zy * zy >= 4.0 {
            break;
        }
        let tmp = zx * zx - zy * zy + cx;
        zy = 2.0 * zx * zy + cy;
        zx = tmp;
    }
    path
}

fn main() -> eframe::Result<()> {
    let mut options = eframe::NativeOptions::default();
    options.viewport = egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]);
    eframe::run_native(
        "Mandelbrot Explorer",
        options,
        Box::new(|cc| Ok(Box::new(MandelbrotApp::new(cc)))),
    )
}
