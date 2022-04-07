//
// color_swatch.rs
// Copyright (C) 2022 gmg137 <gmg137 AT live.com>
// Distributed under terms of the GPL-3.0-or-later license.
//
use glib::clone;
use glib::{ParamSpec, ParamSpecString};
use gtk::gdk::ContentProvider;
use gtk::gdk::RGBA;
use gtk::glib;
use gtk::graphene::Rect;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::DragSource;
use gtk::Orientation;
use gtk::Snapshot;
use once_cell::sync::Lazy;
use std::cell::RefCell;

glib::wrapper! {
    pub struct ColorSwatch(ObjectSubclass<ExColorSwatch>) @extends gtk::Widget;
}

impl ColorSwatch {
    pub fn new(color: &str) -> Self {
        glib::Object::new(&[("color", &color)]).expect("Failed to create a ColorSwatch")
    }
}

#[derive(Debug, Default)]
pub struct ExColorSwatch {
    color: RefCell<Option<String>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ExColorSwatch {
    const NAME: &'static str = "ColorSwatch";
    type Type = ColorSwatch;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_css_name("colorswatch");
    }
}

impl ObjectImpl for ExColorSwatch {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);

        let source = DragSource::new();
        source.connect_prepare(
            clone!(@weak obj =>@default-return None, move| _source, _x, _y| {
                let swatch = obj.imp();
                let color = swatch.color.borrow();
                let color = color.as_ref().cloned().unwrap();
                let rgba = RGBA::parse(&color).unwrap();
                Some(ContentProvider::for_value(&rgba.to_value()))
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
            "color" => *self.color.borrow_mut() = Some(value.get().unwrap()),
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for ExColorSwatch {
    fn snapshot(&self, widget: &Self::Type, snapshot: &Snapshot) {
        let w = widget.width() as f32;
        let h = widget.height() as f32;

        let imp = widget.imp();
        let color = imp.color.borrow();
        let color = color.as_ref().unwrap();
        let color = RGBA::parse(color).unwrap();

        let rect = Rect::new(0., 0., w, h);
        snapshot.append_color(&color, &rect)
    }

    fn measure(
        &self,
        _widget: &Self::Type,
        orientation: Orientation,
        _for_size: i32,
    ) -> (i32, i32, i32, i32) {
        if orientation == Orientation::Horizontal {
            (48, 48, -1, -1)
        } else {
            (32, 32, -1, -1)
        }
    }
}
