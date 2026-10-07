//! Typed event bus resource for decoupled cross-system communication.

use crate::ecs::Resource;
use std::any::{Any, TypeId};
use std::collections::HashMap;

trait AnyEventStorage: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clear(&mut self);
    fn len(&self) -> usize;
}

struct EventStorage<T> {
    events: Vec<T>,
}

impl<T: 'static + Send + Sync> Default for EventStorage<T> {
    fn default() -> Self {
        Self { events: Vec::new() }
    }
}

impl<T: 'static + Send + Sync> AnyEventStorage for EventStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clear(&mut self) {
        self.events.clear();
    }

    fn len(&self) -> usize {
        self.events.len()
    }
}

/// A typed event bus resource allowing systems to send and read strongly typed events.
#[derive(Resource, Default)]
pub struct EventBus {
    storages: HashMap<TypeId, Box<dyn AnyEventStorage>>,
}

impl EventBus {
    /// Creates a new empty `EventBus`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Emits a single event to the bus.
    pub fn send<T: 'static + Send + Sync + Clone>(&mut self, event: T) {
        let type_id = TypeId::of::<T>();
        let storage = self
            .storages
            .entry(type_id)
            .or_insert_with(|| Box::new(EventStorage::<T>::default()));

        if let Some(typed) = storage.as_any_mut().downcast_mut::<EventStorage<T>>() {
            typed.events.push(event);
        }
    }

    /// Emits multiple events to the bus in batch.
    pub fn send_batch<T: 'static + Send + Sync + Clone>(
        &mut self,
        events: impl IntoIterator<Item = T>,
    ) {
        let type_id = TypeId::of::<T>();
        let storage = self
            .storages
            .entry(type_id)
            .or_insert_with(|| Box::new(EventStorage::<T>::default()));

        if let Some(typed) = storage.as_any_mut().downcast_mut::<EventStorage<T>>() {
            typed.events.extend(events);
        }
    }

    /// Reads all currently published events of type `T`.
    pub fn read<T: 'static + Send + Sync + Clone>(&self) -> Vec<T> {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.storages.get(&type_id) {
            if let Some(typed) = storage.as_any().downcast_ref::<EventStorage<T>>() {
                return typed.events.clone();
            }
        }
        Vec::new()
    }

    /// Checks if any events of type `T` are currently available on the bus.
    pub fn has_events<T: 'static>(&self) -> bool {
        self.event_count::<T>() > 0
    }

    /// Returns the number of events of type `T` currently queued on the bus.
    pub fn event_count<T: 'static>(&self) -> usize {
        let type_id = TypeId::of::<T>();
        self.storages.get(&type_id).map(|s| s.len()).unwrap_or(0)
    }

    /// Clears all queued events of type `T`.
    pub fn clear<T: 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.storages.get_mut(&type_id) {
            storage.clear();
        }
    }

    /// Clears all events across all registered types on the bus.
    pub fn clear_all(&mut self) {
        for storage in self.storages.values_mut() {
            storage.clear();
        }
    }
}
