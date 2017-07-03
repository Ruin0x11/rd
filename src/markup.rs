use std::fmt;

use ansi_term::Style;
use convert::*;
use document::{Attributes, FnKind, ModPath};
use markdown_renderer;
use term_size;

pub enum Markup {
    Header(String),
    Section(String),
    Block(String),
    Markdown(String),
    Rule(usize),
    LineBreak,
}

use self::Markup::*;

fn get_term_width() -> u16 {
    match term_size::dimensions() {
        Some((w, _)) => w as u16,
        None         => 80,
    }
}

impl fmt::Display for Markup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match *self {
            Header(ref text) => Style::new().bold().paint(format!("==== {}", text)).to_string(),
            Section(ref text) => Style::new().bold().paint(format!("== {}", text)).to_string(),
            Block(ref text) => text.clone(),
            Markdown(ref md) => {
                let width = get_term_width();
                markdown_renderer::render_ansi(md, width)
            }
            Rule(ref count) => "-".repeat(*count),
            LineBreak => "".to_string()
        };
        write!(f, "{}", string)
    }
}

pub struct MarkupDoc {
    pub parts: Vec<Markup>,
}

impl MarkupDoc {
    pub fn new(parts: Vec<Markup>) -> Self {
        MarkupDoc {
            parts: parts,
        }
    }
}

impl fmt::Display for MarkupDoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for part in self.parts.iter() {
            part.fmt(f)?;
            write!(f, "\n")?;
        }
        Ok(())
    }
}

pub trait Format {
    fn format(&self) -> MarkupDoc;
}

impl Format for NewDocTemp_ {
    fn format(&self) -> MarkupDoc {
        let header = doc_header(self);
        let info = doc_inner_info(self);
        let signature = doc_signature(self);
        let body = doc_body(self);

        let mut result = Vec::new();
        result.extend(header.parts);
        result.extend(info.parts);
        result.extend(signature.parts);
        result.extend(body.parts);

        MarkupDoc::new(result)
    }
}

impl Format for ModPath {
    fn format(&self) -> MarkupDoc {
        MarkupDoc::new(vec![Header(self.to_string())])
    }
}

fn doc_header(data: &NewDocTemp_) -> MarkupDoc {
    let name = match data.inner_data {
        DocInnerData::FnDoc(..) => "Function",
        DocInnerData::StructDoc(..) => "Struct",
        DocInnerData::ConstDoc(..) => "Constant",
        DocInnerData::EnumDoc(..) => "Enum",
        DocInnerData::TraitDoc(..) => "Trait",
        DocInnerData::TraitItemDoc(..) => "Trait Item",
        DocInnerData::ModuleDoc(..) => "Module",
    };

    MarkupDoc::new(vec![Block(format!("({})", data.crate_info)),
                        Header(format!("{} {}", name, data.mod_path))])
}

fn doc_inner_info(data: &NewDocTemp_) -> MarkupDoc {
    let markup = match data.inner_data {
        DocInnerData::FnDoc(ref func) => {
            match func.kind {
                FnKind::MethodFromImpl => Header(format!("Impl on type {}", data.mod_path.parent().unwrap())),
                _                      => LineBreak,
            }
        },
        DocInnerData::TraitItemDoc(..) => {
            Header(format!("From trait {}", data.mod_path.parent().unwrap()))
        }
        DocInnerData::StructDoc(..) |
        DocInnerData::ConstDoc(..) |
        DocInnerData::EnumDoc(..) |
        DocInnerData::TraitDoc(..) |
        DocInnerData::ModuleDoc(..) => LineBreak,
    };
    MarkupDoc::new(vec![markup])
}

fn doc_signature(data: &NewDocTemp_) -> MarkupDoc {
    let vis_string = match data.visibility {
        Some(ref v) => v.to_string(),
        None    => "".to_string(),
    };

    let header = match data.inner_data {
        DocInnerData::FnDoc(ref func) => {
            format!("fn {} {}", data.name, func.header)
        },
        DocInnerData::ModuleDoc(..) => {
            format!("mod {}", data.mod_path)
        },
        DocInnerData::EnumDoc(..) => {
            format!("enum {}", data.name)
        },
        DocInnerData::StructDoc(..) => {
            format!("struct {} {{ /* fields omitted */ }}", data.name)
        },
        DocInnerData::ConstDoc(ref const_) => {
            format!("const {}: {} = {}", data.name, const_.ty.name, const_.expr)
        },
        DocInnerData::TraitDoc(..) => {
            format!("trait {} {{ /* fields omitted */ }}", data.name)
        },
        DocInnerData::TraitItemDoc(ref item) => {
            format!("{}", trait_item(data, item))
        },
    };

    MarkupDoc::new(vec![
        Rule(10),
        LineBreak,
        Block(format!("  {} {}", vis_string, header)),
        LineBreak,
        Rule(10),
        LineBreak])
}

fn trait_item(data: &NewDocTemp_, item: &TraitItem) -> String {
    let item_string = match item.node {
        TraitItemKind::Const(ref ty, ref expr) => {
            let expr_string = match *expr {
                Some(ref e) => e.clone(),
                None    => "".to_string(),
            };
            format!("const {}: {} = {}", data.name, ty.name, expr_string)
        },
        TraitItemKind::Method(ref sig) => {
            format!("fn {} {}", data.name, sig.header)
        },
        TraitItemKind::Type(ref ty) => {
            let ty_string = match *ty {
                Some(ref t) => t.name.clone(),
                None    => "".to_string(),
            };
            format!("type {}", ty_string)
        },
        TraitItemKind::Macro(ref mac) => {
            format!("macro {} {}", data.name, mac)
        },
    };
    item_string
}

fn doc_body(data: &NewDocTemp_) -> MarkupDoc {
    data.attrs.format()
}

impl Format for Attributes {
    fn format(&self) -> MarkupDoc {
        let body = self.doc_strings.join("\n");

        MarkupDoc::new(vec![Markdown(body)])
    }
}