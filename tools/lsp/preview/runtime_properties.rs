// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

use std::{collections::HashMap, fmt::Display};

use slint_interpreter::ComponentInstance;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RuntimeComponent {
    Main,
    Global(String),
}

impl Display for RuntimeComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeComponent::Main => write!(f, "<MAIN>"),
            RuntimeComponent::Global(g) => write!(f, "{g}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeProperty {
    pub name: String,
    pub ty: i_slint_compiler::langtype::Type,
    pub visibility: i_slint_compiler::object_tree::PropertyVisibility,
}

pub fn query_runtime_properties_and_callbacks(
    component_instance: &ComponentInstance,
) -> HashMap<RuntimeComponent, Vec<RuntimeProperty>> {
    let definition = &component_instance.definition();

    let mut result = HashMap::new();

    fn collect_runtime_properties(
        it: &mut dyn Iterator<
            Item = (
                String,
                (
                    i_slint_compiler::langtype::Type,
                    i_slint_compiler::object_tree::PropertyVisibility,
                ),
            ),
        >,
    ) -> Vec<RuntimeProperty> {
        let mut v = it
            .map(|(name, (ty, visibility))| RuntimeProperty { name, ty, visibility })
            .collect::<Vec<_>>();

        v.sort_by_key(|p| p.name.clone());
        v
    }

    result.insert(
        RuntimeComponent::Main,
        collect_runtime_properties(&mut definition.properties_and_callbacks()),
    );

    for global in definition.globals() {
        result.insert(
            RuntimeComponent::Global(global.clone()),
            collect_runtime_properties(
                &mut definition
                    .global_properties_and_callbacks(&global)
                    .expect("Global was just valid"),
            ),
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        common::test::{main_test_file_name, test_file_name},
        preview::test::interpret_test_with_sources,
    };

    use std::collections::HashMap;

    #[test]
    fn test_query_runtime_properties_and_callbacks() {
        let component_instance = interpret_test_with_sources(
            "fluent",
            HashMap::from([
                (
                    main_test_file_name(),
                    String::from(
                        r#"
                    import { User2 } from "user2.slint";
                    export * from "user1.slint";

                    export global MainGlobal {
                        in property <bool> main-global-in: false;
                        out property <bool> main-global-out: false;
                        in-out property <bool> main-global-in-out: false;
                    }

                    export { User2 }

                    export component MainComponent {
                        in property <bool> main-component-in: MainGlobal.main-global-in && User2.user2-in;
                        out property <bool> main-component-out: MainGlobal.main-global-out && User2.user2-out;
                        in-out property <bool> main-component-in-out: MainGlobal.main-global-in-out && User2.user2-in-out;
                    }
                "#,
                    ),
                ),
                (
                    test_file_name("user1.slint"),
                    String::from(
                        r#"
                    export global User1 {
                        in property <bool> user1-in: false;
                        out property <bool> user1-out: false;
                        in-out property <bool> user1-in-out: false;
                    }
                "#,
                    ),
                ),
                (
                    test_file_name("user2.slint"),
                    String::from(
                        r#"
                    export global User2 {
                        in property <bool> user2-in: false;
                        out property <bool> user2-out: false;
                        in-out property <bool> user2-in-out: false;
                    }
                "#,
                    ),
                ),
            ]),
        );

        let properties = query_runtime_properties_and_callbacks(&component_instance);

        assert_eq!(properties.len(), 4);

        let main = properties.get(&RuntimeComponent::Main).unwrap();

        assert_eq!(main.len(), 3);
        assert_eq!(
            main[0],
            RuntimeProperty {
                name: "main-component-in".into(),
                ty: i_slint_compiler::langtype::Type::Bool,
                visibility: i_slint_compiler::object_tree::PropertyVisibility::Input
            }
        );
        assert_eq!(
            main[1],
            RuntimeProperty {
                name: "main-component-in-out".into(),
                ty: i_slint_compiler::langtype::Type::Bool,
                visibility: i_slint_compiler::object_tree::PropertyVisibility::InOut
            }
        );
        assert_eq!(
            main[2],
            RuntimeProperty {
                name: "main-component-out".into(),
                ty: i_slint_compiler::langtype::Type::Bool,
                visibility: i_slint_compiler::object_tree::PropertyVisibility::Output
            }
        );

        let global = properties.get(&RuntimeComponent::Global("MainGlobal".into())).unwrap();

        assert_eq!(global.len(), 3);
        assert_eq!(
            global[0],
            RuntimeProperty {
                name: "main-global-in".into(),
                ty: i_slint_compiler::langtype::Type::Bool,
                visibility: i_slint_compiler::object_tree::PropertyVisibility::Input
            }
        );
        assert_eq!(
            global[1],
            RuntimeProperty {
                name: "main-global-in-out".into(),
                ty: i_slint_compiler::langtype::Type::Bool,
                visibility: i_slint_compiler::object_tree::PropertyVisibility::InOut
            }
        );
        assert_eq!(
            global[2],
            RuntimeProperty {
                name: "main-global-out".into(),
                ty: i_slint_compiler::langtype::Type::Bool,
                visibility: i_slint_compiler::object_tree::PropertyVisibility::Output
            }
        );

        let user1 = properties.get(&RuntimeComponent::Global("User1".into())).unwrap();

        assert_eq!(user1.len(), 3);

        assert_eq!(
            user1[0],
            RuntimeProperty {
                name: "user1-in".into(),
                ty: i_slint_compiler::langtype::Type::Bool,
                visibility: i_slint_compiler::object_tree::PropertyVisibility::Input
            }
        );
        assert_eq!(
            user1[1],
            RuntimeProperty {
                name: "user1-in-out".into(),
                ty: i_slint_compiler::langtype::Type::Bool,
                visibility: i_slint_compiler::object_tree::PropertyVisibility::InOut
            }
        );
        assert_eq!(
            user1[2],
            RuntimeProperty {
                name: "user1-out".into(),
                ty: i_slint_compiler::langtype::Type::Bool,
                visibility: i_slint_compiler::object_tree::PropertyVisibility::Output
            }
        );

        let user2 = properties.get(&RuntimeComponent::Global("User2".into())).unwrap();

        assert_eq!(user2.len(), 3);
        assert_eq!(
            user2[0],
            RuntimeProperty {
                name: "user2-in".into(),
                ty: i_slint_compiler::langtype::Type::Bool,
                visibility: i_slint_compiler::object_tree::PropertyVisibility::Input
            }
        );
        assert_eq!(
            user2[1],
            RuntimeProperty {
                name: "user2-in-out".into(),
                ty: i_slint_compiler::langtype::Type::Bool,
                visibility: i_slint_compiler::object_tree::PropertyVisibility::InOut
            }
        );
        assert_eq!(
            user2[2],
            RuntimeProperty {
                name: "user2-out".into(),
                ty: i_slint_compiler::langtype::Type::Bool,
                visibility: i_slint_compiler::object_tree::PropertyVisibility::Output
            }
        );
    }
}
