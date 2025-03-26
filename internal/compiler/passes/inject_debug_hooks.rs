// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

//! Hooks properties for live inspection.

use crate::{expression_tree, object_tree, typeloader};

pub fn inject_debug_hooks(
    doc: &mut object_tree::Document,
    type_loader: &mut typeloader::TypeLoader,
) {
    if !type_loader.compiler_config.debug_info {
        return;
    }

    eprintln!("INJECTING DEBUG HOOKS!");

    let mut counter = 1_u64;

    doc.visit_all_used_components(|component| {
        object_tree::recurse_elem_including_sub_components(component, &(), &mut |e, &()| {
            process_element(e, counter);
            counter += 1;
        })
    });
}

fn property_id(counter: u64, name: &smol_str::SmolStr) -> smol_str::SmolStr {
    smol_str::format_smolstr!("?{counter}-{name}")
}

fn process_element(element: &object_tree::ElementRc, counter: u64) {
    let mut elem = element.borrow_mut();
    assert_eq!(elem.debug.len(), 1); // We did not merge Elements yet and we have debug info!

    elem.bindings.iter().for_each(|(name, be)| {
        let expr = std::mem::take(&mut be.borrow_mut().expression);
        be.borrow_mut().expression =
            if !matches!(&expr, expression_tree::Expression::DebugHook { .. }) {
                expression_tree::Expression::DebugHook {
                    expression: Box::new(expr),
                    id: property_id(counter, name),
                }
            } else {
                expr
            };
    });
    elem.debug.first_mut().expect("There was one element a moment ago").element_id = counter;
}

#[test]
fn test() {
    let mut compiler_config =
        crate::CompilerConfiguration::new(crate::generator::OutputFormat::Interpreter);
    compiler_config.style = Some("fluent".into());
    compiler_config.debug_info = true;

    let mut test_diags = crate::diagnostics::BuildDiagnostics::default();
    let doc_node = crate::parser::parse(
        r#"
/* ... */
struct Hello { s: string, v: float }
enum Enum { aa, bb, cc }
global G {
    pure function complicated(a: float ) -> bool { if a > 5 { return true; }; if a < 1 { return true; }; uncomplicated() }
    pure function uncomplicated( ) -> bool { false }
    out property <float> p : 3 * 2 + 15 ;
    property <string> q: "foo " + 42;
    out property <float> w : -p / 2;
    out property <Hello> out: { s: q, v: complicated(w + 15) ? -123 : p };

    in-out property <Enum> e: Enum.bb;
}

export component Foo {
    in property <int> input;
    out property<float> out1: G.w;
    out property<float> out2: G.out.v;
    out property<bool> out3: false ? input == 12 : input > 0 ? input == 11 : G.e == Enum.bb;

    if true: Rectangle {
        
    }

    for i in [1, 2, 3 , 4]: Rectangle {
        
    }
}
"#
        .into(),
        Some(std::path::Path::new("HELLO")),
        &mut test_diags,
    );
    let (doc, diag, _) =
        spin_on::spin_on(crate::compile_syntax_node(doc_node, test_diags, compiler_config));
    assert!(!diag.has_errors(), "slint compile error {:#?}", diag.to_string_vec());

    for (e, _) in doc.exports.iter() {
        eprintln!("*** Document exports: {}", e.name);
    }
    // FIXME: Do something...
    assert!(false);
}
