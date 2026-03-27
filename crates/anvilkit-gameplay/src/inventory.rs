//! Slot-based and stackable item inventory system.
//!
//! Provides [`ItemDef`] for item definitions, [`ItemStack`] for quantity tracking,
//! and two [`Inventory`] implementations:
//! - [`SlotInventory`] — fixed-size slot array (classic RPG inventory)
//! - [`StackInventory`] — auto-stacking, dynamically growing container

use bevy_ecs::prelude::*;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Static definition of an item type.
#[derive(Debug, Clone)]
pub struct ItemDef {
    /// Unique item identifier.
    pub id: u32,
    /// Human-readable name.
    pub name: String,
    /// Maximum units that can occupy a single stack.
    pub max_stack: u32,
    /// Per-unit weight.
    pub weight: f32,
}

/// A stack of identical items, identified by `item_id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    /// The item type this stack holds.
    pub item_id: u32,
    /// Number of items in this stack (always >= 1 after construction).
    pub quantity: u32,
}

impl ItemStack {
    /// Create a new stack. Panics if `quantity` is zero.
    pub fn new(item_id: u32, quantity: u32) -> Self {
        assert!(quantity > 0, "ItemStack quantity must be > 0");
        Self { item_id, quantity }
    }

    /// Returns `true` if `other` holds the same item type and could therefore
    /// be merged into this stack (ignoring capacity).
    pub fn can_merge(&self, other: &ItemStack) -> bool {
        self.item_id == other.item_id
    }

    /// Merge `other` into `self` up to `max_stack`.
    ///
    /// Returns `Some(remainder)` if not all units fit, or `None` if the merge
    /// consumed `other` entirely.
    pub fn merge(&mut self, other: ItemStack, max_stack: u32) -> Option<ItemStack> {
        if !self.can_merge(&other) {
            return Some(other);
        }

        let available = max_stack.saturating_sub(self.quantity);
        if available == 0 {
            return Some(other);
        }

        let transfer = available.min(other.quantity);
        self.quantity += transfer;

        let leftover = other.quantity - transfer;
        if leftover > 0 {
            Some(ItemStack::new(other.item_id, leftover))
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Inventory trait
// ---------------------------------------------------------------------------

/// Common interface for containers that hold [`ItemStack`]s.
pub trait Inventory {
    /// Total number of slots this inventory can hold.
    fn capacity(&self) -> usize;

    /// Read the stack at `index`, if any.
    fn get_slot(&self, index: usize) -> Option<&ItemStack>;

    /// Overwrite the stack at `index`. Pass `None` to clear.
    fn set_slot(&mut self, index: usize, stack: Option<ItemStack>);

    /// Try to insert `stack` into the inventory, stacking where possible.
    ///
    /// Returns `Some(remainder)` if not everything fit, or `None` on full success.
    fn add_item(&mut self, stack: ItemStack, max_stack: u32) -> Option<ItemStack>;

    /// Remove up to `quantity` units of `item_id`.
    ///
    /// Returns the number of units actually removed.
    fn remove_item(&mut self, item_id: u32, quantity: u32) -> u32;
}

// ---------------------------------------------------------------------------
// SlotInventory — fixed-size
// ---------------------------------------------------------------------------

/// Fixed-size inventory backed by a `Vec<Option<ItemStack>>`.
#[derive(Debug, Clone, Component)]
pub struct SlotInventory {
    slots: Vec<Option<ItemStack>>,
}

impl SlotInventory {
    /// Create an empty inventory with `size` slots.
    pub fn new(size: usize) -> Self {
        Self {
            slots: vec![None; size],
        }
    }
}

impl Inventory for SlotInventory {
    fn capacity(&self) -> usize {
        self.slots.len()
    }

    fn get_slot(&self, index: usize) -> Option<&ItemStack> {
        self.slots.get(index).and_then(|s| s.as_ref())
    }

    fn set_slot(&mut self, index: usize, stack: Option<ItemStack>) {
        if index < self.slots.len() {
            self.slots[index] = stack;
        }
    }

    fn add_item(&mut self, mut stack: ItemStack, max_stack: u32) -> Option<ItemStack> {
        // Phase 1: try to merge into existing stacks of the same item.
        for slot in self.slots.iter_mut() {
            if let Some(existing) = slot {
                if existing.can_merge(&stack) {
                    match existing.merge(stack, max_stack) {
                        Some(remainder) => stack = remainder,
                        None => return None,
                    }
                }
            }
        }

        // Phase 2: place remainder into the first empty slot.
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                if stack.quantity <= max_stack {
                    *slot = Some(stack);
                    return None;
                } else {
                    // Fill this slot to max and keep going.
                    *slot = Some(ItemStack::new(stack.item_id, max_stack));
                    stack.quantity -= max_stack;
                }
            }
        }

        // Could not fit everything.
        Some(stack)
    }

    fn remove_item(&mut self, item_id: u32, quantity: u32) -> u32 {
        let mut remaining = quantity;

        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if let Some(existing) = slot {
                if existing.item_id == item_id {
                    let take = remaining.min(existing.quantity);
                    existing.quantity -= take;
                    remaining -= take;
                    if existing.quantity == 0 {
                        *slot = None;
                    }
                }
            }
        }

        quantity - remaining
    }
}

// ---------------------------------------------------------------------------
// StackInventory — auto-stacking, dynamic growth
// ---------------------------------------------------------------------------

/// Dynamically growing inventory that automatically stacks items.
#[derive(Debug, Clone, Component, Default)]
pub struct StackInventory {
    stacks: Vec<ItemStack>,
}

impl StackInventory {
    /// Create a new, empty stack inventory.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Inventory for StackInventory {
    fn capacity(&self) -> usize {
        self.stacks.len()
    }

    fn get_slot(&self, index: usize) -> Option<&ItemStack> {
        self.stacks.get(index)
    }

    fn set_slot(&mut self, index: usize, stack: Option<ItemStack>) {
        match stack {
            Some(s) => {
                if index < self.stacks.len() {
                    self.stacks[index] = s;
                } else if index == self.stacks.len() {
                    self.stacks.push(s);
                }
            }
            None => {
                if index < self.stacks.len() {
                    self.stacks.remove(index);
                }
            }
        }
    }

    fn add_item(&mut self, mut stack: ItemStack, max_stack: u32) -> Option<ItemStack> {
        // Phase 1: merge into existing stacks.
        for existing in self.stacks.iter_mut() {
            if existing.can_merge(&stack) {
                match existing.merge(stack, max_stack) {
                    Some(remainder) => stack = remainder,
                    None => return None,
                }
            }
        }

        // Phase 2: create new stacks as needed (no size limit).
        while stack.quantity > max_stack {
            self.stacks.push(ItemStack::new(stack.item_id, max_stack));
            stack.quantity -= max_stack;
        }
        self.stacks.push(stack);
        None
    }

    fn remove_item(&mut self, item_id: u32, quantity: u32) -> u32 {
        let mut remaining = quantity;

        self.stacks.retain_mut(|s| {
            if remaining == 0 || s.item_id != item_id {
                return true;
            }
            let take = remaining.min(s.quantity);
            s.quantity -= take;
            remaining -= take;
            s.quantity > 0
        });

        quantity - remaining
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- ItemStack tests ----------------------------------------------------

    #[test]
    fn item_stack_new() {
        let stack = ItemStack::new(1, 10);
        assert_eq!(stack.item_id, 1);
        assert_eq!(stack.quantity, 10);
    }

    #[test]
    #[should_panic(expected = "quantity must be > 0")]
    fn item_stack_new_zero_panics() {
        ItemStack::new(1, 0);
    }

    #[test]
    fn merge_stacks_full_fit() {
        let mut a = ItemStack::new(1, 5);
        let b = ItemStack::new(1, 3);
        let remainder = a.merge(b, 10);
        assert!(remainder.is_none());
        assert_eq!(a.quantity, 8);
    }

    #[test]
    fn merge_stacks_overflow() {
        let mut a = ItemStack::new(1, 8);
        let b = ItemStack::new(1, 5);
        let remainder = a.merge(b, 10);
        assert_eq!(a.quantity, 10);
        let r = remainder.expect("should have remainder");
        assert_eq!(r.item_id, 1);
        assert_eq!(r.quantity, 3);
    }

    #[test]
    fn merge_different_items_returns_other() {
        let mut a = ItemStack::new(1, 5);
        let b = ItemStack::new(2, 3);
        let remainder = a.merge(b, 10);
        assert_eq!(a.quantity, 5); // unchanged
        let r = remainder.expect("different item should be returned");
        assert_eq!(r.item_id, 2);
        assert_eq!(r.quantity, 3);
    }

    // -- SlotInventory tests ------------------------------------------------

    #[test]
    fn slot_inventory_new_is_empty() {
        let inv = SlotInventory::new(5);
        assert_eq!(inv.capacity(), 5);
        for i in 0..5 {
            assert!(inv.get_slot(i).is_none());
        }
    }

    #[test]
    fn slot_inventory_add_single() {
        let mut inv = SlotInventory::new(3);
        let remainder = inv.add_item(ItemStack::new(1, 5), 10);
        assert!(remainder.is_none());
        let slot = inv.get_slot(0).expect("slot 0 should be occupied");
        assert_eq!(slot.item_id, 1);
        assert_eq!(slot.quantity, 5);
    }

    #[test]
    fn slot_inventory_add_stack_overflow() {
        let mut inv = SlotInventory::new(1);
        // Only one slot with max_stack=5, try to add 8.
        let remainder = inv.add_item(ItemStack::new(1, 8), 5);
        let slot = inv.get_slot(0).expect("slot 0 should be filled");
        assert_eq!(slot.quantity, 5);
        let r = remainder.expect("should have remainder");
        assert_eq!(r.quantity, 3);
    }

    #[test]
    fn slot_inventory_remove_partial() {
        let mut inv = SlotInventory::new(3);
        inv.add_item(ItemStack::new(1, 10), 20);
        let removed = inv.remove_item(1, 4);
        assert_eq!(removed, 4);
        let slot = inv.get_slot(0).expect("slot still has items");
        assert_eq!(slot.quantity, 6);
    }

    #[test]
    fn slot_inventory_remove_full() {
        let mut inv = SlotInventory::new(3);
        inv.add_item(ItemStack::new(1, 5), 10);
        let removed = inv.remove_item(1, 5);
        assert_eq!(removed, 5);
        assert!(inv.get_slot(0).is_none(), "slot should be cleared");
    }

    #[test]
    fn slot_inventory_get_set() {
        let mut inv = SlotInventory::new(3);
        assert!(inv.get_slot(1).is_none());
        inv.set_slot(1, Some(ItemStack::new(42, 7)));
        let slot = inv.get_slot(1).expect("slot 1 should be set");
        assert_eq!(slot.item_id, 42);
        assert_eq!(slot.quantity, 7);

        inv.set_slot(1, None);
        assert!(inv.get_slot(1).is_none(), "slot 1 should be cleared");
    }

    // -- StackInventory tests -----------------------------------------------

    #[test]
    fn stack_inventory_auto_stacks() {
        let mut inv = StackInventory::new();
        inv.add_item(ItemStack::new(1, 5), 10);
        inv.add_item(ItemStack::new(1, 3), 10);
        // Should merge into a single stack of 8.
        assert_eq!(inv.capacity(), 1);
        assert_eq!(inv.get_slot(0).unwrap().quantity, 8);
    }

    #[test]
    fn stack_inventory_grows_dynamically() {
        let mut inv = StackInventory::new();
        inv.add_item(ItemStack::new(1, 5), 10);
        inv.add_item(ItemStack::new(2, 3), 10);
        assert_eq!(inv.capacity(), 2);
    }

    #[test]
    fn stack_inventory_remove() {
        let mut inv = StackInventory::new();
        inv.add_item(ItemStack::new(1, 10), 20);
        let removed = inv.remove_item(1, 7);
        assert_eq!(removed, 7);
        assert_eq!(inv.get_slot(0).unwrap().quantity, 3);
    }
}
