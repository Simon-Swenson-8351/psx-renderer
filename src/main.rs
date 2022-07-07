pub mod fractional_type;

static RESOLUTION_X: u32 = 640;
static RESOLUTION_Y: u32 = 480;
static FOCAL_LENGTH: f32 = 1.0;

fn main() {
    use fractional_type::FractionalType;
    use rulinalg::matrix::Matrix;
    use rulinalg::vector::Vector;
    // model stuff
    let teapot_path = std::path::Path::new("res/mdl/monkey/monkey.obj");
    let teapot_mdl = obj::Obj::load(teapot_path).unwrap();

    let knight_path = std::path::Path::new("res/mdl/psx-knight/psx-knight.gltf");
    let knight_gltf = gltf::Gltf::open(knight_path).unwrap();

    // camera stuff
    // https://www.cs.cmu.edu/~16385/s17/Slides/11.1_Camera_matrix.pdf
    // P = K * R * [I | -C]
    // where P is the camera projection, K is the 3x3 intrinsics, R is the 3D rotation matrix, -C is the negative 3D translation
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
                    camera_position.0 = camera_position.0 + 0.1;
                },
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::S), .. } => {
                    camera_position.1 = camera_position.1 - 0.1;
                },
                sdl2::event::Event::KeyDown { keycode: Some(sdl2::keyboard::Keycode::D), .. } => {
                    camera_position.0 = camera_position.0 - 0.1;
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

        
        canvas.set_draw_color(sdl2::pixels::Color::RGB(127, 127, 127));
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
                    for i in 0..(polygon.0.len() - 1)
                    {
                        let v0_idx = polygon.0[i].0;
                        let v1_idx = polygon.0[i + 1].0;
                        let v0_array = &teapot_mdl.data.position[v0_idx];
                        let v1_array = &teapot_mdl.data.position[v1_idx];
                        let v0_world_homogeneous = rulinalg::vector![
                            FractionalType::from_num(v0_array[0]),
                            FractionalType::from_num(v0_array[1]),
                            FractionalType::from_num(v0_array[2]),
                            FractionalType::from_num(1)
                        ];
                        let v1_world_homogeneous = rulinalg::vector![
                            FractionalType::from_num(v1_array[0]),
                            FractionalType::from_num(v1_array[1]),
                            FractionalType::from_num(v1_array[2]),
                            FractionalType::from_num(1)
                        ];
                        let v0_im_homogeneous = &camera_matrix * v0_world_homogeneous;
                        let v1_im_homogeneous = &camera_matrix * v1_world_homogeneous;
                        if v0_im_homogeneous[2] == FractionalType::from_num(0) || 
                            v1_im_homogeneous[2] == FractionalType::from_num(0)
                        {
                            continue;
                        }
                        let v0_im: (i32, i32) = ((v0_im_homogeneous[0] / v0_im_homogeneous[2]).to_num(), (v0_im_homogeneous[1] / v0_im_homogeneous[2]).to_num());
                        let v1_im: (i32, i32) = ((v1_im_homogeneous[0] / v1_im_homogeneous[2]).to_num(), (v1_im_homogeneous[1] / v1_im_homogeneous[2]).to_num());
                        let _ = canvas.draw_line(v0_im, v1_im);
                    }
                }
            }
        }

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
