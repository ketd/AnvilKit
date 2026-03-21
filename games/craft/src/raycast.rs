use crate::block::BlockType;
use crate::resources::VoxelWorld;

/// Result of a voxel raycast hit.
#[derive(Debug, Clone)]
pub struct VoxelHit {
    /// Integer coordinates of the hit block.
    pub block_pos: [i32; 3],
    /// Normal of the face that was hit (-1, 0, or 1 per axis).
    pub face_normal: [i32; 3],
    /// Distance from ray origin to hit point.
    pub distance: f32,
}

/// DDA voxel raycast: step through grid cells along the ray.
/// Returns the first non-air obstacle block hit within `max_dist`.
pub fn raycast_voxels(
    world: &VoxelWorld,
    origin: [f32; 3],
    direction: [f32; 3],
    max_dist: f32,
) -> Option<VoxelHit> {
    let ox = origin[0];
    let oy = origin[1];
    let oz = origin[2];
    let dx = direction[0];
    let dy = direction[1];
    let dz = direction[2];

    // Current voxel coordinates
    let mut ix = ox.floor() as i32;
    let mut iy = oy.floor() as i32;
    let mut iz = oz.floor() as i32;

    // Step direction per axis
    let step_x: i32 = if dx > 0.0 { 1 } else { -1 };
    let step_y: i32 = if dy > 0.0 { 1 } else { -1 };
    let step_z: i32 = if dz > 0.0 { 1 } else { -1 };

    // t_delta: how far along the ray (in t) to cross one voxel in each axis
    let t_delta_x = if dx.abs() > 1e-10 { (1.0 / dx).abs() } else { f32::MAX };
    let t_delta_y = if dy.abs() > 1e-10 { (1.0 / dy).abs() } else { f32::MAX };
    let t_delta_z = if dz.abs() > 1e-10 { (1.0 / dz).abs() } else { f32::MAX };

    // t_max: t value at which the ray crosses the next voxel boundary
    let t_max_x = if dx.abs() > 1e-10 {
        let boundary = if dx > 0.0 { (ix + 1) as f32 } else { ix as f32 };
        (boundary - ox) / dx
    } else {
        f32::MAX
    };
    let t_max_y = if dy.abs() > 1e-10 {
        let boundary = if dy > 0.0 { (iy + 1) as f32 } else { iy as f32 };
        (boundary - oy) / dy
    } else {
        f32::MAX
    };
    let t_max_z = if dz.abs() > 1e-10 {
        let boundary = if dz > 0.0 { (iz + 1) as f32 } else { iz as f32 };
        (boundary - oz) / dz
    } else {
        f32::MAX
    };

    let mut t_max = [t_max_x, t_max_y, t_max_z];
    let mut face_normal = [0i32; 3];
    // Track the t-value of the last boundary crossing (entry distance of current voxel).
    let mut last_t: f32 = 0.0;

    // Step through voxels
    let max_steps = (max_dist * 3.0) as usize + 1;
    for _ in 0..max_steps {
        // Check current voxel
        let block = world.get_block(ix, iy, iz);
        if block != BlockType::Air && block.is_obstacle() {
            return Some(VoxelHit {
                block_pos: [ix, iy, iz],
                face_normal,
                distance: last_t,
            });
        }

        // Advance to next voxel boundary
        if t_max[0] < t_max[1] {
            if t_max[0] < t_max[2] {
                if t_max[0] > max_dist { break; }
                last_t = t_max[0];
                ix += step_x;
                t_max[0] += t_delta_x;
                face_normal = [-step_x, 0, 0];
            } else {
                if t_max[2] > max_dist { break; }
                last_t = t_max[2];
                iz += step_z;
                t_max[2] += t_delta_z;
                face_normal = [0, 0, -step_z];
            }
        } else if t_max[1] < t_max[2] {
            if t_max[1] > max_dist { break; }
            last_t = t_max[1];
            iy += step_y;
            t_max[1] += t_delta_y;
            face_normal = [0, -step_y, 0];
        } else {
            if t_max[2] > max_dist { break; }
            last_t = t_max[2];
            iz += step_z;
            t_max[2] += t_delta_z;
            face_normal = [0, 0, -step_z];
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_gen::WorldGenerator;

    #[test]
    fn raycast_hits_ground() {
        let gen = WorldGenerator::new(42);
        let mut world = VoxelWorld::default();
        // Generate a small area
        for cx in -1..=1 {
            for cz in -1..=1 {
                world.chunks.insert((cx, cz), gen.generate_chunk(cx, cz));
            }
        }
        // Cast straight down from high up
        let hit = raycast_voxels(&world, [16.0, 100.0, 16.0], [0.0, -1.0, 0.0], 200.0);
        assert!(hit.is_some(), "Should hit terrain when casting down");
        let h = hit.unwrap();
        assert_eq!(h.face_normal, [0, 1, 0], "Should hit top face");
        assert!(h.block_pos[1] < 100, "Hit should be below origin");
    }
}
