use eframe::{egui, App, CreationContext};

struct MandelbrotApp {
    mandelbrot_texture: egui::TextureHandle,
    last_size: [usize; 2],
    last_click: Option<(usize, usize)>,
    last_path: Vec<(f64, f64)>,
    center: (f64, f64), // center of view in Mandelbrot space
    scale: f64,         // Mandelbrot units per image width
}

impl MandelbrotApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        let size = [800, 600];
        let image = render_mandelbrot(size[0], size[1], (-0.5, 0.0), 3.0);
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
            center: (-0.5, 0.0), // default Mandelbrot center
            scale: 3.0,          // default Mandelbrot width
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
            let mut zoomed = false;
            let image_size = egui::Vec2::new(side as f32, side as f32);
            let offset_x = (available.x - image_size.x) / 2.0;
            let offset_y = (available.y - image_size.y) / 2.0;
            let image_rect = egui::Rect::from_min_size(
                egui::pos2(offset_x.max(0.0), offset_y.max(0.0)),
                image_size,
            );
            let hover_pos = ctx.input(|i| i.pointer.hover_pos());
            // Handle zoom (mouse wheel) only if hovered
            if let Some(hover_pos) = hover_pos {
                if image_rect.contains(hover_pos) {
                    let zoom_event = ui.input(|i| {
                        i.events.iter().find_map(|e| match e {
                            egui::Event::MouseWheel { delta, .. } => Some(delta.y),
                            _ => None,
                        })
                    });
                    if let Some(scroll) = zoom_event {
                        if scroll.abs() > 0.0 {
                            // Mandelbrot coordinate under mouse before zoom
                            let px = (hover_pos.x - offset_x.max(0.0)).clamp(0.0, side as f32 - 1.0) as usize;
                            let py = (hover_pos.y - offset_y.max(0.0)).clamp(0.0, side as f32 - 1.0) as usize;
                            let (cx, cy) = pixel_to_mandelbrot(px, py, side, side, self.center, self.scale);
                            // Zoom factor
                            let zoom_factor = if scroll > 0.0 { 0.8 } else { 1.25 };
                            let new_scale = self.scale * zoom_factor;
                            // After zoom, what center keeps (cx, cy) under the mouse?
                            let (new_center_x, new_center_y) = {
                                let fx = px as f64 / side as f64;
                                let fy = py as f64 / side as f64;
                                let new_center_x = cx - (fx - 0.5) * new_scale;
                                let new_center_y = cy - (fy - 0.5) * new_scale;
                                (new_center_x, new_center_y)
                            };
                            self.center = (new_center_x, new_center_y);
                            self.scale = new_scale;
                            zoomed = true;
                        }
                    }
                }
            }
            // Re-render if size changed or zoomed
            if size != self.last_size || zoomed {
                let image = render_mandelbrot(side, side, self.center, self.scale);
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
                    let path = mandelbrot_path(px, py, side, side, self.center, self.scale);
                    self.last_click = Some((px, py));
                    self.last_path = path;
                }
                // Draw the path if available
                if !self.last_path.is_empty() {
                    let painter = ui.painter();
                    let to_screen = |zx: f64, zy: f64| -> egui::Pos2 {
                        let (fx, fy) = mandelbrot_to_pixel(zx, zy, side, side, self.center, self.scale);
                        egui::pos2(
                            image_response.rect.left() + fx,
                            image_response.rect.top() + fy,
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

fn render_mandelbrot(width: usize, height: usize, center: (f64, f64), scale: f64) -> egui::ColorImage {
    let mut pixels = Vec::with_capacity(width * height);
    let max_iter = 100;
    for y in 0..height {
        for x in 0..width {
            let (cx, cy) = pixel_to_mandelbrot(x, y, width, height, center, scale);
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
                // Map t to hue (0..360) for a rainbow spectrum
                let t = 1.0 - (iter as f32 / max_iter as f32);
                let hue = t * 360.0;
                let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
                egui::Color32::from_rgb(r, g, b)
            };
            pixels.push(color);
        }
    }
    egui::ColorImage {
        size: [width, height],
        pixels,
    }
}

fn pixel_to_mandelbrot(x: usize, y: usize, width: usize, height: usize, center: (f64, f64), scale: f64) -> (f64, f64) {
    let fx = x as f64 / width as f64;
    let fy = y as f64 / height as f64;
    let cx = center.0 + (fx - 0.5) * scale;
    let cy = center.1 + (fy - 0.5) * scale;
    (cx, cy)
}

fn mandelbrot_to_pixel(zx: f64, zy: f64, width: usize, height: usize, center: (f64, f64), scale: f64) -> (f32, f32) {
    let fx = ((zx - center.0) / scale + 0.5) * width as f64;
    let fy = ((zy - center.1) / scale + 0.5) * height as f64;
    (fx as f32, fy as f32)
}

fn mandelbrot_path(px: usize, py: usize, width: usize, height: usize, center: (f64, f64), scale: f64) -> Vec<(f64, f64)> {
    let mut path = Vec::new();
    let (cx, cy) = pixel_to_mandelbrot(px, py, width, height, center, scale);
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

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        300..=359 => (c, 0.0, x),
        _ => (0.0, 0.0, 0.0),
    };
    let r = ((r1 + m) * 255.0).round() as u8;
    let g = ((g1 + m) * 255.0).round() as u8;
    let b = ((b1 + m) * 255.0).round() as u8;
    (r, g, b)
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
