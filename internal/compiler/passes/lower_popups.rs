// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

//! Passe that transform the PopupWindow element into a component

use crate::diagnostics::BuildDiagnostics;
use crate::expression_tree::{Expression, NamedReference};
use crate::langtype::{ElementType, Type};
use crate::object_tree::*;
use crate::typeregister::TypeRegister;
use std::cell::RefCell;
use std::rc::Rc;

pub fn lower_popups(
    component: &Rc<Component>,
    type_register: &TypeRegister,
    diag: &mut BuildDiagnostics,
) {
    let window_type = type_register.lookup_builtin_element("Window").unwrap();

    recurse_elem_including_sub_components_no_borrow(
        component,
        &None,
        &mut |elem, parent_element: &Option<ElementRc>| {
            let is_popup = match &elem.borrow().base_type {
                ElementType::Builtin(base_type) => base_type.name == "PopupWindow",
                ElementType::Component(base_type) => base_type.inherits_popup_window.get(),
                _ => false,
            };

            if is_popup {
                lower_popup_window(elem, parent_element.as_ref(), &window_type, diag);
            }
            Some(elem.clone())
        },
    )
}

fn lower_popup_window(
    popup_window_element: &ElementRc,
    parent_element: Option<&ElementRc>,
    window_type: &ElementType,
    diag: &mut BuildDiagnostics,
) {
    let parent_component = popup_window_element.borrow().enclosing_component.upgrade().unwrap();
    let parent_element = match parent_element {
        None => {
            if matches!(popup_window_element.borrow().base_type, ElementType::Builtin(_)) {
                popup_window_element.borrow_mut().base_type = window_type.clone();
            }
            parent_component.inherits_popup_window.set(true);
            return;
        }
        Some(parent_element) => {
            if crate::layout::is_layout(&parent_element.borrow().base_type) {
                diag.push_warning(
                    "PopupWindow shouldn't be a children of a layout".into(),
                    &*popup_window_element.borrow(),
                )
            }
            parent_element
        }
    };

    if Rc::ptr_eq(&parent_component.root_element, popup_window_element) {
        diag.push_error(
            "PopupWindow cannot be directly repeated or conditional".into(),
            &*popup_window_element.borrow(),
        );
        return;
    }

    // Remove the popup_window_element from its parent
    let old_size = parent_element.borrow().children.len();
    parent_element.borrow_mut().children.retain(|child| !Rc::ptr_eq(child, popup_window_element));
    debug_assert_eq!(
        parent_element.borrow().children.len() + 1,
        old_size,
        "Exactly one child must be removed (the popup itself)"
    );
    parent_element.borrow_mut().has_popup_child = true;

    if matches!(popup_window_element.borrow().base_type, ElementType::Builtin(_)) {
        popup_window_element.borrow_mut().base_type = window_type.clone();
    }

    const CLOSE_ON_CLICK: &str = "close-on-click";
    let close_on_click = popup_window_element.borrow_mut().bindings.remove(CLOSE_ON_CLICK);
    let close_on_click = close_on_click
        .map(|b| {
            let b = b.into_inner();
            (b.expression, b.span)
        })
        .or_else(|| {
            let mut base = popup_window_element.borrow().base_type.clone();
            while let ElementType::Component(b) = base {
                base = b.root_element.borrow().base_type.clone();
                if let Some(binding) = b.root_element.borrow().bindings.get(CLOSE_ON_CLICK) {
                    let b = binding.borrow();
                    return Some((b.expression.clone(), b.span.clone()));
                }
            }
            None
        });

    let close_on_click = match close_on_click {
        Some((expr, location)) => match expr {
            Expression::BoolLiteral(value) => value,
            _ => {
                diag.push_error(
                    "The close-on-click property only supports constants at the moment".into(),
                    &location,
                );
                return;
            }
        },
        None => true,
    };

    let popup_comp = Rc::new(Component {
        root_element: popup_window_element.clone(),
        parent_element: Rc::downgrade(parent_element),
        ..Component::default()
    });

    let weak = Rc::downgrade(&popup_comp);
    recurse_elem(&popup_comp.root_element, &(), &mut |e, _| {
        e.borrow_mut().enclosing_component = weak.clone()
    });

    // Generate a x and y property, relative to the window coordinate
    // FIXME: this is a hack that doesn't always work, perhaps should we store an item ref or something
    let coord_x = create_coordinate(&popup_comp, parent_element, "x");
    let coord_y = create_coordinate(&popup_comp, parent_element, "y");

    // Throw error when accessing the popup from outside
    // FIXME:
    // - the span is the span of the PopupWindow, that's wrong, we should have the span of the reference
    // - There are other object reference than in the NamedReference
    // - Maybe this should actually be allowed
    visit_all_named_references(&parent_component, &mut |nr| {
        if std::rc::Weak::ptr_eq(&nr.element().borrow().enclosing_component, &weak) {
            diag.push_error(
                "Cannot access the inside of a PopupWindow from enclosing component".into(),
                &*popup_window_element.borrow(),
            );
            // just set it to whatever is a valid NamedReference, otherwise we'll panic later
            *nr = coord_x.clone();
        }
    });

    parent_component.popup_windows.borrow_mut().push(PopupWindow {
        component: popup_comp,
        x: coord_x,
        y: coord_y,
        close_on_click,
        parent_element: parent_element.clone(),
    });
}

fn create_coordinate(
    popup_comp: &Rc<Component>,
    parent_element: &ElementRc,
    coord: &str,
) -> NamedReference {
    let expression = popup_comp
        .root_element
        .borrow_mut()
        .bindings
        .remove(coord)
        .map(|e| e.into_inner().expression)
        .unwrap_or(Expression::NumberLiteral(0., crate::expression_tree::Unit::Phx));
    let property_name = format!("{}-popup-{}", popup_comp.root_element.borrow().id, coord);
    parent_element
        .borrow_mut()
        .property_declarations
        .insert(property_name.clone(), Type::LogicalLength.into());
    parent_element
        .borrow_mut()
        .bindings
        .insert(property_name.clone(), RefCell::new(expression.into()));
    NamedReference::new(parent_element, &property_name)
}
