//! # Navigation / AI
//!
//! NavMesh 寻路系统：支持 A* 路径规划和 NavAgent 自动导航。

use bevy_ecs::prelude::*;
use glam::{Vec3, Vec2};
use std::collections::BinaryHeap;
use std::cmp::Ordering;

use crate::physics::DeltaTime;
use crate::schedule::AnvilKitSchedule;
use anvilkit_core::math::Transform;

// ---------------------------------------------------------------------------
//  NavMesh
// ---------------------------------------------------------------------------

/// Navigation mesh resource.
#[derive(Resource, Clone)]
pub struct NavMesh {
    /// World-space vertex positions.
    pub vertices: Vec<Vec3>,
    /// Triangle indices (each element is [v0, v1, v2]).
    pub triangles: Vec<[u32; 3]>,
    /// Per-triangle adjacency: `adjacency[i]` lists triangle indices sharing an edge with triangle `i`.
    pub adjacency: Vec<Vec<usize>>,
}

impl NavMesh {
    /// Build a NavMesh from vertices and triangles, auto-computing adjacency.
    pub fn new(vertices: Vec<Vec3>, triangles: Vec<[u32; 3]>) -> Self {
        let adjacency = Self::build_adjacency(&triangles);
        Self { vertices, triangles, adjacency }
    }

    /// Build adjacency list: two triangles are adjacent if they share exactly 2 vertices (an edge).
    fn build_adjacency(triangles: &[[u32; 3]]) -> Vec<Vec<usize>> {
        let n = triangles.len();
        let mut adj = vec![Vec::new(); n];
        for i in 0..n {
            for j in (i + 1)..n {
                let shared = Self::shared_vertex_count(&triangles[i], &triangles[j]);
                if shared >= 2 {
                    adj[i].push(j);
                    adj[j].push(i);
                }
            }
        }
        adj
    }

    fn shared_vertex_count(a: &[u32; 3], b: &[u32; 3]) -> usize {
        let mut count = 0;
        for &va in a {
            for &vb in b {
                if va == vb {
                    count += 1;
                }
            }
        }
        count
    }

    /// Compute the centroid of triangle `idx`.
    pub fn triangle_centroid(&self, idx: usize) -> Vec3 {
        let tri = &self.triangles[idx];
        let a = self.vertices[tri[0] as usize];
        let b = self.vertices[tri[1] as usize];
        let c = self.vertices[tri[2] as usize];
        (a + b + c) / 3.0
    }

    /// Find which triangle contains a point (projected onto XZ plane).
    /// Returns the triangle index, or None if the point is outside the NavMesh.
    pub fn find_triangle(&self, point: Vec3) -> Option<usize> {
        let p = Vec2::new(point.x, point.z);
        for (i, tri) in self.triangles.iter().enumerate() {
            let a = Vec2::new(self.vertices[tri[0] as usize].x, self.vertices[tri[0] as usize].z);
            let b = Vec2::new(self.vertices[tri[1] as usize].x, self.vertices[tri[1] as usize].z);
            let c = Vec2::new(self.vertices[tri[2] as usize].x, self.vertices[tri[2] as usize].z);
            if Self::point_in_triangle(p, a, b, c) {
                return Some(i);
            }
        }
        None
    }

    /// Barycentric point-in-triangle test (2D, XZ plane).
    pub fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
        let v0 = c - a;
        let v1 = b - a;
        let v2 = p - a;

        let dot00 = v0.dot(v0);
        let dot01 = v0.dot(v1);
        let dot02 = v0.dot(v2);
        let dot11 = v1.dot(v1);
        let dot12 = v1.dot(v2);

        let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

        u >= 0.0 && v >= 0.0 && (u + v) <= 1.0
    }

    /// A* path planning on the triangle adjacency graph.
    ///
    /// Returns a list of waypoints (start → triangle centroids → goal), or None if no path exists.
    pub fn find_path(&self, start: Vec3, goal: Vec3) -> Option<Vec<Vec3>> {
        let start_tri = self.find_triangle(start)?;
        let goal_tri = self.find_triangle(goal)?;

        if start_tri == goal_tri {
            return Some(vec![start, goal]);
        }

        let n = self.triangles.len();
        let mut g_score = vec![f32::INFINITY; n];
        let mut came_from = vec![usize::MAX; n];
        let mut closed = vec![false; n];

        g_score[start_tri] = 0.0;

        let goal_centroid = self.triangle_centroid(goal_tri);

        let mut open = BinaryHeap::new();
        open.push(AStarNode {
            cost: self.triangle_centroid(start_tri).distance(goal_centroid),
            triangle_idx: start_tri,
        });

        while let Some(current) = open.pop() {
            if current.triangle_idx == goal_tri {
                // Reconstruct path
                let mut path = vec![goal];
                let mut idx = goal_tri;
                while idx != start_tri {
                    path.push(self.triangle_centroid(idx));
                    idx = came_from[idx];
                }
                path.push(start);
                path.reverse();
                return Some(path);
            }

            if closed[current.triangle_idx] {
                continue;
            }
            closed[current.triangle_idx] = true;

            let current_g = g_score[current.triangle_idx];
            let current_centroid = self.triangle_centroid(current.triangle_idx);

            for &neighbor in &self.adjacency[current.triangle_idx] {
                if closed[neighbor] {
                    continue;
                }

                let neighbor_centroid = self.triangle_centroid(neighbor);
                let tentative_g = current_g + current_centroid.distance(neighbor_centroid);

                if tentative_g < g_score[neighbor] {
                    g_score[neighbor] = tentative_g;
                    came_from[neighbor] = current.triangle_idx;
                    let h = neighbor_centroid.distance(goal_centroid);
                    open.push(AStarNode {
                        cost: tentative_g + h,
                        triangle_idx: neighbor,
                    });
                }
            }
        }

        None // No path found
    }
}

/// A* open set node (min-heap by cost).
#[derive(Debug, Clone)]
struct AStarNode {
    cost: f32,
    triangle_idx: usize,
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.triangle_idx == other.triangle_idx
    }
}
impl Eq for AStarNode {}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (BinaryHeap is max-heap)
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

// ---------------------------------------------------------------------------
//  NavAgent
// ---------------------------------------------------------------------------

/// Navigation agent component. Moves an entity along a computed path.
#[derive(Debug, Clone, Component)]
pub struct NavAgent {
    /// Movement speed (units per second).
    pub speed: f32,
    /// Current waypoint path.
    pub path: Vec<Vec3>,
    /// Index of the current target waypoint.
    pub current_waypoint: usize,
    /// Distance threshold to consider a waypoint reached.
    pub arrival_threshold: f32,
    /// Whether the agent is active.
    pub enabled: bool,
}

impl NavAgent {
    /// Create a new NavAgent with the given speed.
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            path: Vec::new(),
            current_waypoint: 0,
            arrival_threshold: 0.3,
            enabled: true,
        }
    }

    /// Compute a path from `from` to `to` using the given NavMesh.
    /// Returns true if a path was found.
    pub fn set_destination(&mut self, navmesh: &NavMesh, from: Vec3, to: Vec3) -> bool {
        if let Some(path) = navmesh.find_path(from, to) {
            self.path = path;
            self.current_waypoint = 0;
            true
        } else {
            self.path.clear();
            self.current_waypoint = 0;
            false
        }
    }

    /// Returns true if the agent has reached the end of its path.
    pub fn has_arrived(&self) -> bool {
        self.path.is_empty() || self.current_waypoint >= self.path.len()
    }
}

// ---------------------------------------------------------------------------
//  Systems
// ---------------------------------------------------------------------------

/// Steering system: moves NavAgent entities along their paths.
pub fn nav_agent_steering_system(
    dt: Res<DeltaTime>,
    mut query: Query<(&mut Transform, &mut NavAgent)>,
) {
    for (mut transform, mut agent) in query.iter_mut() {
        if !agent.enabled || agent.has_arrived() {
            continue;
        }

        let target = agent.path[agent.current_waypoint];
        let to_target = target - transform.translation;
        let distance = to_target.length();

        if distance < agent.arrival_threshold {
            agent.current_waypoint += 1;
            continue;
        }

        let direction = to_target / distance;
        let step = agent.speed * dt.0;
        if step >= distance {
            transform.translation = target;
            agent.current_waypoint += 1;
        } else {
            transform.translation += direction * step;
        }
    }
}

// ---------------------------------------------------------------------------
//  Plugin
// ---------------------------------------------------------------------------

/// Navigation plugin. Registers the NavAgent steering system.
pub struct NavigationPlugin;

impl crate::plugin::Plugin for NavigationPlugin {
    fn build(&self, app: &mut crate::app::App) {
        app.add_systems(AnvilKitSchedule::Update, nav_agent_steering_system);
    }

    fn name(&self) -> &str {
        "NavigationPlugin"
    }
}

// ---------------------------------------------------------------------------
//  Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quad_navmesh() -> NavMesh {
        // A 2x2 quad split into 4 triangles:
        //   v0(0,0) --- v1(1,0) --- v2(2,0)
        //     |   \    /   |   \    /
        //   v3(0,1) --- v4(1,1) --- v5(2,1)
        let vertices = vec![
            Vec3::new(0.0, 0.0, 0.0), // 0
            Vec3::new(1.0, 0.0, 0.0), // 1
            Vec3::new(2.0, 0.0, 0.0), // 2
            Vec3::new(0.0, 0.0, 1.0), // 3
            Vec3::new(1.0, 0.0, 1.0), // 4
            Vec3::new(2.0, 0.0, 1.0), // 5
        ];
        let triangles = vec![
            [0, 1, 4], // tri 0 (left-top)
            [0, 4, 3], // tri 1 (left-bottom)
            [1, 2, 5], // tri 2 (right-top)
            [1, 5, 4], // tri 3 (right-bottom)
        ];
        NavMesh::new(vertices, triangles)
    }

    #[test]
    fn test_point_in_triangle() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(1.0, 0.0);
        let c = Vec2::new(0.0, 1.0);

        assert!(NavMesh::point_in_triangle(Vec2::new(0.2, 0.2), a, b, c));
        assert!(!NavMesh::point_in_triangle(Vec2::new(1.0, 1.0), a, b, c));
        assert!(NavMesh::point_in_triangle(Vec2::new(0.0, 0.0), a, b, c));
    }

    #[test]
    fn test_navmesh_find_triangle() {
        let mesh = make_quad_navmesh();
        // Point in tri 0 (upper-left)
        let idx = mesh.find_triangle(Vec3::new(0.4, 0.0, 0.2));
        assert!(idx.is_some());
        assert_eq!(idx.unwrap(), 0);

        // Point outside mesh
        let idx = mesh.find_triangle(Vec3::new(-1.0, 0.0, -1.0));
        assert!(idx.is_none());
    }

    #[test]
    fn test_navmesh_adjacency() {
        let mesh = make_quad_navmesh();
        // tri 0 (0,1,4) shares edge (0,4) with tri 1 and edge (1,4) with tri 3
        assert!(mesh.adjacency[0].contains(&1));
        assert!(mesh.adjacency[0].contains(&3));
        // tri 2 (1,2,5) shares edge (1,5) with tri 3
        assert!(mesh.adjacency[2].contains(&3));
    }

    #[test]
    fn test_astar_path() {
        let mesh = make_quad_navmesh();
        let start = Vec3::new(0.2, 0.0, 0.2);
        let goal = Vec3::new(1.8, 0.0, 0.8);

        let path = mesh.find_path(start, goal);
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.len() >= 2);
        assert_eq!(path.first().unwrap(), &start);
        assert_eq!(path.last().unwrap(), &goal);
    }

    #[test]
    fn test_nav_agent_steering() {
        use crate::prelude::*;

        let mut app = App::new();
        app.init_resource::<DeltaTime>();
        app.add_systems(AnvilKitSchedule::Update, nav_agent_steering_system);

        let mut agent = NavAgent::new(10.0);
        agent.path = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 5.0),
        ];
        agent.current_waypoint = 1; // skip start, target = (5,0,0)

        let entity = app.world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            agent,
        )).id();

        // Run several frames
        for _ in 0..60 {
            app.update();
        }

        let t = app.world.get::<Transform>(entity).unwrap();
        // After ~1 second at speed 10, should have moved significantly
        assert!(t.translation.x > 1.0, "Agent should have moved towards waypoint");
    }
}
