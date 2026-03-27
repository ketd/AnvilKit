//! Object pool for entity recycling.
//!
//! [`EntityPool`] is a resource that keeps a list of "available" entities that
//! can be acquired and released, avoiding the cost of repeated spawn/despawn
//! cycles.  Because entity creation requires a [`World`], the pool does **not**
//! pre-allocate real entities — it stores recycled [`Entity`] handles and grows
//! dynamically as entities are released back into the pool.

use bevy_ecs::prelude::*;

/// A pool of recyclable [`Entity`] handles.
///
/// # Usage
///
/// 1. Insert as a resource: `world.insert_resource(EntityPool::new(64));`
/// 2. [`acquire`](EntityPool::acquire) — take an entity from the pool (or
///    `None` if empty, in which case the caller should spawn a new one).
/// 3. [`release`](EntityPool::release) — return an entity to the pool for
///    later reuse.
#[derive(Resource, Debug, Clone)]
pub struct EntityPool {
    /// Entities available for reuse.
    pub available: Vec<Entity>,
    /// The maximum number of entities the pool will hold.
    pub capacity: usize,
}

impl EntityPool {
    /// Create a new, empty pool with the given capacity.
    ///
    /// No entities are pre-allocated — the pool fills as entities are
    /// [`release`](Self::release)d.
    pub fn new(capacity: usize) -> Self {
        Self {
            available: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Take an entity from the pool.
    ///
    /// Returns `None` if the pool is empty; the caller should spawn a new
    /// entity in that case.
    pub fn acquire(&mut self) -> Option<Entity> {
        self.available.pop()
    }

    /// Return an entity to the pool for later reuse.
    ///
    /// If the pool is already at capacity the entity is silently dropped
    /// (not added) to prevent unbounded growth.
    pub fn release(&mut self, entity: Entity) {
        if self.available.len() < self.capacity {
            self.available.push(entity);
        }
    }

    /// Returns the number of entities currently available in the pool.
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// Returns `true` when no entities are available.
    pub fn is_empty(&self) -> bool {
        self.available.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper — create a throwaway world and spawn `n` entities, returning
    /// their handles.
    fn spawn_entities(n: usize) -> (World, Vec<Entity>) {
        let mut world = World::new();
        let entities: Vec<Entity> = (0..n).map(|_| world.spawn_empty().id()).collect();
        (world, entities)
    }

    #[test]
    fn new_pool_is_empty() {
        let pool = EntityPool::new(10);
        assert!(pool.is_empty());
        assert_eq!(pool.available_count(), 0);
        assert_eq!(pool.capacity, 10);
    }

    #[test]
    fn acquire_returns_none_when_empty() {
        let mut pool = EntityPool::new(4);
        assert!(pool.acquire().is_none());
    }

    #[test]
    fn release_and_acquire_round_trip() {
        let (_world, entities) = spawn_entities(3);
        let mut pool = EntityPool::new(10);

        for &e in &entities {
            pool.release(e);
        }
        assert_eq!(pool.available_count(), 3);

        let acquired = pool.acquire().unwrap();
        // Vec::pop returns the last element, so we expect LIFO order.
        assert_eq!(acquired, entities[2]);
        assert_eq!(pool.available_count(), 2);
    }

    #[test]
    fn release_respects_capacity() {
        let (_world, entities) = spawn_entities(5);
        let mut pool = EntityPool::new(3);

        for &e in &entities {
            pool.release(e);
        }
        // Only 3 should have been accepted.
        assert_eq!(pool.available_count(), 3);
    }

    #[test]
    fn acquire_drains_pool() {
        let (_world, entities) = spawn_entities(2);
        let mut pool = EntityPool::new(10);
        pool.release(entities[0]);
        pool.release(entities[1]);

        assert!(pool.acquire().is_some());
        assert!(pool.acquire().is_some());
        assert!(pool.acquire().is_none());
        assert!(pool.is_empty());
    }
}
