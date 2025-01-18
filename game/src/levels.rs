use crate::shape::loader::GameLevel;

pub const LEVELS: [GameLevel; 20] = [
    GameLevel::tetrahedron(1, 1),
    GameLevel::cube(2, 2),
    GameLevel::octahedron(3, 3),
    GameLevel::dodecahedron(1),
    GameLevel::icosahedron(2, 2),
    GameLevel::octahedron(4, 4),
    GameLevel::tetrahedron(6, 0),
    GameLevel::cube(4, 3),
    GameLevel::tetrahedron(7, 0),
    GameLevel::octahedron(5, 0),
    GameLevel::icosahedron(3, 2),
    GameLevel::tetrahedron(8, 0),
    GameLevel::cube(5, 0),
    GameLevel::octahedron(6, 0),
    GameLevel::tetrahedron(9, 0),
    GameLevel::icosahedron(4, 2),
    GameLevel::cube(6, 1),
    GameLevel::octahedron(7, 0),
    GameLevel::cube(7, 0),
    GameLevel::icosahedron(5, 0),
];
