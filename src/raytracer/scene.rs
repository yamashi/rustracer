use super::hittable::Hittable;
use super::light::Light;
use super::ray::Ray;
use super::vec3::Vec3;

pub struct Scene {
    lights: Vec<Light>,
    objects: Vec<Box<dyn Hittable>>,
}

fn background_color(ray: &Ray) -> (f32, f32, f32) {
    let unit_x = Vec3::new(1.0, 0.0, 0.0);
    let unit_y = Vec3::new(0.0, 1.0, 0.0);

    const DOT_DIVIDE: f32 = 0.5;

    let dot_x = 1.0 + Vec3::dot_product(&unit_x, ray.direction()) * DOT_DIVIDE;
    let dot_y = 1.0 + Vec3::dot_product(&unit_y, ray.direction()) * DOT_DIVIDE;

    let r = (dot_x * 0.392).min(1.0);
    let g = (dot_x * 0.392).min(1.0);
    let b = (0.392 + (dot_y * 0.392)).min(1.0);

    (r, g, b)
}

fn reflect(r: &Vec3, n: &Vec3) -> Vec3 {
    2.0 * n * Vec3::dot_product(&n, &r) - &r
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            lights: Vec::new(),
            objects: Vec::new(),
        }
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn add_object(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }

    pub fn trace(&self, ray: Ray, max_iter: u32) -> (f32, f32, f32, f32) {
        let mut closest_hit = std::f32::INFINITY;
        let mut closest_position = Vec3::zero();
        let mut closest_normal = Vec3::zero();
        let mut closest_object: Option<usize> = None;

        for (i, object) in self.objects.iter().enumerate() {
            let mut hit_distance = 0.0;
            let mut hit_position = Vec3::zero();
            let mut hit_normal = Vec3::zero();
            if object.compute_hit(&ray, &mut hit_distance, &mut hit_position, &mut hit_normal) {
                if hit_distance < closest_hit {
                    closest_hit = hit_distance;
                    closest_position = hit_position;
                    closest_normal = hit_normal;
                    closest_object = Some(i);
                }
            }
        }

        match closest_object {
            Some(object) => {
                let object = &self.objects[object];

                let (o_r, o_g, o_b) = object.get_color(&closest_position);

                let reflection_factor = object.get_reflection_factor();
                let inverse_reflection_factor = object.get_inverse_reflection_factor();
                closest_normal.normalize();

                let (mut r, mut g, mut b): (f32, f32, f32);
                if reflection_factor > 0.001 {
                    let reflection = reflect(&-ray.direction(), &closest_normal);

                    let mut final_color_r = o_r * inverse_reflection_factor;
                    let mut final_color_g = o_g * inverse_reflection_factor;
                    let mut final_color_b = o_b * inverse_reflection_factor;

                    if max_iter > 0 {
                        let (hit, reflected_r, reflected_g, reflected_b) = self.trace(
                            Ray::new(closest_position + reflection * 0.01, reflection),
                            max_iter - 1,
                        );

                        final_color_r += reflected_r * reflection_factor;
                        final_color_g += reflected_g * reflection_factor;
                        final_color_b += reflected_b * reflection_factor;
                    }

                    r = final_color_r.min(1.0);
                    g = final_color_g.min(1.0);
                    b = final_color_b.min(1.0);
                } else {
                    r = o_r;
                    g = o_g;
                    b = o_b;
                }

                if max_iter > 0 {
                    for light in self.lights.iter() {
                        let mut direction = light.get_position() - closest_position;
                        let mut length = 0.0;

                        direction.normalize_out_length(&mut length);

                        let (dist, _, _, _) = self.trace(
                            Ray::new(closest_position + direction * 0.01, direction),
                            max_iter - 1,
                        );

                        let phong = Vec3::dot_product(&direction, &closest_normal).abs();
                        r *= phong;
                        g *= phong;
                        b *= phong;

                        if dist < length {
                            r *= 0.1;
                            g *= 0.1;
                            b *= 0.1;
                        }
                    }
                }

                (closest_hit, r, g, b)
            }
            None => {
                let (r, g, b) = background_color(&ray);
                (closest_hit, r, g, b)
            }
        }        
    }
}
