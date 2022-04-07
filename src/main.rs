use gtk::gdk::{ContentProvider, Rectangle, BUTTON_PRIMARY, BUTTON_SECONDARY};
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::*;
mod canvas_item;
mod color_swatch;
mod css_button;
use canvas_item::*;
use color_swatch::*;
use css_button::*;
pub static mut ITEM_ID: i32 = 0;

fn main() {
    let application = gtk::Application::new(Some("com.github.gmg137.gtk4-dnd"), Default::default());
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(application: &gtk::Application) {
    let colors = [
        "red", "green", "blue", "magenta", "orange", "gray", "black", "yellow", "white", "gray",
        "brown", "pink", "cyan", "bisque", "gold", "maroon", "navy", "orchid", "olive", "peru",
        "salmon", "silver", "wheat",
    ];

    let provider = CssProvider::new();
    provider.load_from_path("dnd.css");
    let gdk_display = gdk::Display::default().unwrap();
    StyleContext::add_provider_for_display(&gdk_display, &provider, 800);

    let window = gtk::ApplicationWindow::new(application);

    window.set_title(Some("Drag-and-Drop"));
    window.set_default_size(640, 480);

    let box1 = Box::new(Orientation::Vertical, 0);
    window.set_child(Some(&box1));

    let box2 = Box::new(Orientation::Horizontal, 0);
    box1.append(&box2);

    let canvas = canvas_new();
    box2.append(&canvas);

    unsafe {
        ITEM_ID = 0;
    }

    let (mut x, mut y) = (40., 40.);
    for _ in 0..4 {
        let item = CanvasItem::new();
        canvas.put(&item, x, y);
        item.apply_transform();
        x += 150.;
        y += 100.;
    }

    let separator = Separator::new(Orientation::Horizontal);
    box1.append(&separator);

    let sw = ScrolledWindow::new();
    sw.set_policy(PolicyType::Automatic, PolicyType::Never);
    box1.append(&sw);

    let box3 = Box::new(Orientation::Horizontal, 0);
    box3.add_css_class("linked");
    sw.set_child(Some(&box3));

    for i in colors {
        box3.append(&ColorSwatch::new(i));
    }

    box3.append(&CssButton::new("rainbow1"));
    box3.append(&CssButton::new("rainbow2"));
    box3.append(&CssButton::new("rainbow3"));

    window.show();
}

fn canvas_new() -> Fixed {
    let canvas = Fixed::new();
    canvas.set_hexpand(true);
    canvas.set_vexpand(true);

    let source = DragSource::new();
    source.set_actions(gdk::DragAction::MOVE);
    source.connect_prepare(|source, x, y| {
        let canvas = source.widget();
        let item = canvas.pick(x, y, PickFlags::DEFAULT).unwrap();

        if let Some(item) = item.ancestor(CanvasItem::static_type()) {
            unsafe {
                canvas.set_data::<Widget>("dragged-item", item.to_owned());
                return Some(ContentProvider::for_value(&item.to_value()));
            }
        }
        None
    });
    source.connect_drag_begin(|source, _drag| {
        let fixed_widget = source.widget();
        unsafe {
            let item_label = fixed_widget.data::<Label>("dragged-item").unwrap();
            let item_label = item_label.as_ref();

            let paintable = WidgetPaintable::new(Some(item_label));

            if let Some(canvas_item) = item_label.ancestor(CanvasItem::static_type()) {
                let canvas_item: CanvasItem = canvas_item.downcast().unwrap();
                source.set_icon(Some(&paintable), canvas_item.r(), canvas_item.r());
                canvas_item.set_opacity(0.3);
            }
        }
    });
    source.connect_drag_end(move |source, _drag, _| unsafe {
        let canvas = source.widget();
        if let Some(item) = canvas.steal_data::<Widget>("dragged-item") {
            item.set_opacity(1.0);
        }
    });
    source.connect_drag_cancel(|_, _, _| false);
    canvas.add_controller(&source);

    let dest = DropTarget::new(Widget::static_type(), gdk::DragAction::MOVE);
    dest.connect_drop(|_target, value, x, y| {
        let item = value.get::<Widget>().unwrap();
        let item: CanvasItem = item.downcast().unwrap();

        let canvas = item.parent().unwrap();
        let last_child = canvas.last_child().unwrap();
        if item != last_child {
            item.insert_after(&canvas, Some(&last_child));
        }
        canvas
            .downcast::<Fixed>()
            .unwrap()
            .move_(&item, x - item.r() as f64, y - item.r() as f64);
        true
    });
    canvas.add_controller(&dest);

    let gesture = GestureClick::new();
    gesture.set_button(0);
    gesture.connect_pressed(clone!(@weak canvas =>move |gesture, _n_press, x, y| {
        let widget = gesture.widget();
        let child = widget.pick(x, y, PickFlags::DEFAULT).unwrap();
        let child = child.ancestor(child.type_()).unwrap();

        if gesture.current_button() == BUTTON_SECONDARY {
            let menu = Popover::new();
            menu.set_parent(&widget);
            menu.set_has_arrow(false);
            let rect = Rectangle::new(x as i32, y as i32, 1, 1);
            menu.set_pointing_to(Some(&rect));
            let box1 = Box::new(Orientation::Vertical, 0);
            menu.set_child(Some(&box1));

            let item = Button::with_label("New");
            item.set_has_frame(false);
            item.connect_clicked(clone!(@weak canvas =>move |button| {
                let popover = button.ancestor(Popover::static_type()).unwrap();
                let popover: Popover = popover.downcast().unwrap();
                let rect = popover.pointing_to().1;

                let item = CanvasItem::new();
                canvas.put(&item, rect.x() as f64, rect.y() as f64);
                item.apply_transform();

                popover.popdown();
            }));
            box1.append(&item);

            let item = Separator::new(Orientation::Horizontal);
            box1.append(&item);

            let item = Button::with_label("Edit");
            item.set_has_frame(false);
            item.set_sensitive(child != widget);
            item.connect_clicked(clone!(@weak child => move|button|{
                let item = child.ancestor(CanvasItem::static_type()).unwrap();
                let item: CanvasItem  = item.downcast().unwrap();

                let popover = button.ancestor(Popover::static_type()).unwrap();
                let popover: Popover = popover.downcast().unwrap();
                popover.popdown();

                if !item.is_editing() {
                    item.start_editing();
                }
            }));
            box1.append(&item);

            let item = Separator::new(Orientation::Horizontal);
            box1.append(&item);

            let item = Button::with_label("Delete");
            item.set_has_frame(false);
            item.set_sensitive(child != widget);
            item.connect_clicked(clone!(@weak child => move|button|{
                let canvas = child.parent().unwrap();
                let canvas: Fixed  = canvas.downcast().unwrap();
                canvas.remove(&child);
                let popover = button.ancestor(Popover::static_type()).unwrap();
                let popover: Popover = popover.downcast().unwrap();
                popover.popdown();
            }));
            box1.append(&item);

            menu.popup();
        }
    }));

    gesture.connect_released(|gesture, _, x, y| {
        let widget = gesture.widget();
        let child = widget.pick(x, y, PickFlags::DEFAULT).unwrap();
        if let Some(item) = child.ancestor(CanvasItem::static_type()) {
            if gesture.current_button() == BUTTON_PRIMARY {
                let item: CanvasItem = item.downcast().unwrap();
                if item.is_editing() {
                    item.stop_editing();
                } else {
                    item.start_editing();
                }
            }
        }
    });
    canvas.add_controller(&gesture);
    canvas
}
