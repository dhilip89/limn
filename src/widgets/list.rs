use linked_hash_map::LinkedHashMap;

use event::Target;
use widget::{EventArgs, EventHandler};
use widget::style::Value;
use widget::property::{Property, PropChange};
use widget::property::states::*;
use drawable::rect::RectStyleField;
use resources::WidgetId;
use input::mouse::WidgetMouseButton;
use util::Color;

pub struct WidgetListItemSelected {
    widget: WidgetId,
}

static COLOR_LIST_ITEM_DEFAULT: Color = [0.3, 0.3, 0.3, 1.0];
static COLOR_LIST_ITEM_HOVER: Color = [0.6, 0.6, 0.6, 1.0];
static COLOR_LIST_ITEM_SELECTED: Color = [0.2, 0.2, 1.0, 1.0];

lazy_static! {
    pub static ref STYLE_LIST_ITEM: Vec<RectStyleField> = {
        let mut selector = LinkedHashMap::new();
        selector.insert(STATE_SELECTED.deref().clone(), COLOR_LIST_ITEM_SELECTED);
        selector.insert(STATE_HOVER.deref().clone(), COLOR_LIST_ITEM_HOVER);

        vec!{ RectStyleField::BackgroundColor(Value::Selector((selector, COLOR_LIST_ITEM_DEFAULT))) }
    };
}

pub struct ListHandler {
    selected: Option<WidgetId>,
}
impl ListHandler {
    pub fn new() -> Self {
        ListHandler { selected: None }
    }
}
impl EventHandler<WidgetListItemSelected> for ListHandler {
    fn handle(&mut self, event: &WidgetListItemSelected, mut args: EventArgs) {
        let selected = event.widget;
        if let Some(old_selected) = self.selected {
            if selected != old_selected {
                args.queue.push(Target::SubTree(old_selected), PropChange::Remove(Property::Selected));
            }
        }
        self.selected = Some(selected);
    }
}

pub struct ListItemHandler {
    list_id: WidgetId,
}
impl ListItemHandler {
    pub fn new(list_id: WidgetId) -> Self {
        ListItemHandler { list_id: list_id }
    }
}
impl EventHandler<WidgetMouseButton> for ListItemHandler {
    fn handle(&mut self, _: &WidgetMouseButton, mut args: EventArgs) {
       if !args.widget.props.contains(&Property::Selected) {
            args.queue.push(Target::SubTree(args.widget.id), PropChange::Add(Property::Selected));
            let event = WidgetListItemSelected { widget: args.widget.id };
            args.queue.push(Target::Widget(self.list_id), event);
        }
    }
}
