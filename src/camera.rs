struct Camera
{
    pub position: [fractional_type::FractionalType; 3],
    pub rotation: [fractional_type::FractionalType; 3],
    pub fov: fractional_type::FractionalType,

    intrinsic_matrix: rulinalg::matrix::Matrix<fractional_type::FractionalType>,
    rotation_matrix: rulinalg::matrix::Matrix<fractional_type::FractionalType>,
    translation_matrix: rulinalg::matrix::Matrix<fractional_type::FractionalType>,
}

// impl Camera
// {
//     fn rot_left_right()
// }