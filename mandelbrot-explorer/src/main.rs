use eframe::{egui, App, CreationContext};

struct MandelbrotApp {
    mandelbrot_texture: egui::TextureHandle,
    last_size: [usize; 2],
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
        }
    }
}

impl App for MandelbrotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mandelbrot Explorer");
            let available = ui.available_size();
            let width = available.x.max(100.0) as usize;
            let height = available.y.max(100.0) as usize;
            let size = [width, height];
            if size != self.last_size {
                let image = render_mandelbrot(width, height);
                self.mandelbrot_texture.set(image, egui::TextureOptions::default());
                self.last_size = size;
            }
            ui.image(&self.mandelbrot_texture);
        });
    }
}

fn render_mandelbrot(width: usize, height: usize) -> egui::ColorImage {
    let mut pixels = Vec::with_capacity(width * height);
    let max_iter = 100;
    let scale_x = 3.5 / width as f64;
    let scale_y = 2.0 / height as f64;
    for y in 0..height {
        for x in 0..width {
            let cx = x as f64 * scale_x - 2.5;
            let cy = y as f64 * scale_y - 1.0;
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

fn main() -> eframe::Result<()> {
    let mut options = eframe::NativeOptions::default();
    options.viewport = egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]);
    eframe::run_native(
        "Mandelbrot Explorer",
        options,
        Box::new(|cc| Ok(Box::new(MandelbrotApp::new(cc)))),
    )
}
