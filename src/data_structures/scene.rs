use crate::traits::RenderObject;
use crate::data_structures::Color;
use crate::data_structures::Ray;
use crate::traits::Material;
use crate::samplers::RenderObjectSampler;
use crate::samplers::MixtureSampler;
use crate::traits::Sampler;
use crate::samplers::CosineSampler;
use crate::data_structures::IntersectionPayload;

pub struct Scene {
    render_objects: Vec<Box<dyn RenderObject>>,
    materials: Vec<Box<dyn Material>>,
    lights: Vec<Box<dyn RenderObject>>,
    background_color: Color,

    max_depth: usize
}

impl Scene {

    pub fn get_color(&self, ray: Ray) -> Color {
        self.trace(ray, 0)
    }

    fn trace(&self, ray: Ray, depth: usize) -> Color {
        if depth > self.max_depth { return Color (0.0, 0.0, 0.0, 1.0); }

        let payload_option = self.get_intersect(&ray);
        match payload_option {
            None => self.background_color,
            Some(payload) => {
                let material = &self.materials[payload.material_id];
                let light_emmited = material.emmission(&payload);

                let pdf_a = RenderObjectSampler::new(payload.position, &self.lights[0]);
                let pdf_b = CosineSampler::new(payload.normal);
                let mix_pdf = pdf_a; // MixturePDF::new(&pdf_a, &pdf_b);

                let outgoing_direction = mix_pdf.generate();
                let outgoing_ray = Ray { origin: payload.position, direction: outgoing_direction };
                let pdf_value = mix_pdf.value(outgoing_direction);

                let light_sampled = self.trace(outgoing_ray, depth + 1);
                let light_transmitted = material.transmission(&payload, ray.direction, outgoing_direction);

                if pdf_value != 0.0 { 
                    light_emmited + (light_transmitted * light_sampled) / pdf_value
                } else {
                    light_emmited
                }
            }
        }
    }

    fn get_intersect(&self, ray: &Ray) -> Option<IntersectionPayload> {
        let mut record_payload = None;
        
        for object in &self.render_objects {
            let bounds = object.bounds();
            if !bounds.intersect(ray) { continue; }
            match object.intersect(ray) {
                None => (),
                Some(payload) => {
                    record_payload = match record_payload {
                        None => Some(payload),
                        Some(record) => {
                            if payload.distance < record.distance { Some(payload) } else { Some(record) }
                        }
                    }
                }
            }
        }
        record_payload
    }

    pub fn new(render_objects: Vec<Box<dyn RenderObject>>, materials: Vec<Box<dyn Material>>, lights: Vec<Box<dyn RenderObject>>, background_color: Color, max_depth: usize) -> Scene {
        Scene { render_objects, materials, lights, background_color, max_depth }
    }
}
