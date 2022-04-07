//
// css_button.rs
// Copyright (C) 2022 gmg137 <gmg137 AT live.com>
// Distributed under terms of the GPL-3.0-or-later license.
//
use glib::clone;
use glib::{ParamSpec, ParamSpecString};
use gtk::gdk::ContentProvider;
use gtk::subclass::prelude::*;
use gtk::DragSource;
use gtk::{glib, Widget};
use gtk::{prelude::*, Image};
use once_cell::sync::Lazy;
use std::cell::RefCell;

glib::wrapper! {
    pub struct CssButton(ObjectSubclass<ExCssButton>)
        @extends gtk::Widget;
}

impl CssButton {
    pub fn new(color: &str) -> Self {
        glib::Object::new(&[("color", &color)]).expect("Failed to create a ColorSwatch")
    }
}

#[derive(Debug, Default)]
pub struct ExCssButton {
    image: RefCell<Option<Widget>>,
    color: RefCell<Option<String>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ExCssButton {
    const NAME: &'static str = "CssButton";
    type Type = CssButton;
    type ParentType = Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<gtk::BinLayout>();
    }
}

impl ObjectImpl for ExCssButton {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        let image = Image::new();
        image.set_size_request(48, 32);
        image.set_parent(obj);
        *self.image.borrow_mut() = Some(image.upcast::<gtk::Widget>());

        let source = DragSource::new();
        source.connect_prepare(
            clone!(@weak obj =>@default-return None, move|source, _x, _y| {
                let imp = obj.imp();
                let image = imp.image.borrow();
                let image = image.as_ref().cloned().unwrap();
                let image:Image = image.downcast().unwrap();
                let paintable = image.paintable();
                source.set_icon(paintable.as_ref(), 0, 0);
                let color = imp.color.borrow();
                let color = color.as_ref().unwrap();
                Some(ContentProvider::for_value(&color.to_value()))
            }),
        );
        obj.add_controller(&source);
    }

    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![ParamSpecString::new(
                "color",
                "color",
                "color",
                Some(""),
                glib::ParamFlags::READWRITE,
            )]
        });
        PROPERTIES.as_ref()
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> glib::Value {
        match pspec.name() {
            "color" => self.color.borrow().to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(&self, _tag: &Self::Type, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
        match pspec.name() {
            "color" => {
                *self.color.borrow_mut() = Some(value.get().unwrap());
                let image = self.image.borrow();
                let image = image.as_ref().cloned().unwrap();
                let image = image.downcast::<Image>().unwrap();
                let color = value.get::<String>().unwrap();
                image.add_css_class(&color);
                unsafe {
                    image.set_data("css-class", color);
                }
            }
            _ => unimplemented!(),
        }
    }

    fn dispose(&self, _obj: &Self::Type) {
        if let Some(image) = self.image.borrow_mut().take() {
            image.unparent();
        }
    }
}

impl WidgetImpl for ExCssButton {}
