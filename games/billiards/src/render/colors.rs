/// Ball colors as [R, G, B, A] u8 values.
/// Index 0 = cue ball, 1-7 = solids, 8 = eight ball, 9-15 = stripes.
pub const BALL_COLORS: [[u8; 4]; 16] = [
    [255, 255, 255, 255], // 0  White (cue ball)
    [255, 210, 0, 255],   // 1  Yellow
    [0, 60, 200, 255],    // 2  Blue
    [220, 30, 30, 255],   // 3  Red
    [100, 0, 160, 255],   // 4  Purple
    [255, 120, 0, 255],   // 5  Orange
    [0, 140, 60, 255],    // 6  Green
    [140, 50, 20, 255],   // 7  Maroon
    [20, 20, 20, 255],    // 8  Black
    [255, 210, 0, 255],   // 9  Yellow stripe
    [0, 60, 200, 255],    // 10 Blue stripe
    [220, 30, 30, 255],   // 11 Red stripe
    [100, 0, 160, 255],   // 12 Purple stripe
    [255, 120, 0, 255],   // 13 Orange stripe
    [0, 140, 60, 255],    // 14 Green stripe
    [140, 50, 20, 255],   // 15 Maroon stripe
];

/// Metallic value per ball. Stripes (9-15) get higher metallic for visual distinction.
pub const BALL_METALLIC: [f32; 16] = [
    0.05, // 0  cue
    0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, // 1-7 solids
    0.1,                                   // 8
    0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5,   // 9-15 stripes
];

/// Table surface color: dark green felt.
pub const TABLE_COLOR: [u8; 4] = [34, 120, 50, 255];

/// Cushion (rail) color: dark brown wood.
pub const CUSHION_COLOR: [u8; 4] = [101, 67, 33, 255];
