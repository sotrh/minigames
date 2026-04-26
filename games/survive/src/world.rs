use framework::glam;

#[derive(Debug, Clone, Copy)]
pub struct EntityId(u32, u32);

pub struct Entry {
    index: Option<u32>,
    generation: u32,
}

pub struct DenseStorage<T> {
    directory: Vec<Entry>,
    data: Vec<T>,
    free: Vec<usize>,
}

#[allow(unused)]
impl<T> DenseStorage<T> {
    pub fn new() -> Self {
        Self {
            directory: Vec::new(),
            data: Vec::new(),
            free: Vec::new(),
            last: None,
        }
    }

    pub fn with_capacity(capacity: u32) -> Self {
        Self {
            directory: Vec::with_capacity(capacity as _),
            data: Vec::with_capacity(capacity as _),
            free: Vec::new(),
            last: None,
        }
    }

    pub fn insert(&mut self, item: T) -> EntityId {
        if let Some(free) = self.free.pop() {
            let data_index = self.data.len();

            let entry = &mut self.directory[free];
            entry.generation += 1;
            entry.index = Some(data_index as _);

            self.data.push(item);

            let id = EntityId(free as _, entry.generation);
            id
        } else {
            let id = EntityId(self.directory.len() as _, 0);

            let entry = Entry {
                index: Some(self.data.len() as _),
                generation: id.1,
            };

            self.data.push(item);
            self.directory.push(entry);

            id
        }
    }

    pub fn remove(&mut self, id: EntityId) -> Option<T> {
        let dir_i = id.0 as usize;

        if dir_i < self.directory.len() {
            let entry = &self.directory[dir_i];

            if id.1 == entry.generation && let Some(data_i) = entry.index {
                let item = Some(self.data.swap_remove(data_i as _));

                todo!("Fix the directory")
            }
        }

        None
    }

    pub fn get(&self, id: EntityId) -> Option<&T> {
        let dir_index = id.0 as usize;
        if dir_index < self.directory.len() {
            let entry = &self.directory[dir_index];

            if entry.generation == id.1
                && let Some(data_index) = entry.index
            {
                return self.data.get(data_index as usize);
            }
        }

        None
    }
}

impl<T> Default for DenseStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct World {
    positions: DenseStorage<glam::Vec3>,
}

impl World {
    pub fn new() -> Self {
        Self {
            positions: Default::default(),
        }
    }

    pub(crate) fn clear(&mut self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::world::DenseStorage;

    #[test]
    fn test_dense_insert() {
        let mut s = DenseStorage::new();
        let a = s.insert("a");
        assert_eq!(s.get(a), Some(&"a"));
    }

    #[test]
    fn test_dense_remove() {
        let mut s = DenseStorage::new();

        let a = s.insert("a");

        assert_eq!(s.get(a), Some(&"a"));
        assert_eq!(s.remove(a), Some("a"));
        assert_eq!(s.get(a), None);
        assert_eq!(s.remove(a), None);

        let b = s.insert("b");

        assert_eq!(b.0, a.0);
        assert_ne!(b.1, a.1);

        assert_eq!(s.get(a), None);
        assert_eq!(s.get(b), Some(&"b"));
    }
}
