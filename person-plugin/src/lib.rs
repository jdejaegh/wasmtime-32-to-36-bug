#[allow(warnings)]
mod bindings;

use crate::bindings::{ResPerson, ResPersonMut};
use bindings::example::plugin::printer::print;
use bindings::Guest;

struct DemoComponent;

impl Guest for DemoComponent {
    fn greet(who: &ResPerson) {
        print(format!("Hello {} ({} yo)", who.get_name(), who.get_age()).as_str());
    }

    fn rename(who: &ResPersonMut) {
        who.set_name(format!("New {}", who.get_name()).as_str())
    }
}

bindings::export!(DemoComponent with_types_in bindings);