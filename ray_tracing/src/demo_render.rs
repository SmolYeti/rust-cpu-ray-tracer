use crate::affine_transforms::{RotateY, Translate};
use crate::bezier_patch::BezierPatch;
use crate::bvh_node::BVHNode;
use crate::camera::Camera;
use crate::checker_texture::CheckerTexture;
use crate::constant_medium::ConstantMedium;
use crate::dielectric::Dielectric;
use crate::diffuse_light::DiffuseLight;
use crate::hittable::Hittable;
use crate::hittable_list::HittableList;
use crate::image_texture::ImageTexture;
use crate::lambertian::Lambertian;
use crate::material::Material;
use crate::metal::Metal;
use crate::noise_texture::NoiseTexture;
use crate::quad::{Quad, quad_box};
use crate::save_show::{render_buffer, save_image};
use crate::sphere::Sphere;
use crate::texture::Texture;
use nurbs::{
    bezier_curve::BezierCurve3D, bezier_surface::BezierSurface, point_types::Point3D,
    utility::random_f64_range, vector_3::Vec3,
};
use rand;
use std::sync::Arc;

pub fn final_render_book1(width: i32, samples: i32, depth: i32, show: bool, save: bool) {
    // World
    let mut build_world = HittableList::new();

    // Ground
    let ground_mat = Arc::new(Lambertian::from_color(Vec3::new(0.5, 0.5, 0.5)));
    build_world.add(Arc::new(Sphere::new(
        Vec3::new(0.0, -1000.0, -0.0),
        1000.0,
        ground_mat,
    )));

    //Create random spheres
    let compare_vec = Vec3::new(4.0, 0.2, 0.0);
    for a in -11..11 {
        for b in -11..11 {
            let rand_mat = rand::random::<f64>();
            let center = Vec3::new(
                a as f64 + (0.9 * rand::random::<f64>()),
                0.2,
                b as f64 + (0.9 * rand::random::<f64>()),
            );

            if (center - compare_vec).length() > 0.9 {
                let mat: Arc<dyn Material + Sync + Send>;
                if rand_mat < 0.8 {
                    let albedo = Vec3::random() * Vec3::random();
                    mat = Arc::new(Lambertian::from_color(albedo));
                    let center_end = center + Vec3::new(0.0, rand::random::<f64>() * 0.25, 0.0);
                    build_world.add(Arc::new(Sphere::new_moving(center, 0.2, mat, center_end)));
                } else if rand_mat < 0.95 {
                    let albedo = Vec3::random_range(0.5, 1.0);
                    let fuzz = random_f64_range(0.0, 0.5);
                    mat = Arc::new(Metal::new(albedo, fuzz));
                    build_world.add(Arc::new(Sphere::new(center, 0.2, mat)));
                } else {
                    mat = Arc::new(Dielectric::new(1.5));
                    build_world.add(Arc::new(Sphere::new(center, 0.2, mat)));
                }
            }
        }
    }

    // Center spheres
    let mat = Arc::new(Dielectric::new(1.5));
    build_world.add(Arc::new(Sphere::new(Vec3::new(0.0, 1.0, 0.0), 1.0, mat)));

    let mat = Arc::new(Lambertian::from_color(Vec3::new(0.4, 0.2, 0.1)));
    build_world.add(Arc::new(Sphere::new(Vec3::new(-4.0, 1.0, 0.0), 1.0, mat)));

    let mat = Arc::new(Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.0));
    build_world.add(Arc::new(Sphere::new(Vec3::new(4.0, 1.0, 0.0), 1.0, mat)));

    // Make the world into a bvh
    let mut world = HittableList::new();
    world.add(Arc::new(BVHNode::from_list(&build_world)));

    // Camera
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = width;
    cam.samples_per_pixel = samples;
    cam.max_depth = depth;

    cam.vfov = 20.0;
    cam.look_from = Vec3::new(13.0, 2.0, 3.0);
    cam.look_at = Vec3::new(0.0, 0.0, 0.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.6;
    cam.focus_dist = 10.0;

    let world = Arc::new(world);
    let buffer = cam.render(world, None, true, 6);
    if save {
        save_image(
            "Book1_Final",
            &buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
            samples,
            depth,
        );
    }
    if show {
        render_buffer(
            buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
        );
    }
}

pub fn checker_spheres(width: i32, samples: i32, depth: i32, show: bool, save: bool) {
    // World
    let mut world = HittableList::new();

    let checker: Arc<dyn Texture + Sync + Send> = Arc::new(CheckerTexture::new(
        0.32,
        Vec3::new(0.2, 0.3, 0.1),
        Vec3::new(0.9, 0.9, 0.9),
    ));
    let ground_mat = Arc::new(Lambertian::new(Arc::clone(&checker)));
    world.add(Arc::new(Sphere::new(
        Vec3::new(0.0, -10.0, -0.0),
        10.0,
        ground_mat,
    )));
    let ground_mat = Arc::new(Lambertian::new(checker));
    world.add(Arc::new(Sphere::new(
        Vec3::new(0.0, 10.0, -0.0),
        10.0,
        ground_mat,
    )));

    // Camera
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = width;
    cam.samples_per_pixel = samples;
    cam.max_depth = depth;

    cam.vfov = 20.0;
    cam.look_from = Vec3::new(13.0, 2.0, 3.0);
    cam.look_at = Vec3::new(0.0, 0.0, 0.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;

    let world = Arc::new(world);
    let buffer = cam.render(world, None, true, 6);
    if save {
        save_image(
            "Checker_Spheres",
            &buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
            samples,
            depth,
        );
    }
    if show {
        render_buffer(
            buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
        );
    }
}

pub fn earth(width: i32, samples: i32, depth: i32, show: bool, save: bool) {
    let mut world = HittableList::new();

    let earth_texture = Arc::new(ImageTexture::new("ray_tracing/src/earthmap.jpg"));
    let ground_mat = Arc::new(Lambertian::new(earth_texture));
    world.add(Arc::new(Sphere::new(
        Vec3::new(0.0, 0.0, 0.0),
        2.0,
        ground_mat,
    )));

    // Camera
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = width;
    cam.samples_per_pixel = samples;
    cam.max_depth = depth;

    cam.vfov = 20.0;
    cam.look_from = Vec3::new(0.0, 0.0, 12.0);
    cam.look_at = Vec3::new(0.0, 0.0, 0.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;

    let world = Arc::new(world);
    let buffer = cam.render(world, None, false, 6);
    if save {
        save_image(
            "Earth",
            &buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
            samples,
            depth,
        );
    }
    if show {
        render_buffer(
            buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
        );
    }
}

pub fn two_perlin_sphere(width: i32, samples: i32, depth: i32, show: bool, save: bool) {
    let mut world = HittableList::new();

    let pertext: Arc<dyn Texture + Sync + Send> = Arc::new(NoiseTexture::new(4.0));
    let mat_0 = Arc::new(Lambertian::new(Arc::clone(&pertext)));
    world.add(Arc::new(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        mat_0,
    )));
    let mat_1 = Arc::new(Lambertian::new(Arc::clone(&pertext)));
    world.add(Arc::new(Sphere::new(Vec3::new(0.0, 2.0, 0.0), 2.0, mat_1)));

    // Camera
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = width;
    cam.samples_per_pixel = samples;
    cam.max_depth = depth;

    cam.vfov = 20.0;
    cam.look_from = Vec3::new(13.0, 2.0, 3.0);
    cam.look_at = Vec3::new(0.0, 0.0, 0.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;

    let world = Arc::new(world);
    let buffer = cam.render(world, None, true, 6);
    if save {
        save_image(
            "Perlin_Sphere",
            &buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
            samples,
            depth,
        );
    }
    if show {
        render_buffer(
            buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
        );
    }
}

pub fn quads(width: i32, samples: i32, depth: i32, show: bool, save: bool) {
    let mut world = HittableList::new();

    let red = Arc::new(Lambertian::from_color(Vec3::new(1.0, 0.2, 0.2)));
    let green = Arc::new(Lambertian::from_color(Vec3::new(0.2, 1.0, 0.2)));
    let blue = Arc::new(Lambertian::from_color(Vec3::new(0.2, 0.2, 1.0)));
    let orange = Arc::new(Lambertian::from_color(Vec3::new(1.0, 0.5, 0.2)));
    let teal = Arc::new(Lambertian::from_color(Vec3::new(0.2, 0.8, 0.8)));

    world.add(Arc::new(Quad::new(
        Vec3::new(-3.0, -2.0, 5.0),
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 4.0, 0.0),
        red,
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(-2.0, -2.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
        green,
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(3.0, -2.0, 1.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 4.0, 0.0),
        blue,
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(-2.0, 3.0, 1.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        orange,
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(-2.0, -3.0, 5.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -4.0),
        teal,
    )));

    // Camera
    let mut cam = Camera::new();
    cam.aspect_ratio = 1.0;
    cam.image_width = width;
    cam.samples_per_pixel = samples;
    cam.max_depth = depth;

    cam.vfov = 80.0;
    cam.look_from = Vec3::new(0.0, 0.0, 9.0);
    cam.look_at = Vec3::new(0.0, 0.0, 0.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;

    let world = Arc::new(world);
    let buffer = cam.render(world, None, true, 6);
    if save {
        save_image(
            "Quads",
            &buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
            samples,
            depth,
        );
    }
    if show {
        render_buffer(
            buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
        );
    }
}

pub fn simple_light(width: i32, samples: i32, depth: i32, show: bool, save: bool) {
    let mut world = HittableList::new();

    let pertext: Arc<dyn Material + Sync + Send> =
        Arc::new(Lambertian::new(Arc::new(NoiseTexture::new(4.0))));
    world.add(Arc::new(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        -1000.0,
        Arc::clone(&pertext),
    )));
    world.add(Arc::new(Sphere::new(
        Vec3::new(0.0, 2.0, 0.0),
        2.0,
        Arc::clone(&pertext),
    )));

    let mut lights = HittableList::new();
    let diff_light: Arc<dyn Material + Sync + Send> =
        Arc::new(DiffuseLight::color(Vec3::new(4.0, 4.0, 4.0)));
    let light: Arc<dyn Hittable + Sync + Send> = Arc::new(Sphere::new(
        Vec3::new(0.0, 7.0, 0.0),
        2.0,
        Arc::clone(&diff_light),
    ));
    world.add(Arc::clone(&light));
    lights.add(light);
    let light: Arc<dyn Hittable + Sync + Send> = Arc::new(Quad::new(
        Vec3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        diff_light,
    ));
    world.add(Arc::clone(&light));
    lights.add(light);

    // Camera
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = width;
    cam.samples_per_pixel = samples;
    cam.max_depth = depth;
    cam.background = Vec3::new(0.0, 0.0, 0.0);

    cam.vfov = 20.0;
    cam.look_from = Vec3::new(26.0, 3.0, 6.0);
    cam.look_at = Vec3::new(0.0, 2.0, 0.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;

    let world = Arc::new(world);
    let lights = Arc::new(lights);
    let buffer = cam.render(world, Some(lights), true, 6);
    if save {
        save_image(
            "Simple_Light",
            &buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
            samples,
            depth,
        );
    }
    if show {
        render_buffer(
            buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
        );
    }
}

pub fn cornell_box(width: i32, samples: i32, depth: i32, show: bool, save: bool) {
    let mut world = HittableList::new();

    let red = Arc::new(Lambertian::from_color(Vec3::new(0.65, 0.05, 0.05)));
    let white: Arc<dyn Material + Sync + Send> =
        Arc::new(Lambertian::from_color(Vec3::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::from_color(Vec3::new(0.12, 0.45, 0.15)));
    let light_mat = Arc::new(DiffuseLight::color(Vec3::new(15.0, 15.0, 15.0)));

    // Scene
    let light: Arc<dyn Hittable + Sync + Send> = Arc::new(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light_mat,
    ));
    world.add(Arc::clone(&light));
    world.add(Arc::new(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&white),
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        Arc::clone(&white),
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Arc::clone(&white),
    )));

    // Boxes
    let box_1 = quad_box(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(165.0, 330.0, 165.0),
        Arc::clone(&white),
    );
    let box_1 = Arc::new(RotateY::new(box_1, 15.0));
    let box_1 = Arc::new(Translate::new(box_1, Vec3::new(265.0, 0.0, 295.0)));
    world.add(box_1);

    let box_2 = quad_box(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(165.0, 165.0, 165.0),
        Arc::clone(&white),
    );
    let box_2 = Arc::new(RotateY::new(box_2, -18.0));
    let box_2 = Arc::new(Translate::new(box_2, Vec3::new(130.0, 0.0, 65.0)));
    world.add(box_2);

    // Camera
    let mut cam = Camera::new();
    cam.aspect_ratio = 1.0;
    cam.image_width = width;
    cam.samples_per_pixel = samples;
    cam.max_depth = depth;
    cam.background = Vec3::new(0.0, 0.0, 0.0);

    cam.vfov = 40.0;
    cam.look_from = Vec3::new(278.0, 278.0, -800.0);
    cam.look_at = Vec3::new(278.0, 278.0, 0.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;

    let world = Arc::new(world);
    let buffer = cam.render(world, Some(light), true, 6);

    if save {
        save_image(
            "Cornell_Box",
            &buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
            samples,
            depth,
        );
    }

    if show {
        render_buffer(
            buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
        );
    }
}

pub fn cornell_smoke(width: i32, samples: i32, depth: i32, show: bool, save: bool) {
    let mut world = HittableList::new();
    let mut lights = HittableList::new();

    let red = Arc::new(Lambertian::from_color(Vec3::new(0.65, 0.05, 0.05)));
    let white: Arc<dyn Material + Sync + Send> =
        Arc::new(Lambertian::from_color(Vec3::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::from_color(Vec3::new(0.12, 0.45, 0.15)));
    let light_mat = Arc::new(DiffuseLight::color(Vec3::new(7.0, 7.0, 7.0)));

    // Scene
    let light: Arc<dyn Hittable + Sync + Send> = Arc::new(Quad::new(
        Vec3::new(113.0, 554.0, 127.0),
        Vec3::new(330.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 305.0),
        light_mat,
    ));
    world.add(Arc::clone(&light));
    lights.add(light);
    world.add(Arc::new(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&white),
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        Arc::clone(&white),
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Arc::clone(&white),
    )));

    // Boxes
    let box_1 = quad_box(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(165.0, 330.0, 165.0),
        Arc::clone(&white),
    );
    let box_1 = Arc::new(RotateY::new(box_1, 15.0));
    let box_1 = Arc::new(Translate::new(box_1, Vec3::new(265.0, 0.0, 295.0)));
    world.add(Arc::new(ConstantMedium::color(
        box_1,
        0.01,
        Vec3::new(0.0, 0.0, 0.0),
    )));

    let box_2 = quad_box(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(165.0, 165.0, 165.0),
        Arc::clone(&white),
    );
    let box_2 = Arc::new(RotateY::new(box_2, -18.0));
    let box_2 = Arc::new(Translate::new(box_2, Vec3::new(130.0, 0.0, 65.0)));
    world.add(Arc::new(ConstantMedium::color(
        box_2,
        0.01,
        Vec3::new(1.0, 1.0, 1.0),
    )));

    // Camera
    let mut cam = Camera::new();
    cam.aspect_ratio = 1.0;
    cam.image_width = width;
    cam.samples_per_pixel = samples;
    cam.max_depth = depth;
    cam.background = Vec3::new(0.0, 0.0, 0.0);

    cam.vfov = 40.0;
    cam.look_from = Vec3::new(278.0, 278.0, -800.0);
    cam.look_at = Vec3::new(278.0, 278.0, 0.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;

    let world = Arc::new(world);
    let lights = Arc::new(lights);
    let buffer = cam.render(world, Some(lights), true, 6);

    if save {
        save_image(
            "Cornell_Smoke",
            &buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
            samples,
            depth,
        );
    }

    if show {
        render_buffer(
            buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
        );
    }
}

pub fn final_scene_book2(width: i32, samples: i32, depth: i32, show: bool, save: bool, threads: u32) {
    // Create the ground
    let mut boxes_1 = HittableList::new();
    let ground_mat: Arc<dyn Material + Sync + Send> =
        Arc::new(Lambertian::from_color(Vec3::new(0.48, 0.83, 0.53)));

    let boxes_per_side = 20;
    for i in 0..boxes_per_side {
        for j in 0..boxes_per_side {
            let w = 100.0;
            let x0 = -1000.0 + i as f64 * w;
            let z0 = -1000.0 + j as f64 * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = random_f64_range(1.0, 101.0);
            let z1 = z0 + w;

            boxes_1.add(quad_box(
                Vec3::new(x0, y0, z0),
                Vec3::new(x1, y1, z1),
                Arc::clone(&ground_mat),
            ));
        }
    }

    // Create the scene
    let mut scene = HittableList::new();
    scene.add(Arc::new(BVHNode::from_list(&boxes_1)));

    // Light
    let diff_light = Arc::new(DiffuseLight::color(Vec3::new(7.0, 7.0, 7.0)));
    let light: Arc<dyn Hittable + Sync + Send> = Arc::new(Quad::new(
        Vec3::new(123.0, 554.0, 147.0),
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 265.0),
        diff_light,
    ));
    scene.add(Arc::clone(&light));

    // Spheres
    let center_1 = Vec3::new(400.0, 400.0, 200.0);
    let center_2 = center_1 + Vec3::new(30.0, 0.0, 0.0);
    let sphere_mat: Arc<dyn Material + Sync + Send> =
        Arc::new(Lambertian::from_color(Vec3::new(0.7, 0.3, 0.1)));
    scene.add(Arc::new(Sphere::new_moving(
        center_1, 50.0, sphere_mat, center_2,
    )));

    scene.add(Arc::new(Sphere::new(
        Vec3::new(260.0, 150.0, 45.0),
        50.0,
        Arc::new(Dielectric::new(1.5)),
    )));
    scene.add(Arc::new(Sphere::new(
        Vec3::new(0.0, 150.0, 145.0),
        50.0,
        Arc::new(Metal::new(Vec3::new(0.8, 0.8, 0.9), 1.0)),
    )));

    // Fog
    let boundary: Arc<dyn Hittable + Sync + Send> = Arc::new(Sphere::new(
        Vec3::new(360.0, 150.0, 145.0),
        70.0,
        Arc::new(Dielectric::new(1.5)),
    ));
    scene.add(Arc::clone(&boundary));
    scene.add(Arc::new(ConstantMedium::color(
        boundary,
        0.2,
        Vec3::new(0.2, 0.4, 0.9),
    )));
    let fog = Arc::new(Sphere::new(
        Vec3::new(0.0, 0.0, 0.0),
        5000.0,
        Arc::new(Dielectric::new(1.5)),
    ));
    scene.add(Arc::new(ConstantMedium::color(
        fog,
        0.0001,
        Vec3::new(1.0, 1.0, 1.0),
    )));

    // Earth
    let earth_texture = Arc::new(ImageTexture::new("ray_tracing/src/earthmap.jpg"));
    let emat = Arc::new(Lambertian::new(earth_texture));
    scene.add(Arc::new(Sphere::new(
        Vec3::new(400.0, 200.0, 400.0),
        100.0,
        emat,
    )));

    // Perlin
    let pertext = Arc::new(NoiseTexture::new(0.1));
    let pmat = Arc::new(Lambertian::new(pertext));
    scene.add(Arc::new(Sphere::new(
        Vec3::new(220.0, 280.0, 300.0),
        80.0,
        pmat,
    )));

    // Sphere box
    let mut spheres = HittableList::new();
    let white: Arc<dyn Material + Sync + Send> =
        Arc::new(Lambertian::from_color(Vec3::new(0.73, 0.73, 0.73)));
    let ns = 1000;
    for _ in 0..ns {
        spheres.add(Arc::new(Sphere::new(
            Vec3::random_range(0.0, 165.0),
            10.0,
            Arc::clone(&white),
        )));
    }
    scene.add(Arc::new(Translate::new(
        Arc::new(RotateY::new(Arc::new(BVHNode::from_list(&spheres)), 15.0)),
        Vec3::new(-100.0, 270.0, 395.0),
    )));

    // Render
    let mut cam = Camera::new();
    cam.aspect_ratio = 1.0;
    cam.image_width = width;
    cam.samples_per_pixel = samples;
    cam.max_depth = depth;
    cam.background = Vec3::new(0.0, 0.0, 0.0);

    cam.vfov = 40.0;
    cam.look_from = Vec3::new(478.0, 278.0, -600.0);
    cam.look_at = Vec3::new(278.0, 278.0, 0.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;

    let world = Arc::new(scene);
    let buffer = cam.render(world, Some(light), true, threads);

    if save {
        save_image(
            "Book2_Final",
            &buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
            samples,
            depth,
        );
    }

    // Render to a window
    if show {
        render_buffer(
            buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
        );
    }
}

pub fn cornell_box_metal(width: i32, samples: i32, depth: i32, show: bool, save: bool) {
    let mut world = HittableList::new();

    let red = Arc::new(Lambertian::from_color(Vec3::new(0.65, 0.05, 0.05)));
    let white: Arc<dyn Material + Sync + Send> =
        Arc::new(Lambertian::from_color(Vec3::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::from_color(Vec3::new(0.12, 0.45, 0.15)));
    let light_mat = Arc::new(DiffuseLight::color(Vec3::new(15.0, 15.0, 15.0)));
    let metal_mat: Arc<dyn Material + Sync + Send> =
        Arc::new(Metal::new(Vec3::new(0.8, 0.85, 0.88), 0.0));

    // Scene
    let light: Arc<dyn Hittable + Sync + Send> = Arc::new(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light_mat,
    ));
    world.add(Arc::clone(&light));
    world.add(Arc::new(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&white),
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        Arc::clone(&white),
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Arc::clone(&white),
    )));

    // Boxes
    let box_1 = quad_box(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(165.0, 330.0, 165.0),
        metal_mat,
    );
    let box_1 = Arc::new(RotateY::new(box_1, 15.0));
    let box_1 = Arc::new(Translate::new(box_1, Vec3::new(265.0, 0.0, 295.0)));
    world.add(box_1);

    let box_2 = quad_box(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(165.0, 165.0, 165.0),
        Arc::clone(&white),
    );
    let box_2 = Arc::new(RotateY::new(box_2, -18.0));
    let box_2 = Arc::new(Translate::new(box_2, Vec3::new(130.0, 0.0, 65.0)));
    world.add(box_2);

    // Camera
    let mut cam = Camera::new();
    cam.aspect_ratio = 1.0;
    cam.image_width = width;
    cam.samples_per_pixel = samples;
    cam.max_depth = depth;
    cam.background = Vec3::new(0.0, 0.0, 0.0);

    cam.vfov = 40.0;
    cam.look_from = Vec3::new(278.0, 278.0, -800.0);
    cam.look_at = Vec3::new(278.0, 278.0, 0.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;

    let world = Arc::new(world);
    let buffer = cam.render(world, Some(light), true, 7);

    if save {
        save_image(
            "Cornell_Box_Metal",
            &buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
            samples,
            depth,
        );
    }

    if show {
        render_buffer(
            buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
        );
    }
}

pub fn cornell_box_glass_sphere(width: i32, samples: i32, depth: i32, show: bool, save: bool) {
    let mut world = HittableList::new();

    let red = Arc::new(Lambertian::from_color(Vec3::new(0.65, 0.05, 0.05)));
    let white: Arc<dyn Material + Sync + Send> =
        Arc::new(Lambertian::from_color(Vec3::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::from_color(Vec3::new(0.12, 0.45, 0.15)));
    let light_mat = Arc::new(DiffuseLight::color(Vec3::new(15.0, 15.0, 15.0)));
    let glass = Arc::new(Dielectric::new(1.5));

    // Scene
    let light: Arc<dyn Hittable + Sync + Send> = Arc::new(Quad::new(
        Vec3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light_mat,
    ));
    world.add(Arc::clone(&light));
    world.add(Arc::new(Quad::new(
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Arc::clone(&white),
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        Arc::clone(&white),
    )));
    world.add(Arc::new(Quad::new(
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Arc::clone(&white),
    )));

    // Boxes
    let box_1 = quad_box(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(165.0, 330.0, 165.0),
        Arc::clone(&white),
    );
    let box_1 = Arc::new(RotateY::new(box_1, 15.0));
    let box_1 = Arc::new(Translate::new(box_1, Vec3::new(265.0, 0.0, 295.0)));
    world.add(box_1);

    let sphere = Arc::new(Sphere::new(Vec3::new(190.0, 90.0, 190.0), 90.0, glass));
    world.add(sphere);

    // Camera
    let mut cam = Camera::new();
    cam.aspect_ratio = 1.0;
    cam.image_width = width;
    cam.samples_per_pixel = samples;
    cam.max_depth = depth;
    cam.background = Vec3::new(0.0, 0.0, 0.0);

    cam.vfov = 40.0;
    cam.look_from = Vec3::new(278.0, 278.0, -800.0);
    cam.look_at = Vec3::new(278.0, 278.0, 0.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;

    let world = Arc::new(world);
    let buffer = cam.render(world, Some(light), true, 7);

    if save {
        save_image(
            "Cornell_Box_Glass_Sphere",
            &buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
            samples,
            depth,
        );
    }

    if show {
        render_buffer(
            buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
        );
    }
}

pub fn utah_teapot(width: i32, samples: i32, depth: i32, show: bool, save: bool, threads: u32) {
    // Model from https://users.cs.utah.edu/~dejohnso/models/teapot.html

    // World
    let mut build_world = HittableList::new();

    let mat = Arc::new(Lambertian::from_color(Vec3::new(0.5, 0.5, 0.5)));
    // Teapot
    /*
    #
    # teapot: this is the original non-rational bezier teapot data. the data
    #         comes from Martin Newel and Jim Blinn.
    #
    # Thomas V Thompson II
    # January 28 2000
    #
    */
    {
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.4000, 2.25000, 0.0]),
                    Point3D::new([1.3375, 2.38125, 0.0]),
                    Point3D::new([1.4375, 2.38125, 0.0]),
                    Point3D::new([1.5000, 2.25000, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.4000, 2.25000, 0.784]),
                    Point3D::new([1.3375, 2.38125, 0.789]),
                    Point3D::new([1.4375, 2.38125, 0.805]),
                    Point3D::new([1.5000, 2.25000, 0.840]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.784, 2.25000, 1.4000]),
                    Point3D::new([0.749, 2.38125, 1.3375]),
                    Point3D::new([0.805, 2.38125, 1.4375]),
                    Point3D::new([0.840, 2.25000, 1.5000]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 2.25000, 1.4000]),
                    Point3D::new([0.0, 2.38125, 1.3375]),
                    Point3D::new([0.0, 2.38125, 1.4375]),
                    Point3D::new([0.0, 2.25000, 1.5000]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 2.25000, 1.4000]),
                    Point3D::new([0.0, 2.38125, 1.3375]),
                    Point3D::new([0.0, 2.38125, 1.4375]),
                    Point3D::new([0.0, 2.25000, 1.5000]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.784, 2.25000, 1.4000]),
                    Point3D::new([-0.749, 2.38125, 1.3375]),
                    Point3D::new([-0.805, 2.38125, 1.4375]),
                    Point3D::new([-0.840, 2.25000, 1.5000]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.4000, 2.25000, 0.784]),
                    Point3D::new([-1.3375, 2.38125, 0.749]),
                    Point3D::new([-1.4375, 2.38125, 0.805]),
                    Point3D::new([-1.5000, 2.25000, 0.840]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.4000, 2.25000, 0.0]),
                    Point3D::new([-1.3375, 2.38125, 0.0]),
                    Point3D::new([-1.4375, 2.38125, 0.0]),
                    Point3D::new([-1.5000, 2.25000, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.4000, 2.25000, 0.0]),
                    Point3D::new([-1.3375, 2.38125, 0.0]),
                    Point3D::new([-1.4375, 2.38125, 0.0]),
                    Point3D::new([-1.5000, 2.25000, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.4000, 2.25000, -0.784]),
                    Point3D::new([-1.3375, 2.38125, -0.749]),
                    Point3D::new([-1.4375, 2.38125, -0.805]),
                    Point3D::new([-1.5000, 2.25000, -0.840]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.784, 2.25000, -1.4000]),
                    Point3D::new([-0.749, 2.38125, -1.3375]),
                    Point3D::new([-0.805, 2.38125, -1.4375]),
                    Point3D::new([-0.840, 2.25000, -1.5000]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 2.25000, -1.4000]),
                    Point3D::new([0.0, 2.38125, -1.3375]),
                    Point3D::new([0.0, 2.38125, -1.4375]),
                    Point3D::new([0.0, 2.25000, -1.5000]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 2.25000, -1.4000]),
                    Point3D::new([0.0, 2.38125, -1.3375]),
                    Point3D::new([0.0, 2.38125, -1.4375]),
                    Point3D::new([0.0, 2.25000, -1.5000]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.784, 2.25000, -1.4000]),
                    Point3D::new([0.749, 2.38125, -1.3375]),
                    Point3D::new([0.805, 2.38125, -1.4375]),
                    Point3D::new([0.840, 2.25000, -1.5000]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.4000, 2.25000, -0.784]),
                    Point3D::new([1.3375, 2.38125, -0.749]),
                    Point3D::new([1.4375, 2.38125, -0.805]),
                    Point3D::new([1.5000, 2.25000, -0.840]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.4000, 2.25000, 0.0]),
                    Point3D::new([1.3375, 2.38125, 0.0]),
                    Point3D::new([1.4375, 2.38125, 0.0]),
                    Point3D::new([1.5000, 2.25000, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.50, 2.250, 0.0]),
                    Point3D::new([1.75, 1.725, 0.0]),
                    Point3D::new([2.00, 1.200, 0.0]),
                    Point3D::new([2.00, 0.750, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.50, 2.250, 0.84]),
                    Point3D::new([1.75, 1.725, 0.98]),
                    Point3D::new([2.00, 1.200, 1.12]),
                    Point3D::new([2.00, 0.750, 1.12]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.84, 2.250, 1.50]),
                    Point3D::new([0.98, 1.725, 1.75]),
                    Point3D::new([1.12, 1.200, 2.00]),
                    Point3D::new([1.12, 0.750, 2.00]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 2.250, 1.50]),
                    Point3D::new([0.0, 1.725, 1.75]),
                    Point3D::new([0.0, 1.200, 2.00]),
                    Point3D::new([0.0, 0.750, 2.00]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 2.250, 1.50]),
                    Point3D::new([0.0, 1.725, 1.75]),
                    Point3D::new([0.0, 1.200, 2.00]),
                    Point3D::new([0.0, 0.750, 2.00]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.84, 2.250, 1.50]),
                    Point3D::new([-0.98, 1.725, 1.75]),
                    Point3D::new([-1.12, 1.200, 2.00]),
                    Point3D::new([-1.12, 0.750, 2.00]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.50, 2.250, 0.84]),
                    Point3D::new([-1.75, 1.725, 0.98]),
                    Point3D::new([-2.00, 1.200, 1.12]),
                    Point3D::new([-2.00, 0.750, 1.12]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.50, 2.250, 0.0]),
                    Point3D::new([-1.75, 1.725, 0.0]),
                    Point3D::new([-2.00, 1.200, 0.0]),
                    Point3D::new([-2.00, 0.750, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.50, 2.250, 0.0]),
                    Point3D::new([-1.75, 1.725, 0.0]),
                    Point3D::new([-2.00, 1.200, 0.0]),
                    Point3D::new([-2.00, 0.750, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.50, 2.250, -0.84]),
                    Point3D::new([-1.75, 1.725, -0.98]),
                    Point3D::new([-2.00, 1.200, -1.12]),
                    Point3D::new([-2.00, 0.750, -1.12]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.84, 2.250, -1.50]),
                    Point3D::new([-0.98, 1.725, -1.75]),
                    Point3D::new([-1.12, 1.200, -2.00]),
                    Point3D::new([-1.12, 0.750, -2.00]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 2.250, -1.50]),
                    Point3D::new([0.0, 1.725, -1.75]),
                    Point3D::new([0.0, 1.200, -2.00]),
                    Point3D::new([0.0, 0.750, -2.00]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 2.250, -1.50]),
                    Point3D::new([0.0, 1.725, -1.75]),
                    Point3D::new([0.0, 1.200, -2.00]),
                    Point3D::new([0.0, 0.750, -2.00]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.84, 2.250, -1.50]),
                    Point3D::new([0.98, 1.725, -1.75]),
                    Point3D::new([1.12, 1.200, -2.00]),
                    Point3D::new([1.12, 0.750, -2.00]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.50, 2.250, -0.84]),
                    Point3D::new([1.75, 1.725, -0.98]),
                    Point3D::new([2.00, 1.200, -1.12]),
                    Point3D::new([2.00, 0.750, -1.12]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.50, 2.250, 0.0]),
                    Point3D::new([1.75, 1.725, 0.0]),
                    Point3D::new([2.00, 1.200, 0.0]),
                    Point3D::new([2.00, 0.750, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([2.0, 0.750, 0.0]),
                    Point3D::new([2.0, 0.300, 0.0]),
                    Point3D::new([1.5, 0.075, 0.0]),
                    Point3D::new([1.5, 0.000, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([2.0, 0.750, 1.12]),
                    Point3D::new([2.0, 0.300, 1.12]),
                    Point3D::new([1.5, 0.075, 0.84]),
                    Point3D::new([1.5, 0.000, 0.84]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.12, 0.750, 2.0]),
                    Point3D::new([1.12, 0.300, 2.0]),
                    Point3D::new([0.84, 0.075, 1.5]),
                    Point3D::new([0.84, 0.000, 1.5]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 0.750, 2.0]),
                    Point3D::new([0.0, 0.300, 2.0]),
                    Point3D::new([0.0, 0.075, 1.5]),
                    Point3D::new([0.0, 0.000, 1.5]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 0.750, 2.0]),
                    Point3D::new([0.0, 0.300, 2.0]),
                    Point3D::new([0.0, 0.075, 1.5]),
                    Point3D::new([0.0, 0.000, 1.5]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.12, 0.750, 2.0]),
                    Point3D::new([-1.12, 0.300, 2.0]),
                    Point3D::new([-0.84, 0.075, 1.5]),
                    Point3D::new([-0.84, 0.000, 1.5]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-2.0, 0.750, 1.12]),
                    Point3D::new([-2.0, 0.300, 1.12]),
                    Point3D::new([-1.5, 0.075, 0.84]),
                    Point3D::new([-1.5, 0.000, 0.84]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-2.0, 0.750, 0.0]),
                    Point3D::new([-2.0, 0.300, 0.0]),
                    Point3D::new([-1.5, 0.075, 0.0]),
                    Point3D::new([-1.5, 0.000, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([-2.0, 0.750, 0.0]),
                    Point3D::new([-2.0, 0.300, 0.0]),
                    Point3D::new([-1.5, 0.075, 0.0]),
                    Point3D::new([-1.5, 0.000, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-2.0, 0.750, -1.12]),
                    Point3D::new([-2.0, 0.300, -1.12]),
                    Point3D::new([-1.5, 0.075, -0.84]),
                    Point3D::new([-1.5, 0.000, -0.84]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.12, 0.750, -2.0]),
                    Point3D::new([-1.12, 0.300, -2.0]),
                    Point3D::new([-0.84, 0.075, -1.5]),
                    Point3D::new([-0.84, 0.000, -1.5]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 0.750, -2.0]),
                    Point3D::new([0.0, 0.300, -2.0]),
                    Point3D::new([0.0, 0.075, -1.5]),
                    Point3D::new([0.0, 0.000, -1.5]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 0.750, -2.0]),
                    Point3D::new([0.0, 0.300, -2.0]),
                    Point3D::new([0.0, 0.075, -1.5]),
                    Point3D::new([0.0, 0.000, -1.5]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.12, 0.750, -2.0]),
                    Point3D::new([1.12, 0.300, -2.0]),
                    Point3D::new([0.84, 0.075, -1.5]),
                    Point3D::new([0.84, 0.000, -1.5]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([2.0, 0.750, -1.12]),
                    Point3D::new([2.0, 0.300, -1.12]),
                    Point3D::new([1.5, 0.075, -0.84]),
                    Point3D::new([1.5, 0.000, -0.84]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([2.0, 0.750, 0.0]),
                    Point3D::new([2.0, 0.300, 0.0]),
                    Point3D::new([1.5, 0.075, 0.0]),
                    Point3D::new([1.5, 0.000, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.6, 1.875, 0.0]),
                    Point3D::new([-2.3, 1.875, 0.0]),
                    Point3D::new([-2.7, 1.875, 0.0]),
                    Point3D::new([-2.7, 1.650, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.6, 1.875, 0.3]),
                    Point3D::new([-2.3, 1.875, 0.3]),
                    Point3D::new([-2.7, 1.875, 0.3]),
                    Point3D::new([-2.7, 1.650, 0.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.5, 2.10, 0.3]),
                    Point3D::new([-2.5, 2.10, 0.3]),
                    Point3D::new([-3.0, 2.10, 0.3]),
                    Point3D::new([-3.0, 1.65, 0.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.5, 2.10, 0.0]),
                    Point3D::new([-2.5, 2.10, 0.0]),
                    Point3D::new([-3.0, 2.10, 0.0]),
                    Point3D::new([-3.0, 1.65, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.5, 2.10, 0.0]),
                    Point3D::new([-2.5, 2.10, 0.0]),
                    Point3D::new([-3.0, 2.10, 0.0]),
                    Point3D::new([-3.0, 1.65, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.5, 2.10, -0.3]),
                    Point3D::new([-2.5, 2.10, -0.3]),
                    Point3D::new([-3.0, 2.10, -0.3]),
                    Point3D::new([-3.0, 1.65, -0.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.6, 1.875, -0.3]),
                    Point3D::new([-2.3, 1.875, -0.3]),
                    Point3D::new([-2.7, 1.875, -0.3]),
                    Point3D::new([-2.7, 1.650, -0.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-1.6, 1.875, 0.0]),
                    Point3D::new([-2.3, 1.875, 0.0]),
                    Point3D::new([-2.7, 1.875, 0.0]),
                    Point3D::new([-2.7, 1.650, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([-2.7, 1.650, 0.0]),
                    Point3D::new([-2.7, 1.425, 0.0]),
                    Point3D::new([-2.5, 0.975, 0.0]),
                    Point3D::new([-2.0, 0.750, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-2.7, 1.650, 0.3]),
                    Point3D::new([-2.7, 1.425, 0.3]),
                    Point3D::new([-2.5, 0.975, 0.3]),
                    Point3D::new([-2.0, 0.750, 0.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-3.00, 1.6500, 0.3]),
                    Point3D::new([-3.00, 1.2000, 0.3]),
                    Point3D::new([-2.65, 0.7875, 0.3]),
                    Point3D::new([-1.90, 0.4500, 0.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-3.00, 1.6500, 0.0]),
                    Point3D::new([-3.00, 1.2000, 0.0]),
                    Point3D::new([-2.65, 0.7875, 0.0]),
                    Point3D::new([-1.90, 0.4500, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([-3.00, 1.6500, 0.0]),
                    Point3D::new([-3.00, 1.2000, 0.0]),
                    Point3D::new([-2.65, 0.7875, 0.0]),
                    Point3D::new([-1.90, 0.4500, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-3.00, 1.6500, -0.3]),
                    Point3D::new([-3.00, 1.2000, -0.3]),
                    Point3D::new([-2.65, 0.7875, -0.3]),
                    Point3D::new([-1.90, 0.4500, -0.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-2.7, 1.650, -0.3]),
                    Point3D::new([-2.7, 1.425, -0.3]),
                    Point3D::new([-2.5, 0.975, -0.3]),
                    Point3D::new([-2.0, 0.750, -0.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-2.7, 1.650, 0.0]),
                    Point3D::new([-2.7, 1.425, 0.0]),
                    Point3D::new([-2.5, 0.975, 0.0]),
                    Point3D::new([-2.0, 0.750, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.7, 1.275, 0.0]),
                    Point3D::new([2.6, 1.275, 0.0]),
                    Point3D::new([2.3, 1.950, 0.0]),
                    Point3D::new([2.7, 2.250, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.7, 1.275, 0.66]),
                    Point3D::new([2.6, 1.275, 0.66]),
                    Point3D::new([2.3, 1.950, 0.25]),
                    Point3D::new([2.7, 2.250, 0.25]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.7, 0.450, 0.66]),
                    Point3D::new([3.1, 0.675, 0.66]),
                    Point3D::new([2.4, 1.875, 0.25]),
                    Point3D::new([3.3, 2.250, 0.25]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.7, 0.450, 0.0]),
                    Point3D::new([3.1, 0.675, 0.0]),
                    Point3D::new([2.4, 1.875, 0.0]),
                    Point3D::new([3.3, 2.250, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.7, 0.450, 0.0]),
                    Point3D::new([3.1, 0.675, 0.0]),
                    Point3D::new([2.4, 1.875, 0.0]),
                    Point3D::new([3.3, 2.250, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.7, 0.450, -0.66]),
                    Point3D::new([3.1, 0.675, -0.66]),
                    Point3D::new([2.4, 1.875, -0.25]),
                    Point3D::new([3.3, 2.250, -0.25]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.7, 1.275, -0.66]),
                    Point3D::new([2.6, 1.275, -0.66]),
                    Point3D::new([2.3, 1.950, -0.25]),
                    Point3D::new([2.7, 2.250, -0.25]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([1.7, 1.275, 0.0]),
                    Point3D::new([2.6, 1.275, 0.0]),
                    Point3D::new([2.3, 1.950, 0.0]),
                    Point3D::new([2.7, 2.250, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([2.7, 2.250, 0.0]),
                    Point3D::new([2.8, 2.325, 0.0]),
                    Point3D::new([2.9, 2.325, 0.0]),
                    Point3D::new([2.8, 2.250, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([2.7, 2.250, 0.25]),
                    Point3D::new([2.8, 2.325, 0.25]),
                    Point3D::new([2.9, 2.325, 0.15]),
                    Point3D::new([2.8, 2.250, 0.15]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([3.300, 2.25000, 0.25]),
                    Point3D::new([3.525, 2.34375, 0.25]),
                    Point3D::new([3.450, 2.36250, 0.15]),
                    Point3D::new([3.200, 2.25000, 0.15]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([3.300, 2.25000, 0.0]),
                    Point3D::new([3.525, 2.34375, 0.0]),
                    Point3D::new([3.450, 2.36250, 0.0]),
                    Point3D::new([3.200, 2.25000, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([3.300, 2.25000, 0.0]),
                    Point3D::new([3.525, 2.34375, 0.0]),
                    Point3D::new([3.450, 2.36250, 0.0]),
                    Point3D::new([3.200, 2.25000, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([3.300, 2.25000, -0.25]),
                    Point3D::new([3.525, 2.34375, -0.25]),
                    Point3D::new([3.450, 2.36250, -0.15]),
                    Point3D::new([3.200, 2.25000, -0.15]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([2.7, 2.250, -0.25]),
                    Point3D::new([2.8, 2.325, -0.25]),
                    Point3D::new([2.9, 2.325, -0.15]),
                    Point3D::new([2.8, 2.250, -0.15]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([2.7, 2.250, 0.0]),
                    Point3D::new([2.8, 2.325, 0.0]),
                    Point3D::new([2.9, 2.325, 0.0]),
                    Point3D::new([2.8, 2.250, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.01, 3.00, 0.0]),
                    Point3D::new([0.80, 3.00, 0.0]),
                    Point3D::new([0.00, 2.70, 0.0]),
                    Point3D::new([0.20, 2.55, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 3.00, 0.010]),
                    Point3D::new([0.8, 3.00, 0.450]),
                    Point3D::new([0.0, 2.70, 0.000]),
                    Point3D::new([0.2, 2.55, 0.112]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.010, 3.00, 0.0]),
                    Point3D::new([0.450, 3.00, 0.8]),
                    Point3D::new([0.000, 2.70, 0.0]),
                    Point3D::new([0.112, 2.55, 0.2]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 3.00, 0.01]),
                    Point3D::new([0.0, 3.00, 0.80]),
                    Point3D::new([0.0, 2.70, 0.00]),
                    Point3D::new([0.0, 2.55, 0.20]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 3.00, 0.01]),
                    Point3D::new([0.0, 3.00, 0.80]),
                    Point3D::new([0.0, 2.70, 0.00]),
                    Point3D::new([0.0, 2.55, 0.20]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.010, 3.00, 0.0]),
                    Point3D::new([-0.450, 3.00, 0.8]),
                    Point3D::new([00.000, 2.70, 0.0]),
                    Point3D::new([-0.112, 2.55, 0.2]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([00.0, 3.00, 0.010]),
                    Point3D::new([-0.8, 3.00, 0.450]),
                    Point3D::new([00.0, 2.70, 0.000]),
                    Point3D::new([-0.2, 2.55, 0.112]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.01, 3.00, 0.0]),
                    Point3D::new([-0.80, 3.00, 0.0]),
                    Point3D::new([00.00, 2.70, 0.0]),
                    Point3D::new([-0.20, 2.55, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.01, 3.00, 0.0]),
                    Point3D::new([-0.80, 3.00, 0.0]),
                    Point3D::new([00.00, 2.70, 0.0]),
                    Point3D::new([-0.20, 2.55, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([00.0, 3.00, -0.010]),
                    Point3D::new([-0.8, 3.00, -0.450]),
                    Point3D::new([00.0, 2.70, 00.000]),
                    Point3D::new([-0.2, 2.55, -0.112]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.010, 3.00, 00.0]),
                    Point3D::new([-0.450, 3.00, -0.8]),
                    Point3D::new([00.000, 2.70, 00.0]),
                    Point3D::new([-0.112, 2.55, -0.2]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 3.00, -0.01]),
                    Point3D::new([0.0, 3.00, -0.80]),
                    Point3D::new([0.0, 2.70, 00.00]),
                    Point3D::new([0.0, 2.55, -0.20]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 3.00, -0.01]),
                    Point3D::new([0.0, 3.00, -0.80]),
                    Point3D::new([0.0, 2.70, 00.00]),
                    Point3D::new([0.0, 2.55, -0.20]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.010, 3.00, 00.0]),
                    Point3D::new([0.450, 3.00, -0.8]),
                    Point3D::new([0.000, 2.70, 00.0]),
                    Point3D::new([0.112, 2.55, -0.2]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 3.00, -0.010]),
                    Point3D::new([0.8, 3.00, -0.450]),
                    Point3D::new([0.0, 2.70, 00.000]),
                    Point3D::new([0.2, 2.55, -0.112]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.01, 3.00, 0.0]),
                    Point3D::new([0.80, 3.00, 0.0]),
                    Point3D::new([0.00, 2.70, 0.0]),
                    Point3D::new([0.20, 2.55, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.2, 2.55, 0.0]),
                    Point3D::new([0.4, 2.40, 0.0]),
                    Point3D::new([1.3, 2.40, 0.0]),
                    Point3D::new([1.3, 2.25, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.2, 2.55, 0.112]),
                    Point3D::new([0.4, 2.40, 0.224]),
                    Point3D::new([1.3, 2.40, 0.728]),
                    Point3D::new([1.3, 2.25, 0.728]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.112, 2.55, 0.2]),
                    Point3D::new([0.224, 2.40, 0.4]),
                    Point3D::new([0.728, 2.40, 1.3]),
                    Point3D::new([0.728, 2.25, 1.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 2.55, 0.2]),
                    Point3D::new([0.0, 2.40, 0.4]),
                    Point3D::new([0.0, 2.40, 1.3]),
                    Point3D::new([0.0, 2.25, 1.3]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 2.55, 0.2]),
                    Point3D::new([0.0, 2.40, 0.4]),
                    Point3D::new([0.0, 2.40, 1.3]),
                    Point3D::new([0.0, 2.25, 1.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.112, 2.55, 0.2]),
                    Point3D::new([-0.224, 2.40, 0.4]),
                    Point3D::new([-0.728, 2.40, 1.3]),
                    Point3D::new([-0.728, 2.25, 1.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.2, 2.55, 0.112]),
                    Point3D::new([-0.4, 2.40, 0.224]),
                    Point3D::new([-1.3, 2.40, 0.728]),
                    Point3D::new([-1.3, 2.25, 0.728]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.2, 2.55, 0.0]),
                    Point3D::new([-0.4, 2.40, 0.0]),
                    Point3D::new([-1.3, 2.40, 0.0]),
                    Point3D::new([-1.3, 2.25, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.2, 2.55, 0.0]),
                    Point3D::new([-0.4, 2.40, 0.0]),
                    Point3D::new([-1.3, 2.40, 0.0]),
                    Point3D::new([-1.3, 2.25, 0.0]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.2, 2.55, -0.112]),
                    Point3D::new([-0.4, 2.40, -0.224]),
                    Point3D::new([-1.3, 2.40, -0.728]),
                    Point3D::new([-1.3, 2.25, -0.728]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([-0.112, 2.55, -0.2]),
                    Point3D::new([-0.224, 2.40, -0.4]),
                    Point3D::new([-0.728, 2.40, -1.3]),
                    Point3D::new([-0.728, 2.25, -1.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 2.55, -0.2]),
                    Point3D::new([0.0, 2.40, -0.4]),
                    Point3D::new([0.0, 2.40, -1.3]),
                    Point3D::new([0.0, 2.25, -1.3]),
                ]),
            ]),
            mat.clone(),
        )));
        build_world.add(Arc::new(BezierPatch::new(
            BezierSurface::from_curves(vec![
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.0, 2.55, -0.2]),
                    Point3D::new([0.0, 2.40, -0.4]),
                    Point3D::new([0.0, 2.40, -1.3]),
                    Point3D::new([0.0, 2.25, -1.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.112, 2.55, -0.2]),
                    Point3D::new([0.224, 2.40, -0.4]),
                    Point3D::new([0.728, 2.40, -1.3]),
                    Point3D::new([0.728, 2.25, -1.3]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.2, 2.55, -0.112]),
                    Point3D::new([0.4, 2.40, -0.224]),
                    Point3D::new([1.3, 2.40, -0.728]),
                    Point3D::new([1.3, 2.25, -0.728]),
                ]),
                BezierCurve3D::from_points(vec![
                    Point3D::new([0.2, 2.55, 0.0]),
                    Point3D::new([0.4, 2.40, 0.0]),
                    Point3D::new([1.3, 2.40, 0.0]),
                    Point3D::new([1.3, 2.25, 0.0]),
                ]),
            ]),
            mat.clone(),
        )));
    }

    // Make the world into a bvh
    let mut world = HittableList::new();
    world.add(Arc::new(BVHNode::from_list(&build_world)));

    // Camera
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = width;
    cam.samples_per_pixel = samples;
    cam.max_depth = depth;

    cam.vfov = 35.0;
    cam.look_from = Vec3::new(5.0, 5.0, 5.0);
    cam.look_at = Vec3::new(0.0, 1.0, 0.0);
    cam.v_up = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;

    let world = Arc::new(world);
    let buffer = cam.render(world, None, true, threads);
    if save {
        save_image(
            "Teapot",
            &buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
            samples,
            depth,
        );
    }
    if show {
        render_buffer(
            buffer,
            cam.render_width() as u32,
            cam.render_height() as u32,
        );
    }
}
