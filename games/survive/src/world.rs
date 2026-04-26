use framework::glam;

#[derive(Debug, Clone, Copy)]
pub struct EntityId(u32, u32);

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

#[derive(Debug)]
pub struct Entry {
    index: Option<usize>,
    generation: u32,
}

#[derive(Debug)]
pub struct DenseStorage<T> {
    directory: Vec<Entry>,
    data: Vec<T>,
    data_to_directory: Vec<usize>,
    free: Vec<usize>,
}

#[allow(unused)]
impl<T> DenseStorage<T> {
    pub fn new() -> Self {
        Self {
            directory: Vec::new(),
            data: Vec::new(),
            free: Vec::new(),
            data_to_directory: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: u32) -> Self {
        Self {
            directory: Vec::with_capacity(capacity as _),
            data_to_directory: Vec::with_capacity(capacity as _),
            data: Vec::with_capacity(capacity as _),
            free: Vec::new(),
        }
    }

    pub fn insert(&mut self, item: T) -> EntityId {
        if let Some(free) = self.free.pop() {
            let entry = &mut self.directory[free];
            entry.generation += 1;

            let id = EntityId(free as u32, entry.generation);

            entry.index = Some(self.data.len());

            self.data.push(item);
            self.data_to_directory.push(free);

            return id;
        }

        let id = EntityId(self.directory.len() as _, 0);

        let entry = Entry {
            index: Some(self.data.len() as _),
            generation: id.1,
        };

        self.data.push(item);
        self.data_to_directory.push(self.directory.len());
        self.directory.push(entry);

        id
    }

    pub fn remove(&mut self, id: EntityId) -> Option<T> {
        let index = self.index_from_id(id)?;

        let item = self.data.swap_remove(index);
        self.data_to_directory.swap_remove(index);

        self.free.push(id.0 as _);

        self.directory[id.0 as usize].index = None;

        if !self.data_to_directory.is_empty() {
            let dir_i = self.data_to_directory[index];
            self.directory[dir_i].index = Some(index);
        }

        Some(item)
    }

    pub fn get(&self, id: EntityId) -> Option<&T> {
        self.data.get(self.index_from_id(id)?)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut T> {
        let index = self.index_from_id(id)?;
        self.data.get_mut(index)
    }

    fn index_from_id(&self, id: EntityId) -> Option<usize> {
        let i = id.0 as usize;

        if i < self.directory.len() {
            let entry = &self.directory[i];

            if entry.generation == id.1 {
                return entry.index;
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
