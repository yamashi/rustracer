use super::ray::Ray;
use super::vec3::Vec3;

pub trait Hittable: Sync + Send {
    fn compute_hit(&self, ray: &Ray, t: &mut f32, hitpoint: &mut Vec3, normal: &mut Vec3) -> bool;

    fn get_color(&self, position: &Vec3) -> (f32, f32, f32);

    fn get_reflection_factor(&self) -> f32;

    fn get_inverse_reflection_factor(&self) -> f32;
}
