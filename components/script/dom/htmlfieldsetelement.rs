/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding;
use crate::dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding::HTMLFieldSetElementMethods;
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::htmlcollection::{CollectionFilter, HTMLCollection};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::htmllegendelement::HTMLLegendElement;
use crate::dom::node::{window_from_node, Node, ShadowIncluding};
use crate::dom::validitystate::ValidityState;
use crate::dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use std::default::Default;
use style::element_state::ElementState;

#[dom_struct]
pub struct HTMLFieldSetElement {
    htmlelement: HTMLElement,
    form_owner: MutNullableDom<HTMLFormElement>,
}

impl HTMLFieldSetElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLFieldSetElement {
        HTMLFieldSetElement {
            htmlelement: HTMLElement::new_inherited_with_state(
                ElementState::IN_ENABLED_STATE,
                local_name,
                prefix,
                document,
            ),
            form_owner: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLFieldSetElement> {
        Node::reflect_node(
            Box::new(HTMLFieldSetElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            HTMLFieldSetElementBinding::Wrap,
        )
    }
}

impl HTMLFieldSetElementMethods for HTMLFieldSetElement {
    // https://html.spec.whatwg.org/multipage/#dom-fieldset-elements
    fn Elements(&self) -> DomRoot<HTMLCollection> {
        #[derive(JSTraceable, MallocSizeOf)]
        struct ElementsFilter;
        impl CollectionFilter for ElementsFilter {
            fn filter<'a>(&self, elem: &'a Element, _root: &'a Node) -> bool {
                elem.downcast::<HTMLElement>()
                    .map_or(false, HTMLElement::is_listed_element)
            }
        }
        let filter = Box::new(ElementsFilter);
        let window = window_from_node(self);
        HTMLCollection::create(&window, self.upcast(), filter)
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> DomRoot<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(&window, self.upcast())
    }

    // https://html.spec.whatwg.org/multipage/#dom-fieldset-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fieldset-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner()
    }
}

impl VirtualMethods for HTMLFieldSetElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("disabled") => {
                let disabled_state = match mutation {
                    AttributeMutation::Set(None) => true,
                    AttributeMutation::Set(Some(_)) => {
                        // Fieldset was already disabled before.
                        return;
                    },
                    AttributeMutation::Removed => false,
                };
                let node = self.upcast::<Node>();
                let el = self.upcast::<Element>();
                el.set_disabled_state(disabled_state);
                el.set_enabled_state(!disabled_state);
                let mut found_legend = false;
                let children = node.children().filter(|node| {
                    if found_legend {
                        true
                    } else if node.is::<HTMLLegendElement>() {
                        found_legend = true;
                        false
                    } else {
                        true
                    }
                });
                let fields = children.flat_map(|child| {
                    child
                        .traverse_preorder(ShadowIncluding::No)
                        .filter(|descendant| match descendant.type_id() {
                            NodeTypeId::Element(ElementTypeId::HTMLElement(
                                HTMLElementTypeId::HTMLButtonElement,
                            )) |
                            NodeTypeId::Element(ElementTypeId::HTMLElement(
                                HTMLElementTypeId::HTMLInputElement,
                            )) |
                            NodeTypeId::Element(ElementTypeId::HTMLElement(
                                HTMLElementTypeId::HTMLSelectElement,
                            )) |
                            NodeTypeId::Element(ElementTypeId::HTMLElement(
                                HTMLElementTypeId::HTMLTextAreaElement,
                            )) => true,
                            _ => false,
                        })
                });
                if disabled_state {
                    for field in fields {
                        let el = field.downcast::<Element>().unwrap();
                        el.set_disabled_state(true);
                        el.set_enabled_state(false);
                    }
                } else {
                    for field in fields {
                        let el = field.downcast::<Element>().unwrap();
                        el.check_disabled_attribute();
                        el.check_ancestors_disabled_state_for_form_control();
                    }
                }
            },
            &local_name!("form") => {
                self.form_attribute_mutated(mutation);
            },
            _ => {},
        }
    }
}

impl FormControl for HTMLFieldSetElement {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    fn to_element<'a>(&'a self) -> &'a Element {
        self.upcast::<Element>()
    }
}
