pub mod fractional_type;

use fractional_type::FractionalType;
use num_traits::identities::Zero;
use rulinalg::vector::Vector;
use sdl2::pixels::Color;

const RESOLUTION_X: u32 = 640;
const RESOLUTION_Y: u32 = 480;
const FOCAL_LENGTH: f32 = 1.0;

fn to_proper_inf(num: FractionalType) -> FractionalType
{
    if num >= FractionalType::zero()
    {
        // TODO: had to change this to 10000 and -10000 or else sdl2 would 
        // allocate a 2^31^2 frame buffer and run out of memory
        FractionalType::from_num(10000)//i32::MAX
    }
    else
    {
        FractionalType::from_num(-10000)//i32::MIN
    }
}

fn homogenous_image_coordinates_to_image_coordinates(homogeneous_coordinates: &Vector<FractionalType>) -> Vector<FractionalType>
{
    if homogeneous_coordinates[2] == FractionalType::zero()
    {
        rulinalg::vector![
            to_proper_inf(homogeneous_coordinates[0]),
            to_proper_inf(homogeneous_coordinates[1])
        ]
    }
    else
    {
        rulinalg::vector![
            homogeneous_coordinates[0] / homogeneous_coordinates[2],
            homogeneous_coordinates[1] / homogeneous_coordinates[2]
        ]
    }
}

// struct for a triangle with a flat bottom or a flat top
struct FlatTriangle<'a>
{
    v_odd: &'a Vector<FractionalType>,
    v_left: &'a Vector<FractionalType>,
    v_right: &'a Vector<FractionalType>
}

struct NonFlatTriangle<'a>
{
    top: &'a Vector<FractionalType>,
    mid: &'a Vector<FractionalType>,
    bottom: &'a Vector<FractionalType>
}

enum TriangleType<'a>
{
    FlatTop(FlatTriangle<'a>),
    FlatBottom(FlatTriangle<'a>),
    NonFlat(NonFlatTriangle<'a>),
    NotATriangle
}

fn categorize_triangle<'a>(v0: &'a Vector<FractionalType>, v1: &'a Vector<FractionalType>, v2: &'a Vector<FractionalType>) -> TriangleType<'a>
{
    // assert no two occupy the same position
    if v0[0] == v1[0] &&
        v0[1] == v1[1]
    {
        return TriangleType::NotATriangle;
    }
    if v0[0] == v2[0] &&
        v0[1] == v2[1]
    {
        return TriangleType::NotATriangle;
    }
    if v2[0] == v1[0] &&
        v2[1] == v1[1]
    {
        return TriangleType::NotATriangle;
    }

    // assert they are not colinear horizontally
    if v0[1] == v1[1] &&
        v1[1] == v2[1]
    {
        return TriangleType::NotATriangle;
    }
    // handle the case where they are colinear along other lines
    // after the NonFlat is handled, which makes the colinear visible
    // by the above cases

    let mut vs: Vec<&Vector<FractionalType>> = vec![v0, v1, v2];
    vs.sort_by(|a, b| b[1].cmp(&a[1]));
 
    if vs[0][1] == vs[1][1]
    {
        // flat bottom
        if vs[0][0] < vs[1][0]
        {
            // vs[0] is left
            TriangleType::FlatBottom(FlatTriangle{
                v_odd: vs[2],
                v_left: vs[0],
                v_right: vs[1]
            })
        }
        else
        {
            // vs[0] is right
            TriangleType::FlatBottom(FlatTriangle{
                v_odd: vs[2],
                v_left: vs[1],
                v_right: vs[0]
            })
        }
    }
    else if vs[1][1] == vs[2][1]
    {
        //flat top
        if vs[1][0] < vs[2][0]
        {
            // vs[1] is left
            TriangleType::FlatBottom(FlatTriangle{
                v_odd: vs[0],
                v_left: vs[1],
                v_right: vs[2]
            })
        }
        else
        {
            // vs[2] is right
            TriangleType::FlatBottom(FlatTriangle{
                v_odd: vs[0],
                v_left: vs[2],
                v_right: vs[1]
            })
        }
    }
    else
    {
        TriangleType::NonFlat(NonFlatTriangle{
            top: vs[2],
            mid: vs[1],
            bottom: vs[0]
        })
    }
}

fn rasterize_flat_bottom_triangle(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, flat_bottom: &FlatTriangle, c: &Color)
{
    let mut row_coordinate = (flat_bottom.v_odd[0] - FractionalType::from_num(0.5)).ceil() + FractionalType::from_num(0.5);
    while row_coordinate < flat_bottom.v_left[0]
    {
        let col_coord_reciprocal_slope = (flat_bottom.v_odd[0] - flat_bottom.v_left[0]) / (flat_bottom.v_odd[1] - flat_bottom.v_left[1]);
        let mut col_coord = col_coord_reciprocal_slope * flat_bottom.v_odd[1] + flat_bottom.v_odd[0] - col_coord_reciprocal_slope * row_coordinate;
        col_coord = (col_coord - FractionalType::from_num(0.5)).ceil() + FractionalType::from_num(0.5);
        //let mut col_coordinate = 
        let col_guard_reciprocal_slope = (flat_bottom.v_odd[0] - flat_bottom.v_right[0]) / (flat_bottom.v_odd[1] - flat_bottom.v_right[1]);
        let col_guard = col_guard_reciprocal_slope * flat_bottom.v_odd[1] + flat_bottom.v_odd[0] - col_guard_reciprocal_slope * row_coordinate;
        while col_coord < col_guard
        {
            println!("drawing point: ({}, {})", col_coord.to_num::<u32>(), row_coordinate.to_num::<u32>());
            let _ = canvas.draw_point((col_coord.to_num(), row_coordinate.to_num()));
            col_coord = col_coord + FractionalType::from_num(1);
        }
        row_coordinate = row_coordinate + FractionalType::from_num(1);
    }
}

fn rasterize_flat_top_triangle(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, flat_top: &FlatTriangle, c: &Color)
{

}

fn rasterize_non_flat_triangle(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, non_flat: &NonFlatTriangle, c: &Color)
{
    let reciprocal_slope = (non_flat.top[0] - non_flat.bottom[0]) / (non_flat.top[1] - non_flat.bottom[1]);
    let intersection: Vector<FractionalType> = rulinalg::vector![
        reciprocal_slope * non_flat.top[1] + non_flat.top[0] - reciprocal_slope * non_flat.mid[1],
        non_flat.mid[1]
    ];
    if intersection[0] > non_flat.mid[0]
    {
        // intersection is right
        rasterize_flat_bottom_triangle(
            canvas,
            &FlatTriangle{
                v_odd: non_flat.top,
                v_left: non_flat.mid,
                v_right: &intersection
            },
            c);
        rasterize_flat_top_triangle(
            canvas,
            &FlatTriangle{
                v_odd: non_flat.bottom,
                v_left: non_flat.mid,
                v_right: &intersection
            },
            c);
    }
    else
    {
        // intersection is left
        rasterize_flat_bottom_triangle(
            canvas,
            &FlatTriangle{
                v_odd: non_flat.top,
                v_left: &intersection,
                v_right: non_flat.mid
            },
            c);
        rasterize_flat_top_triangle(
            canvas,
            &FlatTriangle{
                v_odd: non_flat.bottom,
                v_left: &intersection,
                v_right: non_flat.mid
            },
            c);
    }
}

fn rasterize(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, v0: &Vector<FractionalType>, v1: &Vector<FractionalType>, v2: &Vector<FractionalType>, c: &Color)
{
    match categorize_triangle(v0, v1, v2)
    {
        TriangleType::FlatTop(flat_top) => rasterize_flat_top_triangle(canvas, &flat_top, c),
        TriangleType::FlatBottom(flat_bottom) => rasterize_flat_bottom_triangle(canvas, &flat_bottom, c),
        TriangleType::NonFlat(non_flat) => rasterize_non_flat_triangle(canvas, &non_flat, c),
        TriangleType::NotATriangle => (),
    }
}

fn main() {
    use rulinalg::matrix::Matrix;
    // model stuff
    let teapot_path = std::path::Path::new("res/mdl/hello/hello.obj");
    let teapot_mdl = obj::Obj::load(teapot_path).unwrap();

    // object -> group -> polygon -> triangle
    let mut teapot_triangle_colors: Vec<Vec<Vec<Vec<Color>>>> = Vec::new();
    for object in &teapot_mdl.data.objects
    {
        teapot_triangle_colors.push(Vec::new());
        let color_obj_idx = teapot_triangle_colors.len() - 1;
        let color_obj_vec = &mut teapot_triangle_colors[color_obj_idx];
        for group in &object.groups
        {
            color_obj_vec.push(Vec::new());
            let color_obj_grp_idx = color_obj_vec.len() - 1;
            let color_obj_grp_vec = &mut color_obj_vec[color_obj_grp_idx];
            for polygon in &group.polys
            {
                if polygon.0.len() < 3
                {
                    continue;
                }
                color_obj_grp_vec.push(Vec::new());
                let color_obj_grp_poly_idx = color_obj_grp_vec.len() - 1;
                let color_obj_grp_poly_vec = &mut color_obj_grp_vec[color_obj_grp_poly_idx];
                for _ in 2..polygon.0.len()
                {
                    let random_color = 127u8 + (rand::random::<u8>() >> 2);
                    color_obj_grp_poly_vec.push(Color::RGB(random_color, random_color, random_color));
                }
            }
        }
    }

    let knight_path = std::path::Path::new("res/mdl/psx-knight/psx-knight.gltf");
    let knight_gltf = gltf::Gltf::open(knight_path).unwrap();

    // camera stuff
    // https://www.cs.cmu.edu/~16385/s17/Slides/11.1_Camera_matrix.pdf
    // P = K * R * [I | -C]
    // where P is the camera projection, K is the 3x3 intrinsics, R is the 3D rotation matrix, -C is the negative 3D translation

    // these variables are modified directly by user input
    let mut camera_position: (f32, f32) = (0.0, -5.0); // x, z
    let mut camera_facing: (f32, f32) = (0.0, 0.0); // left-right rotation (about y), upward rotation (about x)

    // Intrinsic matrix K = (f 0 p_x)
    //                      (0 f p_y)
    //                      (0 0  1 )
    // where f is the focal length, p is translation from camera coordinate system to image coordinate system
    let camera_matrix_image_focal_length: Matrix<FractionalType> = rulinalg::matrix![
        FractionalType::from_num(FOCAL_LENGTH), FractionalType::from_num(0),            FractionalType::from_num(0);
        FractionalType::from_num(0),            FractionalType::from_num(FOCAL_LENGTH), FractionalType::from_num(0);
        FractionalType::from_num(0),            FractionalType::from_num(0),            FractionalType::from_num(1)
    ];
    let camera_matrix_image_flip: Matrix<FractionalType> = rulinalg::matrix![
        FractionalType::from_num(1), FractionalType::from_num(0),  FractionalType::from_num(0);
        FractionalType::from_num(0), FractionalType::from_num(-1), FractionalType::from_num(0);
        FractionalType::from_num(0), FractionalType::from_num(0),  FractionalType::from_num(1)
    ];
    let camera_matrix_image_translation: Matrix<FractionalType> = rulinalg::matrix![
        FractionalType::from_num(1), FractionalType::from_num(0), FractionalType::from_num(1);
        FractionalType::from_num(0), FractionalType::from_num(1), FractionalType::from_num(1);
        FractionalType::from_num(0), FractionalType::from_num(0), FractionalType::from_num(1)
    ];
    let camera_matrix_image_scale: Matrix<FractionalType> = rulinalg::matrix![
        FractionalType::from_num(RESOLUTION_X) / FractionalType::from_num(2), FractionalType::from_num(0),                                          FractionalType::from_num(0);
        FractionalType::from_num(0),                                          FractionalType::from_num(RESOLUTION_Y) / FractionalType::from_num(2), FractionalType::from_num(0);
        FractionalType::from_num(0),                                          FractionalType::from_num(0),                                          FractionalType::from_num(1)
    ];
    let camera_matrix_intrinsics = camera_matrix_image_scale * camera_matrix_image_translation * camera_matrix_image_flip * camera_matrix_image_focal_length;

    // sdl2 stuff
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("psx-renderer", 640, 480)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.clear();

        //////////////////
        //  USER INPUT  //
        //////////////////
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit {..} |
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Escape), .. } => {
                    break 'running
                },
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::W), .. } => {
                    camera_position.1 = camera_position.1 + 0.1;
                },
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::A), .. } => {
                    camera_position.0 = camera_position.0 - 0.1;
                },
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::S), .. } => {
                    camera_position.1 = camera_position.1 - 0.1;
                },
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::D), .. } => {
                    camera_position.0 = camera_position.0 + 0.1;
                },
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::O), .. } => {
                    camera_facing.1 = camera_facing.1 + 0.1;
                },
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::K), .. } => {
                    camera_facing.0 = camera_facing.0 + 0.1;
                },
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::L), .. } => {
                    camera_facing.1 = camera_facing.1 - 0.1;
                },
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::Semicolon), .. } => {
                    camera_facing.0 = camera_facing.0 - 0.1;
                },
                _ => {}
            }
        }

        ///////////////////////////
        // UPDATE CAMERA MATRIX  //
        ///////////////////////////
        let camera_matrix_world_translation: Matrix<FractionalType> = rulinalg::matrix![
            FractionalType::from_num(1), FractionalType::from_num(0), FractionalType::from_num(0), FractionalType::from_num(-camera_position.0);
            FractionalType::from_num(0), FractionalType::from_num(1), FractionalType::from_num(0), FractionalType::from_num(0);
            FractionalType::from_num(0), FractionalType::from_num(0), FractionalType::from_num(1), FractionalType::from_num(-camera_position.1)
        ];
        let camera_matrix_world_rotation_y: Matrix<FractionalType> = rulinalg::matrix![
            FractionalType::from_num(camera_facing.0.cos()), FractionalType::from_num(0), FractionalType::from_num(camera_facing.0.sin());
            FractionalType::from_num(0), FractionalType::from_num(1), FractionalType::from_num(0);
            FractionalType::from_num(-camera_facing.0.sin()), FractionalType::from_num(0), FractionalType::from_num(camera_facing.0.cos())
        ];
        let camera_matrix_world_rotation_x: Matrix<FractionalType> = rulinalg::matrix![
            FractionalType::from_num(1), FractionalType::from_num(0), FractionalType::from_num(0);
            FractionalType::from_num(0), FractionalType::from_num(camera_facing.1.cos()), FractionalType::from_num(-camera_facing.1.sin());
            FractionalType::from_num(0), FractionalType::from_num(camera_facing.1.sin()), FractionalType::from_num(camera_facing.1.cos())
        ];
        let camera_matrix = &camera_matrix_intrinsics * &camera_matrix_world_rotation_x* &camera_matrix_world_rotation_y * &camera_matrix_world_translation;

        /////////////////////////////
        // TRIANGLE RASTERIZATION  //
        /////////////////////////////
        for object in &teapot_mdl.data.objects
        {
            for group in &object.groups
            {
                for polygon in &group.polys
                {
                    if polygon.0.len() < 3
                    {
                        continue;
                    }
                    // we're going to draw all the polygons by splitting it up into triangles
                    // each triangle will use v_0 as a common vertex, then use vertex v_(i - 1) and 
                    // v_i as the other two vertices. Before the loop, we just draw the line between v_0 and v_1
                    // Each loop iteration, we draw two lines, one from v_0 to v_(i - 1) and one from v_0 to v_i
                    let v0_idx = polygon.0[0].0;
                    let v0_array = &teapot_mdl.data.position[v0_idx];
                    let v0_world_homogeneous = rulinalg::vector![
                        FractionalType::from_num(v0_array[0]),
                        FractionalType::from_num(v0_array[1]),
                        FractionalType::from_num(v0_array[2]),
                        FractionalType::from_num(1)
                    ];
                    let v0_im_homogeneous = &camera_matrix * v0_world_homogeneous;
                    let v0_im_frac = homogenous_image_coordinates_to_image_coordinates(&v0_im_homogeneous);


                    let v1_idx = polygon.0[0].0;
                    let v1_array = &teapot_mdl.data.position[v1_idx];
                    let v1_world_homogeneous = rulinalg::vector![
                        FractionalType::from_num(v1_array[0]),
                        FractionalType::from_num(v1_array[1]),
                        FractionalType::from_num(v1_array[2]),
                        FractionalType::from_num(1)
                    ];
                    let v1_im_homogeneous = &camera_matrix * v1_world_homogeneous;
                    let mut v1_im_frac = homogenous_image_coordinates_to_image_coordinates(&v1_im_homogeneous);

                    for i in 2..polygon.0.len()
                    {
                        let color = Color::RGB(127, 127, 127);
                        let v2_idx = polygon.0[i].0;
                        let v2_array = &teapot_mdl.data.position[v2_idx];
                        let v2_world_homogeneous = rulinalg::vector![
                            FractionalType::from_num(v2_array[0]),
                            FractionalType::from_num(v2_array[1]),
                            FractionalType::from_num(v2_array[2]),
                            FractionalType::from_num(1)
                        ];
                        let v2_im_homogeneous = &camera_matrix * v2_world_homogeneous;
                        let v2_im_frac = homogenous_image_coordinates_to_image_coordinates(&v2_im_homogeneous);

                        rasterize(&mut canvas, &v0_im_frac, &v1_im_frac, &v2_im_frac, &color);

                        v1_im_frac = v2_im_frac;
                    }
                }
            }
        }

        ////////////////////
        //  LINE DRAWING  //
        ////////////////////
        canvas.set_draw_color(sdl2::pixels::Color::RGB(191, 191, 191));
        for object in &teapot_mdl.data.objects
        {
            for group in &object.groups
            {
                for polygon in &group.polys
                {
                    if polygon.0.len() < 3
                    {
                        continue;
                    }
                    // we're going to draw all the polygons by splitting it up into triangles
                    // each triangle will use v_0 as a common vertex, then use vertex v_(i - 1) and 
                    // v_i as the other two vertices. Before the loop, we just draw the line between v_0 and v_1
                    // Each loop iteration, we draw two lines, one from v_0 to v_(i - 1) and one from v_0 to v_i
                    let v0_idx = polygon.0[0].0;
                    let v0_array = &teapot_mdl.data.position[v0_idx];
                    let v0_world_homogeneous = rulinalg::vector![
                        FractionalType::from_num(v0_array[0]),
                        FractionalType::from_num(v0_array[1]),
                        FractionalType::from_num(v0_array[2]),
                        FractionalType::from_num(1)
                    ];
                    let v0_im_homogeneous = &camera_matrix * v0_world_homogeneous;
                    let v0_im_frac = homogenous_image_coordinates_to_image_coordinates(&v0_im_homogeneous);
                    let v0_im: (i32, i32) = (v0_im_frac[0].to_num(), v0_im_frac[1].to_num());


                    let v1_idx = polygon.0[0].0;
                    let v1_array = &teapot_mdl.data.position[v1_idx];
                    let v1_world_homogeneous = rulinalg::vector![
                        FractionalType::from_num(v1_array[0]),
                        FractionalType::from_num(v1_array[1]),
                        FractionalType::from_num(v1_array[2]),
                        FractionalType::from_num(1)
                    ];
                    let v1_im_homogeneous = &camera_matrix * v1_world_homogeneous;
                    let v1_im_frac = homogenous_image_coordinates_to_image_coordinates(&v1_im_homogeneous);
                    let mut v1_im: (i32, i32) = (v1_im_frac[0].to_num(), v1_im_frac[1].to_num());

                    let _ = canvas.draw_line(v0_im, v1_im);

                    for i in 2..polygon.0.len()
                    {
                        let v2_idx = polygon.0[i].0;
                        let v2_array = &teapot_mdl.data.position[v2_idx];
                        let v2_world_homogeneous = rulinalg::vector![
                            FractionalType::from_num(v2_array[0]),
                            FractionalType::from_num(v2_array[1]),
                            FractionalType::from_num(v2_array[2]),
                            FractionalType::from_num(1)
                        ];
                        let v2_im_homogeneous = &camera_matrix * v2_world_homogeneous;
                        let v2_im_frac = homogenous_image_coordinates_to_image_coordinates(&v2_im_homogeneous);
                        let v2_im: (i32, i32) = (v2_im_frac[0].to_num(), v2_im_frac[1].to_num());
                        let _ = canvas.draw_line(v0_im, v2_im);
                        let _ = canvas.draw_line(v1_im, v2_im);

                        v1_im = v2_im;
                    }
                }
            }
        }

        //////////////////////
        //  VERTEX DRAWING  //
        //////////////////////
        canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        for vertex_array in &teapot_mdl.data.position
        {
            let vertex_vec4: Vector<FractionalType> = rulinalg::vector![
                FractionalType::from_num(vertex_array[0]),
                FractionalType::from_num(vertex_array[1]),
                FractionalType::from_num(vertex_array[2]),
                FractionalType::from_num(1)
            ];
            let vertex_vec3 = &camera_matrix * vertex_vec4;
            if vertex_vec3[2] == FractionalType::from_num(0)
            {
                continue;
            }
            let im_coords = (vertex_vec3[0] / vertex_vec3[2], vertex_vec3[1] / vertex_vec3[2]);
            let (im_x, im_y): (i32, i32) = (im_coords.0.to_num(), im_coords.1.to_num());
            if (im_x >= 0 && im_x < 640) &&
                (im_y >= 0 && im_y < 480)
            {
                let _ = canvas.draw_point((im_x, im_y));
            }
        }

        canvas.present();
        ::std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
