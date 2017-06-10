extern crate cassowary;
extern crate euclid;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate linked_hash_map;
#[macro_use]
extern crate maplit;

use std::collections::HashSet;
use std::ops::Drop;
use std::mem;

use cassowary::{Variable, Constraint};
use cassowary::WeightedRelation::*;
use cassowary::strength::*;

use self::constraint::ConstraintBuilder;

use euclid::{Point2D, Size2D, UnknownUnit};

pub type Length = euclid::Length<f64, UnknownUnit>;
pub type Size = Size2D<f64>;
pub type Point = Point2D<f64>;
pub type Rect = euclid::Rect<f64>;

pub type LayoutId = usize;

#[derive(Debug)]
pub enum VarUpdate {
    Left,
    Top,
    Width,
    Height,
}
#[derive(Clone)]
pub struct LayoutVars {
    pub left: Variable,
    pub top: Variable,
    pub right: Variable,
    pub bottom: Variable,
    pub width: Variable,
    pub height: Variable,
}
impl LayoutVars {
    pub fn new() -> Self {
        LayoutVars {
            left: Variable::new(),
            top: Variable::new(),
            right: Variable::new(),
            bottom: Variable::new(),
            width: Variable::new(),
            height: Variable::new(),
        }
    }
    pub fn get_var(&self, var: Variable) -> Option<VarUpdate> {
        if var == self.left { Some(VarUpdate::Left) }
        else if var == self.top { Some(VarUpdate::Top) }
        else if var == self.width { Some(VarUpdate::Width) }
        else if var == self.height { Some(VarUpdate::Height) }
        else { None }
    }
    pub fn array(&self) -> [Variable; 6] {
        [self.left, self.top, self.right, self.bottom, self.width, self.height]
    }
}

pub trait LayoutRef {
    fn layout_ref(&self) -> &LayoutVars;
}

impl<'a> LayoutRef for &'a mut Layout {
    fn layout_ref(&self) -> &LayoutVars {
        &self.vars
    }
}
impl LayoutRef for Layout {
    fn layout_ref(&self) -> &LayoutVars {
        &self.vars
    }
}
impl LayoutRef for LayoutVars {
    fn layout_ref(&self) -> &LayoutVars {
        self
    }
}

pub struct Layout {
    pub vars: LayoutVars,
    edit_vars: Vec<EditVariable>,
    constraints: HashSet<Constraint>,
    new_constraints: Vec<Constraint>,
    removed_constraints: Vec<Constraint>,
}
impl Layout {
    pub fn new() -> Self {
        let vars = LayoutVars::new();
        let mut new_constraints = Vec::new();
        new_constraints.push(vars.right - vars.left| EQ(REQUIRED) | vars.width);
        new_constraints.push(vars.bottom - vars.top | EQ(REQUIRED) | vars.height);
        // temporarily disabling this, as it tends to cause width/height to snap to 0
        //constraints.push(vars.width | GE(REQUIRED) | 0.0);
        //constraints.push(vars.height | GE(REQUIRED) | 0.0);
        Layout {
            vars: vars,
            edit_vars: Vec::new(),
            constraints: HashSet::new(),
            new_constraints: new_constraints,
            removed_constraints: Vec::new(),
        }
    }
    pub fn layout(&mut self) -> &mut Self {
        self
    }
    pub fn edit_left(&mut self) -> VariableEditable {
        let var = self.vars.left;
        VariableEditable::new(self, var)
    }
    pub fn edit_top(&mut self) -> VariableEditable {
        let var = self.vars.top;
        VariableEditable::new(self, var)
    }
    pub fn edit_right(&mut self) -> VariableEditable {
        let var = self.vars.right;
        VariableEditable::new(self, var)
    }
    pub fn edit_bottom(&mut self) -> VariableEditable {
        let var = self.vars.bottom;
        VariableEditable::new(self, var)
    }
    pub fn edit_width(&mut self) -> VariableEditable {
        let var = self.vars.width;
        VariableEditable::new(self, var)
    }
    pub fn edit_height(&mut self) -> VariableEditable {
        let var = self.vars.height;
        VariableEditable::new(self, var)
    }
    pub fn add<B: ConstraintBuilder>(&mut self, builder: B) {
        let new_constraints = builder.build(self);
        self.new_constraints.extend(new_constraints);
    }
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.new_constraints.push(constraint);
    }
    pub fn remove_constraint(&mut self, constraint: Constraint) {
        self.removed_constraints.push(constraint);
    }
    pub fn get_constraints(&mut self) -> Vec<Constraint> {
        let new_constraints = mem::replace(&mut self.new_constraints, Vec::new());
        for constraint in new_constraints.clone() {
            self.constraints.insert(constraint);
        }
        new_constraints
    }
    pub fn get_removed_constraints(&mut self) -> Vec<Constraint> {
        let removed_constraints = mem::replace(&mut self.removed_constraints, Vec::new());
        for ref constraint in &removed_constraints {
            self.constraints.remove(constraint);
        }
        removed_constraints
    }
    pub fn get_edit_vars(&mut self) -> Vec<EditVariable> {
        mem::replace(&mut self.edit_vars, Vec::new())
    }
}

pub struct VariableEditable<'a> {
    pub builder: &'a mut Layout,
    pub var: Variable,
    val: Option<f64>,
    strength: f64,
}
impl<'a> VariableEditable<'a> {
    pub fn new(builder: &'a mut Layout, var: Variable) -> Self {
        VariableEditable {
            builder: builder,
            var: var,
            val: None,
            strength: STRONG,
        }
    }
    pub fn strength(mut self, strength: f64) -> Self {
        self.strength = strength;
        self
    }
    pub fn set(mut self, val: f64) -> Self {
        self.val = Some(val);
        self
    }
}
impl<'a> Drop for VariableEditable<'a> {
    fn drop(&mut self) {
        let edit_var = EditVariable::new(&self);
        self.builder.edit_vars.push(edit_var);
    }
}
#[derive(Debug)]
pub struct EditVariable {
    var: Variable,
    val: Option<f64>,
    strength: f64,
}
impl EditVariable {
    fn new(editable: &VariableEditable) -> Self {
        EditVariable {
            var: editable.var,
            val: editable.val,
            strength: editable.strength,
        }
    }
}

pub fn change_strength(constraints: &Vec<Constraint>, strength: f64) -> Vec<Constraint> {
    let mut new_constraints = Vec::new();
    for cons in constraints {
        let cons = Constraint::new(cons.0.expression.clone(), cons.0.op, strength);
        new_constraints.push(cons);
    }
    new_constraints
}

#[macro_export]
macro_rules! layout {
    ($widget:ident: $($func:expr),*) => {
        layout!($widget: $($func, )*);
    };
    ($widget:ident: $($func:expr,)*) => {
        {
            $(
                $widget.layout().add($func);
            )*
        }
    };
}

lazy_static! {
    pub static ref LAYOUT: LayoutVars = LayoutVars::new();
}

pub mod solver;
pub mod constraint;
pub mod linear_layout;

pub use self::solver::LimnSolver;

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use cassowary::strength::*;
    use super::solver;

    use super::{LimnSolver, LayoutId, Layout, LayoutVars, VarUpdate, LayoutRef};
    use super::constraint::*;
    use super::{Size, Point, Rect};

    #[test]
    fn one_widget() {
        let mut layout = TestLayout::new();

        let mut widget = layout.new_widget("widget");
        layout!(widget:
            top_left(Point::new(0.0, 0.0)),
            dimensions(Size::new(200.0, 200.0)),
        );
        layout.add_widget(&mut widget);

        layout.update();
        assert!(layout.layout == hashmap!{
            widget.id => Rect::new(Point::new(0.0, 0.0), Size::new(200.0, 200.0)),
        });
    }
    #[test]
    fn grid() {
        let mut layout = TestLayout::new();

        let mut widget_o = layout.new_widget("widget_o");
        let mut widget_tl = layout.new_widget("widget_tl");
        let mut widget_bl = layout.new_widget("widget_bl");
        let mut widget_tr = layout.new_widget("widget_tr");
        let mut widget_br = layout.new_widget("widget_br");
        layout!(widget_o:
            top_left(Point::new(0.0, 0.0)),
            dimensions(Size::new(300.0, 300.0)),
        );
        layout!(widget_tl:
            align_top(&widget_o),
            align_left(&widget_o),
        );
        layout!(widget_tr:
            to_right_of(&widget_tl),
            align_top(&widget_o),
            align_right(&widget_o),
            match_width(&widget_tl),
        );
        layout!(widget_bl:
            below(&widget_tl),
            align_bottom(&widget_o),
            align_left(&widget_o),
            match_width(&widget_tl),
            match_height(&widget_tl),
        );
        layout!(widget_br:
            below(&widget_tr),
            to_right_of(&widget_bl),
            align_bottom(&widget_o),
            align_right(&widget_o),
            match_width(&widget_bl),
            match_height(&widget_tr),
        );
        layout.add_widget(&mut widget_o);
        layout.add_widget(&mut widget_tl);
        layout.add_widget(&mut widget_tr);
        layout.add_widget(&mut widget_bl);
        layout.add_widget(&mut widget_br);

        layout.update();
        assert!(layout.layout == hashmap!{
            widget_o.id => Rect::new(Point::new(0.0, 0.0), Size::new(300.0, 300.0)),
            widget_tl.id => Rect::new(Point::new(0.0, 0.0), Size::new(150.0, 150.0)),
            widget_tr.id => Rect::new(Point::new(150.0, 0.0), Size::new(150.0, 150.0)),
            widget_bl.id => Rect::new(Point::new(0.0, 150.0), Size::new(150.0, 150.0)),
            widget_br.id => Rect::new(Point::new(150.0, 150.0), Size::new(150.0, 150.0)),
        });
    }
    #[test]
    fn edit_var() {
        let mut layout = TestLayout::new();

        let mut root_widget = layout.new_widget("root");
        let mut slider = layout.new_widget("slider");
        let mut slider_bar_pre = layout.new_widget("slider_bar_pre");
        let mut slider_handle = layout.new_widget("slider_handle");
        layout!(root_widget:
            top_left(Point::new(0.0, 0.0)),
        );
        root_widget.layout.edit_right().set(100.0).strength(STRONG);
        root_widget.layout.edit_bottom().set(100.0).strength(STRONG);
        layout!(slider:
            align_left(&root_widget).padding(50.0),
        );
        layout!(slider_bar_pre:
            to_left_of(&slider_handle),
        );

        layout!(slider_handle:
            bound_by(&slider),
        );
        layout!(slider_bar_pre:
            bound_by(&slider),
        );
        layout!(slider:
            bound_by(&root_widget),
        );
        let slider_handle_left = slider_handle.layout().vars.left;

        layout.add_widget(&mut root_widget);
        layout.update();

        layout.add_widget(&mut slider);
        layout.add_widget(&mut slider_bar_pre);
        layout.add_widget(&mut slider_handle);

        //slider_handle.layout().edit_left().set(50.0);
        //layout.solver.update_from_builder(slider_handle.layout);
        layout.solver.solver.add_edit_variable(slider_handle_left, STRONG).unwrap();
        layout.solver.solver.suggest_value(slider_handle_left, 50.0).unwrap();

        //solver.debug_variables();
        //solver.debug_constraints();
        layout.update();
        //assert!(layout.layout[&slider.id].size.width == 50.0);
        /*assert!(layout == hashmap!{
            root_widget.id => Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0)),
            slider.id => Rect::new(Point::new(50.0, 0.0), Size::new(0.0, 0.0)),
            slider_bar_pre.id => Rect::new(Point::new(50.0, 0.0), Size::new(0.0, 0.0)),
            slider_handle.id => Rect::new(Point::new(50.0, 0.0), Size::new(0.0, 0.0)),
        });*/
    }

    // code below is used to create a test harness for creating layouts outside of the widget graph
    struct TestWidget {
        id: LayoutId,
        layout: Layout,
    }
    impl TestWidget {
        fn layout(&mut self) -> &mut Layout {
            &mut self.layout
        }
    }
    struct TestLayout {
        id_gen: IdGen,
        solver: LimnSolver,
        widget_map: HashMap<LayoutId, LayoutVars>,
        widget_names: HashMap<LayoutId, String>,
        layout: HashMap<LayoutId, Rect>,
    }
    impl TestLayout {
        fn new() -> Self {
            TestLayout {
                id_gen: IdGen::new(),
                solver: LimnSolver::new(),
                widget_map: HashMap::new(),
                widget_names: HashMap::new(),
                layout: HashMap::new(),
            }
        }
        fn new_widget(&mut self, name: &str) -> TestWidget {
            let layout_builder = Layout::new();
            let id = self.id_gen.next();
            self.widget_map.insert(id, layout_builder.vars.clone());
            self.widget_names.insert(id, name.to_owned());
            TestWidget {
                id: id,
                layout: layout_builder,
            }
        }
        fn add_widget(&mut self, widget: &mut TestWidget) {
            use std::mem;
            let name = self.widget_names.get(&widget.id).unwrap().clone();
            let layout_builder = mem::replace(&mut widget.layout, Layout::new());
            self.solver.add_widget(widget.id, &Some(name), layout_builder);
        }
        fn update(&mut self) {
            use solver;
            for (widget_id, var, value) in self.solver.fetch_changes() {
                let rect = self.layout.entry(widget_id).or_insert(Rect::zero());
                let vars = self.widget_map.get(&widget_id).unwrap();
                println!("{} = {}", solver::fmt_variable(var), value);
                let var = vars.get_var(var).unwrap();
                match var {
                    VarUpdate::Left => rect.origin.x = value,
                    VarUpdate::Top => rect.origin.y = value,
                    VarUpdate::Width => rect.size.width = value,
                    VarUpdate::Height => rect.size.height = value,
                }
            }
        }
    }
    impl LayoutRef for TestWidget {
        fn layout_ref(&self) -> &LayoutVars {
            &self.layout.vars
        }
    }
    struct IdGen {
        id: usize,
    }
    impl IdGen {
        fn new() -> Self {
            IdGen {
                id: 0,
            }
        }
        fn next(&mut self) -> LayoutId {
            let next = self.id;
            self.id += 1;
            next
        }
    }
}