//! Functions to convert the data taken from the AST into documentation.
//! Borrows ideas from librustdoc's Clean.

mod wrappers;
mod doc_containers;

pub use convert::doc_containers::*;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fmt::{self, Display};

use serde::ser::{Serialize};
use serde::de::{Deserialize};
use syntax::abi;
use syntax::ast;
use syntax::print::pprust;
use syntax::ptr::P;

use document::{self, Attributes, CrateInfo, PathSegment, ModPath};
use store::Store;
use visitor::OxidocVisitor;

use convert::wrappers::*;
use convert::doc_containers::*;

pub struct Context {
    pub store_path: PathBuf,
    pub crate_info: CrateInfo,
}

pub trait Convert<T> {
    fn convert(&self, context: &Context) -> T;
}

impl<T: Convert<U>, U> Convert<Vec<U>> for [T] {
    fn convert(&self, cx: &Context) -> Vec<U> {
        self.iter().map(|x| x.convert(cx)).collect()
    }
}

impl<T: Convert<U>, U> Convert<U> for P<T> {
    fn convert(&self, cx: &Context) -> U {
        (**self).convert(cx)
    }
}

impl<T: Convert<U>, U> Convert<Option<U>> for Option<T> {
    fn convert(&self, cx: &Context) -> Option<U> {
        self.as_ref().map(|v| v.convert(cx))
    }
}

impl Convert<Unsafety> for ast::Unsafety {
    fn convert(&self, context: &Context) -> Unsafety {
        match *self {
            ast::Unsafety::Normal => Unsafety::Normal,
            ast::Unsafety::Unsafe => Unsafety::Unsafe,
        }
    }
}

impl Convert<Constness> for ast::Constness {
    fn convert(&self, context: &Context) -> Constness {
        match *self {
            ast::Constness::Const    => Constness::Const,
            ast::Constness::NotConst => Constness::NotConst,
        }
    }
}

impl Convert<Visibility> for ast::Visibility{
    fn convert(&self, context: &Context) -> Visibility {
        match *self {
            ast::Visibility::Public    => Visibility::Public,
            ast::Visibility::Inherited => Visibility::Inherited,
            _                          => Visibility::Private,
        }
    }
}

impl Convert<Abi> for abi::Abi {
    fn convert(&self, context: &Context) -> Abi {
        match *self {
            abi::Abi::Cdecl             => Abi::Cdecl,
            abi::Abi::Stdcall           => Abi::Stdcall,
            abi::Abi::Fastcall          => Abi::Fastcall,
            abi::Abi::Vectorcall        => Abi::Vectorcall,
            abi::Abi::Aapcs             => Abi::Aapcs,
            abi::Abi::Win64             => Abi::Win64,
            abi::Abi::SysV64            => Abi::SysV64,
            abi::Abi::PtxKernel         => Abi::PtxKernel,
            abi::Abi::Msp430Interrupt   => Abi::Msp430Interrupt,
            abi::Abi::Rust              => Abi::Rust,
            abi::Abi::C                 => Abi::C,
            abi::Abi::System            => Abi::System,
            abi::Abi::RustIntrinsic     => Abi::RustIntrinsic,
            abi::Abi::RustCall          => Abi::RustCall,
            abi::Abi::PlatformIntrinsic => Abi::PlatformIntrinsic,
            abi::Abi::Unadjusted        => Abi::Unadjusted
        }
    }
}

impl<'a> Convert<Store> for OxidocVisitor<'a> {
    fn convert(&self, context: &Context) -> Store {
        debug!("Converting store");
        let mut store = Store::new(context.store_path.clone());

        let documents = self.crate_module.convert(context);

        for doc in &store.documents {
            debug!("{:?}", doc);
        }

        store.documents = documents;

        store
    }
}

impl Convert<Vec<NewDocTemp_>> for document::Module {
    fn convert(&self, context: &Context) -> Vec<NewDocTemp_> {
        let mut docs: Vec<NewDocTemp_> = vec![];

        docs.extend(self.consts.iter().map(|x| x.convert(context)));
        docs.extend(self.traits.iter().map(|x| x.convert(context)));
        docs.extend(self.fns.iter().map(|x| x.convert(context)));
        docs.extend(self.mods.iter().flat_map(|x| x.convert(context)));
        // structs
        // imports
        // unions
        // enums
        // foreigns
        // typedefs
        // statics
        // traits
        // impls
        // macros
        // def_traits

        let name = match self.ident {
            Some(id) => id.convert(context),
            None     => context.crate_info.package.name.clone(),
        };

        let mod_doc = NewDocTemp_ {
            name: name.clone(),
            attrs: self.attrs.convert(context),
            mod_path: self.path.clone(),
            visibility: Some(self.vis.convert(context)),
            inner_data: ModuleDoc(Module {
                is_crate: self.is_crate,
            }),
            links: HashMap::new(),
        };

        docs.push(mod_doc);

        docs
    }
}

impl Convert<NewDocTemp_> for document::Constant {
    fn convert(&self, context: &Context) -> NewDocTemp_ {
        NewDocTemp_ {
            name: self.ident.convert(context),
            attrs: self.attrs.convert(context),
            mod_path: self.path.clone(),
            visibility: Some(self.vis.convert(context)),
            inner_data: ConstDoc(Constant {
                type_: self.type_.convert(context),
                expr: self.expr.convert(context),
            }),
            links: HashMap::new(),
        }
    }
}

impl Convert<NewDocTemp_> for document::Function {
    fn convert(&self, context: &Context) -> NewDocTemp_ {
        NewDocTemp_ {
            name: self.ident.convert(context),
            attrs: self.attrs.convert(context),
            mod_path: self.path.clone(),
            visibility: Some(self.vis.convert(context)),
            inner_data: FnDoc(Function {
                header: self.decl.convert(context),
                generics: Generics { } ,
                unsafety: self.unsafety.convert(context),
                constness: self.constness.convert(context),
                abi: self.abi.convert(context),
            }),
            links: HashMap::new(),
        }
    }
}

impl Convert<MethodSig> for ast::MethodSig {
    fn convert(&self, context: &Context) -> MethodSig {
        MethodSig {
            unsafety: self.unsafety.convert(context),
            constness: self.constness.node.convert(context),
            abi: self.abi.convert(context),
            header: self.decl.convert(context),
        }
    }
}

impl Convert<NewDocTemp_> for document::Trait {
    fn convert(&self, context: &Context) -> NewDocTemp_ {

        NewDocTemp_ {
            name: self.ident.convert(context),
            attrs: self.attrs.convert(context),
            mod_path: self.path.clone(),
            visibility: Some(self.vis.convert(context)),
            inner_data: TraitDoc(Trait {
                unsafety: self.unsafety.convert(context),
            }),
            links: self.items.convert(context),
        }
    }
}

impl Convert<NewDocTemp_> for document::TraitItem {
    fn convert(&self, context: &Context) -> NewDocTemp_ {
        NewDocTemp_ {
            name: self.ident.convert(context),
            attrs: self.attrs.convert(context),
            mod_path: self.path.clone(),
            visibility: Some(Visibility::Inherited),
            inner_data: TraitItemDoc(TraitItem {
                node: self.node.convert(context),
            }),
            links: HashMap::new(),
        }
    }
}

impl Convert<DocRelatedItems> for [document::TraitItem] {
    fn convert(&self, context: &Context) -> DocRelatedItems {
        let mut consts = Vec::new();
        let mut methods = Vec::new();
        let mut types = Vec::new();
        let mut macros = Vec::new();

        for item in self {
            match item.node {
                ast::TraitItemKind::Const(..) => consts.push(item.clone()),
                ast::TraitItemKind::Method(..) => methods.push(item.clone()),
                ast::TraitItemKind::Type(..) => types.push(item.clone()),
                ast::TraitItemKind::Macro(..) => macros.push(item.clone()),
            }
        }

        let conv = |items: Vec<document::TraitItem>| {
            items.iter().cloned().map(|item|
                                      DocLink {
                                          name: item.ident.convert(context),
                                          path: item.path.clone(),
                                      }
            ).collect()
        };

        let mut links = HashMap::new();
        links.insert(DocType::TraitItemConst, conv(consts));
        links.insert(DocType::TraitItemMethod, conv(methods));
        links.insert(DocType::TraitItemType, conv(types));
        links.insert(DocType::TraitItemMacro, conv(macros));
        links
    }
}

impl Convert<TraitItemKind> for ast::TraitItemKind {
    fn convert(&self, context: &Context) -> TraitItemKind {
        match *self {
            ast::TraitItemKind::Const(ref ty, ref expr) => {
                TraitItemKind::Const(ty.convert(context), expr.convert(context))
            },
            ast::TraitItemKind::Method(ref sig, ref block) => {
                TraitItemKind::Method(sig.convert(context))
            },
            ast::TraitItemKind::Type(ref bounds, ref ty) => {
                TraitItemKind::Type(ty.convert(context))
            },
            ast::TraitItemKind::Macro(ref mac) => {
                TraitItemKind::Macro(mac.convert(context))
            },
        }
    }
}

impl Convert<NewDocTemp_> for document::Struct {
    fn convert(&self, context: &Context) -> NewDocTemp_ {
        NewDocTemp_ {
            name: self.ident.convert(context),
            attrs: self.attrs.convert(context),
            mod_path: self.path.clone(),
            visibility: Some(Visibility::Inherited),
            inner_data: StructDoc(Struct {
                fields: HashMap::new(),//self.fields.clean(context),
            }),
            links: HashMap::new(),
        }
    }
}

impl Convert<DocRelatedItems> for [document::StructField] {
    fn convert(&self, context: &Context) -> DocRelatedItems {
        let mut links = HashMap::new();
        links
    }
}

impl Convert<String> for ast::FnDecl {
    fn convert(&self, context: &Context) -> String {
        pprust::to_string(|s| s.print_fn_args_and_ret(self))
    }
}

impl Convert<String> for ast::Ty {
    fn convert(&self, context: &Context) -> String {
        pprust::ty_to_string(self)
    }
}

impl Convert<String> for ast::Expr {
    fn convert(&self, context: &Context) -> String {
        pprust::expr_to_string(self)
    }
}

impl Convert<String> for ast::Ident {
    fn convert(&self, context: &Context) -> String {
        pprust::ident_to_string(*self)
    }
}

impl Convert<String> for ast::Name {
    fn convert(&self, context: &Context) -> String {
        pprust::to_string(|s| s.print_name(*self))
    }
}

impl Convert<String> for ast::Mac {
    fn convert(&self, context: &Context) -> String {
        pprust::mac_to_string(self)
    }
}

impl Convert<Attributes> for [ast::Attribute] {
    fn convert(&self, context: &Context) -> Attributes {
        Attributes::from_ast(self)
    }
}
