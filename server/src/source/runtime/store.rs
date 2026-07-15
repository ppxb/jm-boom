use std::collections::HashMap;

use super::host::{
    canvas::{Canvas, ImageData},
    html::{HtmlDocument, HtmlElement, HtmlElementList},
    net::NetRequest,
};

pub type Descriptor = i32;

pub enum DescriptorValue {
    Encoded(Vec<u8>),
    Request(Box<NetRequest>),
    HtmlDocument(HtmlDocument),
    HtmlElement(HtmlElement),
    HtmlElementList(HtmlElementList),
    Image(ImageData),
    Canvas(Canvas),
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

    pub fn get_mut(&mut self, descriptor: Descriptor) -> Option<&mut DescriptorValue> {
        self.values.get_mut(&descriptor)
    }

    pub fn remove(&mut self, descriptor: Descriptor) {
        self.values.remove(&descriptor);
        if self.values.is_empty() {
            self.next = 1;
        }
    }
}

impl DescriptorValue {
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Encoded(bytes) => Some(bytes),
            _ => None,
        }
    }

    pub fn as_request(&self) -> Option<&NetRequest> {
        match self {
            Self::Request(request) => Some(request),
            _ => None,
        }
    }

    pub fn as_request_mut(&mut self) -> Option<&mut NetRequest> {
        match self {
            Self::Request(request) => Some(request),
            _ => None,
        }
    }

    pub fn as_image(&self) -> Option<&ImageData> {
        match self {
            Self::Image(image) => Some(image),
            _ => None,
        }
    }

    pub fn as_canvas(&self) -> Option<&Canvas> {
        match self {
            Self::Canvas(canvas) => Some(canvas),
            _ => None,
        }
    }

    pub fn as_canvas_mut(&mut self) -> Option<&mut Canvas> {
        match self {
            Self::Canvas(canvas) => Some(canvas),
            _ => None,
        }
    }
}
