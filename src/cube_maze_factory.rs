struct CubeMazeFactory {
    subdivisions_per_face: u8,
}

impl CubeMazeFactory {
    fn new(subdivisions_per_face: u8) -> Self {
        Self {
            subdivisions_per_face,
        }
    }
}
