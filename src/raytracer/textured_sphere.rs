use super::hittable::Hittable;
use super::ray::Ray;
use super::sphere::Sphere;
use super::vec3::Vec3;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TexturedSphere {
    sphere: Sphere,
}

impl TexturedSphere {
    pub fn new(
        center: Vec3,
        radius: f32,
        color: (u8, u8, u8),
        reflection_factor: f32,
    ) -> TexturedSphere {
        TexturedSphere {
            sphere: Sphere::new(center, radius, color, reflection_factor),
        }
    }
}

impl Hittable for TexturedSphere {
    fn compute_hit(&self, ray: &Ray, t: &mut f32, hitpoint: &mut Vec3, normal: &mut Vec3) -> bool {
        self.sphere.compute_hit(ray, t, hitpoint, normal)
    }

    fn get_color(&self, position: &Vec3) -> (f32, f32, f32) {
        let up = Vec3::new(0.0, 1.0, 0.0);

        let size = 1.0;

        let is_even = (position.z % size).abs() > size / 2.0;

        if (position.x.rem_euclid(size) > size / 2.0) ^ is_even {
            self.sphere.get_color()
        } else {
            (0.0, 0.0, 0.0)
        }
    }

    fn get_reflection_factor(&self) -> f32 {
        self.sphere.get_reflection_factor()
    }

    fn get_inverse_reflection_factor(&self) -> f32 {
        self.sphere.get_inverse_reflection_factor()
    }
}
