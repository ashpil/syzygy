use crate::hittable::{intersect_spheres, Sphere};
use crate::camera::Camera;
use xenon::color::Color;
use crate::ray::Ray;
use crate::random::with_rng;
use std::fs::File;
use xenon::write::fn_to_png;
use antsy::LoadingBar;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Renderer<C: Camera + Sync> {
    world: Vec<Sphere>,
    camera: C,
    num_rays: AtomicUsize,
    image_width: u32,
    aspect_ratio: f64,
    max_depth: u16,
    num_samples: u16,
}

impl<C: Camera + Sync> Renderer<C> {
    pub fn new(world: Vec<Sphere>, camera: C) -> Self {
        Renderer {
            world,
            camera,
            num_rays: AtomicUsize::new(0),
            image_width: 800,
            aspect_ratio: 16.0 / 9.0,
            max_depth: 50,
            num_samples: 100,
        }
    }

    pub fn width(self, image_width: u32) -> Self {
        Renderer {image_width, ..self}
    }

    pub fn aspect_ratio(self, aspect_ratio: f64) -> Self {
        Renderer {aspect_ratio, ..self}
    }

    pub fn num_samples(self, num_samples: u16) -> Self {
        Renderer {num_samples, ..self}
    }

    pub fn render_to_file(self, filename: &str) {
        let file = File::create(filename).unwrap();
        let image_height = (self.image_width as f64 / self.aspect_ratio) as u32;

        let mut loadingbar = Mutex::new(LoadingBar::new(image_height, self.image_width).unwrap());

        fn_to_png(self.image_width, image_height, file, |i, j| {
            loadingbar.lock().unwrap().advance().unwrap();
            (0..(self.num_samples + 1)).map(|_| {
                let u = (i as f64 + with_rng(rand::Rng::gen::<f64>)) / (self.image_width - 1) as f64;
                let v = (j as f64 + with_rng(rand::Rng::gen::<f64>)) / (image_height - 1) as f64;
                let r = self.camera.make_ray(u, v);
                self.ray_color(r, self.max_depth)
            }).sum::<Color>() / self.num_samples as f64
        });
        loadingbar.get_mut().unwrap().advance().unwrap();
        let elapsed = loadingbar.into_inner().unwrap().get_elapsed().as_secs_f64();
        let num_rays = self.num_rays.into_inner();
        let time_str = format!("Took {:.4} seconds, shot {} rays, {:.4} mrays/s", elapsed, num_rays, num_rays as f64 / elapsed / 1_000_000.0);
        println!("{}", time_str);

    }

    fn ray_color(&self, r: Ray, depth: u16) -> Color {
        self.num_rays.fetch_add(1, Ordering::Relaxed);
        if depth == 0 {
            Color::new(0.0, 0.0, 0.0)
        } else if let Some(hit) = intersect_spheres(&self.world, &r, 0.00001, f64::INFINITY) {
            if let Some((scattered_ray, atten)) = hit.mat.scatter(hit, r) {
                atten * self.ray_color(scattered_ray, depth - 1)
            } else {
                Color::new(0.0, 0.0, 0.0)
            }
        } else {
            let unit_dir = r.d.unit_vec();
            let t = 0.5 * (unit_dir.y + 1.0);
            (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
        }
    }
}

