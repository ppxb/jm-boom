use std::collections::HashMap;

pub type Descriptor = i32;

pub enum DescriptorValue {
    Encoded(Vec<u8>),
}

#[derive(Default)]
pub struct DescriptorStore {
    next: Descriptor,
    values: HashMap<Descriptor, DescriptorValue>,
}

impl DescriptorStore {
    pub fn new() -> Self {
        Self {
            next: 1,
            values: HashMap::new(),
        }
    }

    pub fn insert(&mut self, value: DescriptorValue) -> Descriptor {
        let descriptor = self.next;
        self.next = self.next.saturating_add(1).max(1);
        self.values.insert(descriptor, value);
        descriptor
    }

    pub fn get(&self, descriptor: Descriptor) -> Option<&DescriptorValue> {
        self.values.get(&descriptor)
    }

    pub fn remove(&mut self, descriptor: Descriptor) {
        self.values.remove(&descriptor);
        if self.values.is_empty() {
            self.next = 1;
        }
    }
}

impl DescriptorValue {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Encoded(bytes) => bytes,
        }
    }
}
