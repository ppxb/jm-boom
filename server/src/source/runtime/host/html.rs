use std::rc::Rc;

use ego_tree::NodeId;
use scraper::{ElementRef, Html, Selector};
use url::Url;
use wasmer::FunctionEnvMut;

use super::HostState;
use crate::source::runtime::store::DescriptorValue;

const INVALID_DESCRIPTOR: i32 = -1;
const INVALID_STRING: i32 = -2;
const INVALID_QUERY: i32 = -4;
const NO_RESULT: i32 = -5;

#[derive(Clone)]
pub(crate) struct HtmlDocument {
    pub(super) html: Rc<Html>,
    pub(super) base_url: Option<Rc<Url>>,
}

#[derive(Clone)]
pub(crate) struct HtmlElement {
    html: Rc<Html>,
    base_url: Option<Rc<Url>>,
    id: NodeId,
}

#[derive(Clone)]
pub(crate) struct HtmlElementList(pub(crate) Vec<HtmlElement>);

impl HtmlDocument {
    pub(super) fn parse(text: &str, base_url: Option<&str>) -> Self {
        Self {
            html: Rc::new(Html::parse_document(text)),
            base_url: base_url
                .and_then(|value| Url::parse(value).ok())
                .map(Rc::new),
        }
    }

    fn select(&self, selector: &Selector) -> HtmlElementList {
        HtmlElementList(
            self.html
                .select_with_root(selector)
                .map(|element| self.element(element.id()))
                .collect(),
        )
    }

    fn element(&self, id: NodeId) -> HtmlElement {
        HtmlElement {
            html: self.html.clone(),
            base_url: self.base_url.clone(),
            id,
        }
    }
}

impl HtmlElement {
    fn select(&self, selector: &Selector) -> HtmlElementList {
        let Some(node) = self.html.tree.get(self.id) else {
            return HtmlElementList(Vec::new());
        };
        let Some(element) = ElementRef::wrap(node) else {
            return HtmlElementList(Vec::new());
        };
        HtmlElementList(
            element
                .select_with_root(selector)
                .map(|selected| self.with_id(selected.id()))
                .collect(),
        )
    }

    fn attr(&self, name: &str) -> Option<String> {
        let absolute = name.strip_prefix("abs:");
        let name = absolute.unwrap_or(name);
        let node = self.html.tree.get(self.id)?;
        let element = ElementRef::wrap(node)?;
        let value = element.attr(name)?.to_string();
        if absolute.is_some() {
            if let Ok(url) = Url::parse(&value) {
                return Some(url.to_string());
            }
            return self
                .base_url
                .as_ref()
                .and_then(|base| base.join(&value).ok())
                .map(|url| url.to_string());
        }
        Some(value)
    }

    fn text(&self, trimmed: bool) -> Option<String> {
        let node = self.html.tree.get(self.id)?;
        let text = ElementRef::wrap(node)?.text().collect::<String>();
        Some(if trimmed {
            text.trim().to_string()
        } else {
            text
        })
    }

    fn html(&self) -> Option<String> {
        Some(ElementRef::wrap(self.html.tree.get(self.id)?)?.inner_html())
    }

    fn with_id(&self, id: NodeId) -> Self {
        Self {
            html: self.html.clone(),
            base_url: self.base_url.clone(),
            id,
        }
    }
}

impl HtmlElementList {
    fn select(&self, selector: &Selector) -> Self {
        Self(
            self.0
                .iter()
                .flat_map(|element| element.select(selector).0)
                .collect(),
        )
    }

    fn attr(&self, name: &str) -> Option<String> {
        self.0.iter().find_map(|element| element.attr(name))
    }

    fn text(&self) -> String {
        self.0
            .iter()
            .filter_map(|element| element.text(true))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn html(&self) -> String {
        self.0
            .iter()
            .filter_map(HtmlElement::html)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub(super) fn attr(
    mut env: FunctionEnvMut<HostState>,
    descriptor: i32,
    pointer: i32,
    length: i32,
) -> i32 {
    let Ok(name) = read_string(&env, pointer, length) else {
        return INVALID_STRING;
    };
    let Some(value) = env
        .data()
        .descriptors
        .get(descriptor)
        .and_then(|item| match item {
            DescriptorValue::HtmlElement(element) => element.attr(&name),
            DescriptorValue::HtmlElementList(elements) => elements.attr(&name),
            _ => None,
        })
    else {
        return INVALID_DESCRIPTOR;
    };
    store_string(&mut env, value)
}

pub(super) fn get(mut env: FunctionEnvMut<HostState>, descriptor: i32, index: i32) -> i32 {
    if index < 0 {
        return NO_RESULT;
    }
    let Some(value) = env.data().descriptors.get(descriptor) else {
        return INVALID_DESCRIPTOR;
    };
    let result = match value {
        DescriptorValue::HtmlElementList(elements) => elements.0.get(index as usize).cloned(),
        _ => None,
    };
    result
        .map(|element| {
            env.data_mut()
                .descriptors
                .insert(DescriptorValue::HtmlElement(element))
        })
        .unwrap_or(NO_RESULT)
}

pub(super) fn select(
    mut env: FunctionEnvMut<HostState>,
    descriptor: i32,
    pointer: i32,
    length: i32,
) -> i32 {
    let Ok(query) = read_string(&env, pointer, length) else {
        return INVALID_STRING;
    };
    let Ok(selector) = Selector::parse(&query) else {
        return INVALID_QUERY;
    };
    let Some(value) = env.data().descriptors.get(descriptor) else {
        return INVALID_DESCRIPTOR;
    };
    let elements = match value {
        DescriptorValue::HtmlDocument(document) => document.select(&selector),
        DescriptorValue::HtmlElement(element) => element.select(&selector),
        DescriptorValue::HtmlElementList(elements) => elements.select(&selector),
        _ => return INVALID_DESCRIPTOR,
    };
    env.data_mut()
        .descriptors
        .insert(DescriptorValue::HtmlElementList(elements))
}

pub(super) fn select_first(
    mut env: FunctionEnvMut<HostState>,
    descriptor: i32,
    pointer: i32,
    length: i32,
) -> i32 {
    let Ok(query) = read_string(&env, pointer, length) else {
        return INVALID_STRING;
    };
    let Ok(selector) = Selector::parse(&query) else {
        return INVALID_QUERY;
    };
    let Some(value) = env.data().descriptors.get(descriptor) else {
        return INVALID_DESCRIPTOR;
    };
    let element = match value {
        DescriptorValue::HtmlDocument(document) => document.select(&selector).0.into_iter().next(),
        DescriptorValue::HtmlElement(element) => element.select(&selector).0.into_iter().next(),
        DescriptorValue::HtmlElementList(elements) => {
            elements.select(&selector).0.into_iter().next()
        }
        _ => return INVALID_DESCRIPTOR,
    };
    element
        .map(|element| {
            env.data_mut()
                .descriptors
                .insert(DescriptorValue::HtmlElement(element))
        })
        .unwrap_or(NO_RESULT)
}

pub(super) fn size(env: FunctionEnvMut<HostState>, descriptor: i32) -> i32 {
    match env.data().descriptors.get(descriptor) {
        Some(DescriptorValue::HtmlElementList(elements)) => elements.0.len() as i32,
        _ => INVALID_DESCRIPTOR,
    }
}

pub(super) fn text(mut env: FunctionEnvMut<HostState>, descriptor: i32) -> i32 {
    let Some(value) = env.data().descriptors.get(descriptor) else {
        return INVALID_DESCRIPTOR;
    };
    let text = match value {
        DescriptorValue::HtmlElement(element) => element.text(true),
        DescriptorValue::HtmlElementList(elements) => Some(elements.text()),
        _ => None,
    };
    text.map(|value| store_string(&mut env, value))
        .unwrap_or(NO_RESULT)
}

pub(super) fn html(mut env: FunctionEnvMut<HostState>, descriptor: i32) -> i32 {
    let Some(value) = env.data().descriptors.get(descriptor) else {
        return INVALID_DESCRIPTOR;
    };
    let html = match value {
        DescriptorValue::HtmlElement(element) => element.html(),
        DescriptorValue::HtmlElementList(elements) => Some(elements.html()),
        _ => None,
    };
    html.map(|value| store_string(&mut env, value))
        .unwrap_or(NO_RESULT)
}

pub(super) fn parse(text: &str, base_url: Option<&str>) -> HtmlDocument {
    HtmlDocument::parse(text, base_url)
}

fn store_string(env: &mut FunctionEnvMut<HostState>, value: String) -> i32 {
    env.data_mut()
        .descriptors
        .insert(DescriptorValue::Encoded(value.into_bytes()))
}

fn read_string(env: &FunctionEnvMut<HostState>, pointer: i32, length: i32) -> Result<String, ()> {
    if pointer < 0 || length < 0 {
        return Err(());
    }
    env.data()
        .read_bytes(env, pointer as u32, length as u32)
        .map_err(|_| ())
        .and_then(|bytes| String::from_utf8(bytes).map_err(|_| ()))
}

trait SelectWithRoot<'a, 'b> {
    fn select_with_root(&'a self, selector: &'b Selector) -> SelectRoot<'a, 'b>;
}

struct SelectRoot<'a, 'b> {
    root: Option<std::iter::Once<ElementRef<'a>>>,
    element_select: Option<scraper::element_ref::Select<'a, 'b>>,
    document_select: Option<scraper::html::Select<'a, 'b>>,
}

impl<'a> Iterator for SelectRoot<'a, '_> {
    type Item = ElementRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(root) = self.root.take() {
            root.into_iter().next()
        } else if let Some(select) = self.element_select.as_mut() {
            select.next()
        } else {
            self.document_select.as_mut().and_then(Iterator::next)
        }
    }
}

impl<'a, 'b> SelectWithRoot<'a, 'b> for Html {
    fn select_with_root(&'a self, selector: &'b Selector) -> SelectRoot<'a, 'b> {
        let root = ElementRef::wrap(self.tree.root())
            .filter(|element| selector.matches(element))
            .map(std::iter::once);
        SelectRoot {
            root,
            element_select: None,
            document_select: Some(self.select(selector)),
        }
    }
}

impl<'a, 'b> SelectWithRoot<'a, 'b> for ElementRef<'a> {
    fn select_with_root(&'a self, selector: &'b Selector) -> SelectRoot<'a, 'b> {
        SelectRoot {
            root: selector.matches(self).then(|| std::iter::once(*self)),
            element_select: Some(self.select(selector)),
            document_select: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HtmlDocument;
    use scraper::Selector;

    #[test]
    fn selects_elements_and_resolves_absolute_attributes() {
        let document = HtmlDocument::parse(
            r#"<main><a class="entry" href="/comic/1"> First <span>Comic</span></a></main>"#,
            Some("https://example.com/list/"),
        );
        let entries = document.select(&Selector::parse("a.entry").expect("selector"));
        assert_eq!(entries.0.len(), 1);
        assert_eq!(
            entries.0[0].attr("abs:href").as_deref(),
            Some("https://example.com/comic/1")
        );
        assert_eq!(entries.0[0].text(true).as_deref(), Some("First Comic"));
    }
}
