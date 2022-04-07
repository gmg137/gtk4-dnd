//
// canvas_item.rs
// Copyright (C) 2022 gmg137 <gmg137 AT live.com>
// Distributed under terms of the GPL-3.0-or-later license.
//

use gtk::gdk::DragAction;
use gtk::gdk::RGBA;
use gtk::glib;
use gtk::glib::clone;
use gtk::glib::Type;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::Box;
use gtk::CssProvider;
use gtk::DropTarget;
use gtk::Entry;
use gtk::Fixed;
use gtk::GestureClick;
use gtk::GestureRotate;
use gtk::Label;
use gtk::Orientation;
use gtk::Scale;
use gtk::Settings;
use std::cell::RefCell;
use std::f64::consts::PI;

use crate::ITEM_ID;

glib::wrapper! {
    pub struct CanvasItem(ObjectSubclass<ExCanvasItem>) @extends gtk::Widget;
}

impl Default for CanvasItem {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasItem {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create CanvasItem")
    }

    pub fn apply_transform(&self) {
        let imp = self.imp();
        imp.apply_transform();
    }

    pub fn r(&self) -> i32 {
        let imp = self.imp();
        *imp.r.borrow() as i32
    }

    pub fn is_editing(&self) -> bool {
        let imp = self.imp();
        imp.editor.borrow().is_some()
    }

    pub fn stop_editing(&self) {
        if !self.is_editing() {
            return;
        }
        let imp = self.imp();

        let editor = imp.editor.borrow();
        let editor = editor.as_ref().cloned().unwrap();

        let fixed = imp.fixed.borrow();
        let fixed = fixed.as_ref().cloned().unwrap();
        let fixed: Fixed = fixed.downcast().unwrap();
        fixed.remove(&editor);

        imp.editor.replace(None);
    }

    pub fn start_editing(&self) {
        let item = self.imp();
        let canvas = self.parent().unwrap();
        if self.is_editing() {
            return;
        }

        let box1 = Box::new(Orientation::Vertical, 12);
        *item.editor.borrow_mut() = Some(box1.upcast::<gtk::Widget>());

        let entry = Entry::new();

        let label = item.label.borrow();
        let label = label.as_ref().cloned().unwrap();
        let label = label.downcast::<Label>().unwrap();
        entry.set_text(&label.text());

        entry.set_width_chars(12);
        entry.connect_notify_local(
            Some("text"),
            clone!(@weak self as tag => move |editable, _pspec| {
                let imp = tag.imp();
                let label = imp.label.borrow();
                let label = label.as_ref().cloned().unwrap();
                let label = label.downcast::<Label>().unwrap();
                label.set_text(&editable.text());
            }),
        );
        entry.connect_activate(clone!(@weak self as this => move|_entry| {
            if !this.is_editing() {
                return;
            }
            let imp = this.imp();
            {
                let editor = imp.editor.borrow();
                let editor = editor.as_ref().cloned().unwrap();
                let fixed = editor.parent().unwrap();
                let fixed:Fixed = fixed.downcast().unwrap();
                fixed.remove(&editor);
            }
            imp.editor.replace(None);
        }));

        let scale = Scale::with_range(Orientation::Horizontal, 0., 360., 1.);
        scale.set_draw_value(false);
        scale.set_value(*item.angle.borrow() as f64 % 360.);

        scale.connect_value_changed(clone!(@weak self as this => move |range| {
            let item = this.imp();
            *item.angle.borrow_mut() = range.value() as f32;
            item.apply_transform();
        }));

        let editor = item.editor.borrow();
        let editor = editor.as_ref().cloned().unwrap();
        let editor: Box = editor.downcast().unwrap();
        editor.append(&entry);
        editor.append(&scale);

        if let Some((x, y)) = self.translate_coordinates(&canvas, 0., 0.) {
            let canvas: Fixed = canvas.downcast().unwrap();
            canvas.put(&editor, x, y + 2. * *item.r.borrow() as f64);
        }
        entry.grab_focus();
    }
}

#[derive(Debug, Default)]
pub struct ExCanvasItem {
    fixed: RefCell<Option<gtk::Widget>>,
    label: RefCell<Option<gtk::Widget>>,

    angle: RefCell<f32>,
    delta: RefCell<f32>,
    r: RefCell<f32>,

    editor: RefCell<Option<gtk::Widget>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ExCanvasItem {
    const NAME: &'static str = "CanvasItem";
    type Type = CanvasItem;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<gtk::BinLayout>();
        klass.set_css_name("item");
    }
}

impl ObjectImpl for ExCanvasItem {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        unsafe {
            ITEM_ID += 1;
        }
        *self.editor.borrow_mut() = None;

        let label = unsafe { format!("Item {}", ITEM_ID) };
        let label = gtk::Label::new(Some(&label));
        *self.label.borrow_mut() = Some(label.upcast::<gtk::Widget>());
        let label = self.label.borrow();
        let label = label.as_ref().cloned().unwrap();
        let label = label.downcast::<Label>().unwrap();
        label.add_css_class("canvasitem");

        let fixed = Fixed::new();
        *self.fixed.borrow_mut() = Some(fixed.upcast::<gtk::Widget>());
        let fixed = self.fixed.borrow();
        let fixed = fixed.as_ref().cloned().unwrap();
        let fixed = fixed.downcast::<Fixed>().unwrap();
        fixed.set_parent(obj);
        fixed.put(&label, 0., 0.);
        label.add_css_class("frame");
        let id = unsafe { format!("item{}", ITEM_ID) };
        label.set_widget_name(&id);

        let mut color = RGBA::parse("yellow").unwrap();
        if theme_is_dark() {
            color = RGBA::parse("blue").unwrap();
        }

        self.set_label_color(color);

        *self.angle.borrow_mut() = 0.;

        let dest = DropTarget::new(Type::INVALID, DragAction::COPY);
        dest.set_types(&[RGBA::static_type(), Type::STRING]);
        dest.connect_drop(
            clone!(@weak obj => @default-return false, move |_dest, value, _x, _y| {
                let item = obj.imp();
                if value.type_() == RGBA::static_type() {
                    item.set_label_color(value.get::<RGBA>().unwrap());
                } else if value.type_() == Type::STRING {
                    item.set_css(value.get::<String>().unwrap());
                }
                true
            }),
        );
        label.add_controller(&dest);

        let gesture = GestureRotate::new();
        gesture.connect_angle_changed(clone!(@weak obj => move | _gesture, angle, _delta | {
                let item = obj.imp();
                *item.delta.borrow_mut() = (angle as f64 / PI * 180.) as f32;
                item.apply_transform();
            }
        ));

        gesture.connect_end(clone!(@weak obj=>move|_, _| {
            let item = obj.imp();
            *item.angle.borrow_mut() = *item.angle.borrow() + *item.delta.borrow();
            *item.delta.borrow_mut() = 0.;
        }));
        obj.add_controller(&gesture);

        let gesture = GestureClick::new();
        gesture.connect_released(clone!(@weak obj =>move | _, _, _, _ | {
                let canvas = obj.parent().unwrap();
                let last_child = canvas.last_child().unwrap();

                if obj != last_child {
                    obj.insert_after(&canvas, Some(&last_child));
                }
            }
        ));
        obj.add_controller(&gesture);
    }

    fn dispose(&self, _obj: &Self::Type) {
        if let Some(child) = self.fixed.borrow_mut().take() {
            child.unparent();
        }
        if let Some(child) = self.label.borrow_mut().take() {
            child.unparent();
        }
        if let Some(child) = self.editor.borrow_mut().take() {
            child.unparent();
        }
    }
}

impl WidgetImpl for ExCanvasItem {
    fn map(&self, widget: &Self::Type) {
        self.parent_map(widget);
        self.apply_transform();
    }
}

impl ExCanvasItem {
    fn apply_transform(&self) {
        let label = self.label.borrow();
        let label = label.as_ref().cloned().unwrap();
        let x = label.allocated_width() as f32 / 2.;
        let y = label.allocated_height() as f32 / 2.;
        let r = (x * x + y * y).sqrt();
        *self.r.borrow_mut() = r;
        let transform = gtk::gsk::Transform::new();
        let transform = transform
            .translate(&gtk::graphene::Point::new(r, r))
            .unwrap();
        let transform = transform
            .rotate(*self.angle.borrow() + *self.delta.borrow())
            .unwrap();
        let transform = transform
            .translate(&gtk::graphene::Point::new(-x, -y))
            .unwrap();
        let fixed = self.fixed.borrow();
        let fixed = fixed.as_ref().cloned().unwrap();
        let fixed = fixed.downcast::<Fixed>().unwrap();
        fixed.set_child_transform(&label, Some(&transform));
    }

    fn set_css(&self, class: String) {
        let label = self.label.borrow();
        let label = label.as_ref().cloned().unwrap();
        let context = label.style_context();
        unsafe {
            if let Some(provider) = context.data::<CssProvider>("style-provider") {
                context.remove_provider(provider.as_ref());
            }
        }
        unsafe {
            if let Some(old_class) = label.data::<String>("css-class") {
                label.remove_css_class(old_class.as_ref());
            }
        }
        unsafe {
            label.set_data::<String>("css-class", class.to_owned());
        }
        label.add_css_class(&class);
    }

    fn set_label_color(&self, color: RGBA) {
        let label = self.label.borrow();
        let label = label.as_ref().cloned().unwrap();

        let color_str = color.to_string();
        let css_str = format!("* {{ background: {}; }}", color_str);

        let context = label.style_context();
        unsafe {
            if let Some(provider) = context.data::<CssProvider>("style-provider") {
                context.remove_provider(provider.as_ref());
            }
            if let Some(old_class) = label.data::<String>("css-class") {
                label.remove_css_class(old_class.as_ref());
            }
        }

        let provider = CssProvider::new();
        provider.load_from_data(css_str.as_bytes());
        label.style_context().add_provider(&provider, 800);
        unsafe {
            context.set_data("style-provider", provider);
        }
    }
}

fn theme_is_dark() -> bool {
    let settings = Settings::default().unwrap();
    settings.is_gtk_application_prefer_dark_theme()
}
