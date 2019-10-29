#![allow(unused_variables)]
mod raytracer;

extern crate minifb;
extern crate png;
extern crate rand;
extern crate rayon;

use minifb::{Key, Scale, Window, WindowOptions};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::{Arc, Mutex};
// To use encoder.set()
use rand::Rng;
use raytracer::camera::Camera;
use raytracer::light::Light;
use raytracer::scene::Scene;
use raytracer::sphere::Sphere;
use raytracer::textured_sphere::TexturedSphere;
use raytracer::vec3::Vec3;
use std::time::Instant;

use rayon::prelude::*;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const WIDTH_DIV: f32 = (1.0 / WIDTH as f32);
const HEIGHT_DIV: f32 = (1.0 / HEIGHT as f32);
const BOX_SIDE: usize = 96;
const MAX_ITERATION: u32 = 5;
const RAY_PER_PIXEL: u32 = 100;
const RANDOM_OFFSET_COUNT: usize = RAY_PER_PIXEL as usize * 100;

fn color(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}

fn main() {
    let mut window = Window::new(
        "Raytracer - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let origin = Vec3::new(0.0, 0.5, 0.0);
    let direction = Vec3::new(0.0, 0.0, -1.0);

    let camera = Camera::new(origin, direction, 4.0, 2.0);

    let mut scene = Scene::new();

    scene.add_light(Light::new(origin + Vec3::new(0.0, 1.5, 1.0)));

    // Ground
    scene.add_object(Box::new(TexturedSphere::new(
        Vec3::new(0.0, -10000.0, -1.0),
        10000.0,
        (255u8, 255u8, 255u8),
        0.2,
    )));

    // Left - Black
    scene.add_object(Box::new(Sphere::new(
        Vec3::new(-1.5, 0.5, -1.0),
        0.5,
        (0u8, 0u8, 0u8),
        0.9,
    )));

    // Middle - Yellow
    scene.add_object(Box::new(Sphere::new(
        Vec3::new(0.0, 0.75, -1.5),
        0.75,
        (255u8, 255u8, 0u8),
        0.5,
    )));

    // Right - Red
    scene.add_object(Box::new(Sphere::new(
        Vec3::new(1.5, 0.5, -1.0),
        0.5,
        (255u8, 0u8, 0u8),
        0.2,
    )));

    let mut rng = rand::XorShiftRng::new_unseeded();
    let mut random_offsets: Vec<f32> = vec![0.0; RANDOM_OFFSET_COUNT];
    if RAY_PER_PIXEL > 1 {
        for i in 0..RANDOM_OFFSET_COUNT {
            random_offsets[i] = rng.next_f32() * 2.0 - 1.0;
        }
    }

    let box_count_x: usize = WIDTH / BOX_SIDE + if WIDTH % BOX_SIDE != 0 { 1 } else { 0 };
    let box_count_y: usize = HEIGHT / BOX_SIDE + if HEIGHT % BOX_SIDE != 0 { 1 } else { 0 };

    let mut boxes: Vec<usize> = (0..box_count_x * box_count_y).collect();
    rng.shuffle(&mut boxes);

    let start = Instant::now();

    let result: Vec<_> = boxes.par_iter().map(|i| -> (usize, usize, Vec<u32>) {
        let x = i % box_count_x;
        let y = i / box_count_x;

        let min_x = x * BOX_SIDE;
        let min_y = y * BOX_SIDE;

        let max_x = (min_x + BOX_SIDE).min(WIDTH);
        let max_y = (min_y + BOX_SIDE).min(HEIGHT);

        let mut tmp_buffer: Vec<u32> = vec![0; BOX_SIDE * BOX_SIDE];

        let mut random_offset = 0usize;

        for y in min_y..max_y {
            for x in min_x..max_x {
                let mut color_r = 0u32;
                let mut color_g = 0u32;
                let mut color_b = 0u32;
                

                for i in 0..RAY_PER_PIXEL {
                    let factor_x = (x as f32 + random_offsets[random_offset + 0]) * WIDTH_DIV;
                    let factor_y = (y as f32 + random_offsets[random_offset + 1]) * HEIGHT_DIV;
                    random_offset += 2;
                    random_offset %= RANDOM_OFFSET_COUNT;

                    let ray = camera.get_ray(factor_x, factor_y);
                    let (_, r, g, b) = scene.trace(ray, MAX_ITERATION);

                    color_r += r as u32;
                    color_g += g as u32;
                    color_b += b as u32;
                }

                let y_index = y - min_y;
                let x_index = x - min_x;

                color_r /= RAY_PER_PIXEL;
                color_g /= RAY_PER_PIXEL;
                color_b /= RAY_PER_PIXEL;

                tmp_buffer[(y_index * BOX_SIDE + x_index) as usize] = color(color_r as u8, color_g as u8, color_b as u8);
            }
        }

        (min_x, min_y, tmp_buffer)
    }).collect();

    for (x,y,vec) in result {
        let min_x = x;
        let min_y = y;

        let max_x = (min_x + BOX_SIDE).min(WIDTH);
        let max_y = (min_y + BOX_SIDE).min(HEIGHT);

        for y in min_y..max_y {
            for x in min_x..max_x {
                let y_index = y - min_y;
                let x_index = x - min_x;

                buffer[(y * WIDTH + x) as usize] = vec[(y_index * BOX_SIDE + x_index) as usize];
            }
        }
    }

    let duration = start.elapsed();

    println!("Rendering took {}s", duration.as_secs_f32());

    window.update_with_buffer(&buffer).unwrap();

    // Save as PNG
    let path = Path::new(r"raytracer.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, WIDTH as u32, HEIGHT as u32); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();

    let mut png_data = vec![0u8; 0];
    png_data.reserve_exact(WIDTH * HEIGHT * 3);

    for value in buffer.iter() {
        let r = ((value & 0x00FF0000) >> 16) as u8;
        let g = ((value & 0x0000FF00) >> 8) as u8;
        let b = ((value & 0x000000FF) >> 0) as u8;

        png_data.push(r);
        png_data.push(g);
        png_data.push(b);
    }

    writer.write_image_data(&png_data).unwrap(); // Save

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update();
    }
}
