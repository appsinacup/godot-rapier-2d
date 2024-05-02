use crate::rapier2d::convert::*;
use crate::rapier2d::handle::*;
use crate::rapier2d::physics_world::*;
use crate::rapier2d::vector::Vector;
use godot::builtin::Transform2D;
use rapier2d::prelude::*;

pub fn pixel_point_array_to_vec(pixel_data: Vec<Vector>) -> Vec<Point<Real>> {
    let mut vec = Vec::<Point<Real>>::with_capacity(pixel_data.len());
    for pixel_point in pixel_data {
        let point = &vector_pixels_to_meters(&pixel_point);
        vec.push(Point::<Real> {
            coords: vector![point.x, point.y],
        });
    }
    return vec;
}

#[derive(Clone)]
pub struct ShapeInfo {
    pub handle: Handle,
    pub pixel_position: Vector,
    pub rotation: Real,
    pub skew: Real,
    pub scale: Vector,
}

pub fn shape_info_from_body_shape(shape_handle: Handle, transform: Transform2D) -> ShapeInfo {
    let origin = transform.origin;
    let scale = transform.scale();
    return ShapeInfo {
        handle: shape_handle,
        pixel_position: Vector::new(origin.x, origin.y),
        rotation: transform.rotation(),
        skew: transform.skew(),
        scale: Vector::new(scale.x, scale.y),
    };
}

pub fn shape_create_box(pixel_size: &Vector) -> Handle {
    let size = &vector_pixels_to_meters(pixel_size);
    let shape = SharedShape::cuboid(0.5 * size.x, 0.5 * size.y);
    let mut physics_engine = singleton().lock().unwrap();
    return physics_engine.insert_shape(shape);
}

pub fn shape_create_halfspace(normal: &Vector, pixel_distance: Real) -> Handle {
    let distance = pixels_to_meters(pixel_distance);

    let shape = SharedShape::halfspace(UnitVector::new_normalize(vector![normal.x, -normal.y]));
    let shape_position = Isometry::new(vector![normal.x * distance, normal.y * distance], 0.0);
    let mut shapes_vec = Vec::<(Isometry<Real>, SharedShape)>::new();
    shapes_vec.push((shape_position, shape));
    let shape_compound = SharedShape::compound(shapes_vec);
    let mut physics_engine = singleton().lock().unwrap();
    return physics_engine.insert_shape(shape_compound);
}

pub fn shape_create_circle(pixel_radius: Real) -> Handle {
    let radius = pixels_to_meters(pixel_radius);
    let shape = SharedShape::ball(radius);
    let mut physics_engine = singleton().lock().unwrap();
    return physics_engine.insert_shape(shape);
}

pub fn shape_create_capsule(pixel_half_height: Real, pixel_radius: Real) -> Handle {
    let half_height = pixels_to_meters(pixel_half_height);
    let radius = pixels_to_meters(pixel_radius);

    let shape = SharedShape::capsule_y(half_height, radius);
    let mut physics_engine = singleton().lock().unwrap();
    return physics_engine.insert_shape(shape);
}

pub fn shape_create_convex_polyline(pixel_points: Vec<Vector>) -> Handle {
    let points_vec = pixel_point_array_to_vec(pixel_points);
    let shape_data = SharedShape::convex_polyline(points_vec);
    if shape_data.is_none() {
        return Handle::default();
    }
    let shape = shape_data.unwrap();
    let mut physics_engine = singleton().lock().unwrap();
    return physics_engine.insert_shape(shape);
}

pub fn shape_create_concave_polyline(pixel_points: Vec<Vector>) -> Handle {
    let points_vec = pixel_point_array_to_vec(pixel_points);
    let shape = SharedShape::polyline(points_vec, None);
    let mut physics_engine = singleton().lock().unwrap();
    return physics_engine.insert_shape(shape);
}

pub fn shape_destroy(shape_handle: Handle) {
    let mut physics_engine = singleton().lock().unwrap();
    return physics_engine.remove_shape(shape_handle);
}
