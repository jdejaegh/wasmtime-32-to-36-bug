use std::error::Error;
use wasmtime::{component::{Component, Linker, Resource, HasSelf}, Config, Engine, Store};

pub fn get_wasm_component<T>(wasm_file: &str) -> Result<(Engine, Component, Linker<T>), Box<dyn Error>> {
    let mut config = Config::new();
    config.wasm_component_model(true);

    let engine = Engine::new(&config).expect("Could not create engine");
    let component = Component::from_file(&engine, format!("{wasm_file}.wasm")).expect("Could not load component");
    let linker = Linker::new(&engine);

    Ok((engine, component, linker))
}


/// The `Person` struct has methods and some of them can be overridden using a WebAssembly component
/// The methods to override here are `greet` and `rename`
pub struct Person {
    name: String,
    age: u32,
}

mod person_bindings {
    wasmtime::component::bindgen!({world : "person-plugin" , path : "../wit" });
}
#[derive(Default)]
pub struct StatePerson<'a> {
    person_table: std::collections::HashMap<u32, MaybePerson<'a>>,
}

/// Enum to hold a mutable or non-mutable reference to an instance of `Person`
/// Used to provide to `&self: Person` or `&mut self: Person` to the WebAssembly component using
/// a resource
pub enum MaybePerson<'a> {
    Mut(&'a mut Person),
    NotMut(&'a Person),
}

impl Person {
    pub fn set_name(&mut self, new: String) {
        self.name = new;
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_age(&self) -> u32 {
        self.age
    }

    pub fn birthday(&mut self) {
        self.age += 1;
    }

    pub fn greet(&self) {
        let plugin = get_wasm_component("person_plugin");
        if let Ok((engine, component, mut linker)) = plugin {
            let mut data = StatePerson::default();
            let res_idx = 0_u32;
            data.person_table.insert(res_idx, MaybePerson::NotMut(self));
            let self_res = Resource::new_borrow(res_idx);

            let mut store = Store::new(&engine, data);
            person_bindings::example::plugin::imp::add_to_linker::<StatePerson<'_>, HasSelf<_>>(&mut linker, |state| state).unwrap();
            person_bindings::example::plugin::printer::add_to_linker::<StatePerson<'_>, HasSelf<_>>(&mut linker, |state| state).unwrap();

            let bindings = person_bindings::PersonPlugin::instantiate(&mut store, &component, &linker).unwrap();
            bindings.call_greet(&mut store, self_res).unwrap()
        } else { self.greet_internal() }
    }
    fn greet_internal(&self) {
        // Default code to execute if plugin fails to load
    }

    pub fn rename(&mut self) {
        let plugin = get_wasm_component("person_plugin");
        if let Ok((engine, component, mut linker)) = plugin {
            let mut data = StatePerson::default();
            let res_idx = 0_u32;
            data.person_table.insert(res_idx, MaybePerson::Mut(self));
            let self_res = Resource::new_borrow(res_idx);

            let mut store = Store::new(&engine, data);
            person_bindings::example::plugin::imp::add_to_linker::<StatePerson<'_>, HasSelf<_>>(&mut linker, |state: &mut StatePerson| state).unwrap();
            person_bindings::example::plugin::printer::add_to_linker::<StatePerson<'_>, HasSelf<_>>(&mut linker, |state: &mut StatePerson| state).unwrap();

            let bindings = person_bindings::PersonPlugin::instantiate(&mut store, &component, &linker).unwrap();
            bindings.call_rename(&mut store, self_res).unwrap()
        } else { self.rename_internal() }
    }
    fn rename_internal(&mut self) {
        // Default code to execute if plugin fails to load
    }
}

/// Implement the methods from `Person` that should be available on the resource in the WebAssembly
/// component.
/// All implementations here expect the `Person` to be mutable (hence the name `HostResPersonMut`)
impl person_bindings::example::plugin::imp::HostResPersonMut for StatePerson<'_> {
    fn set_name(&mut self, self_: Resource<person_bindings::example::plugin::imp::ResPersonMut>, new: String) -> () {
        match self.person_table.get_mut(&self_.rep()).unwrap() {
            MaybePerson::Mut(ref mut p) => p.set_name(new),
            MaybePerson::NotMut(_) => unreachable!()
        }
    }
    fn get_name(&mut self, self_: Resource<person_bindings::example::plugin::imp::ResPersonMut>) -> String {
        match self.person_table.get_mut(&self_.rep()).unwrap() {
            MaybePerson::Mut(ref mut p) => p.get_name(),
            MaybePerson::NotMut(_) => unreachable!()
        }
    }
    fn get_age(&mut self, self_: Resource<person_bindings::example::plugin::imp::ResPersonMut>) -> u32 {
        match self.person_table.get_mut(&self_.rep()).unwrap() {
            MaybePerson::Mut(ref mut p) => p.get_age(),
            MaybePerson::NotMut(_) => unreachable!()
        }
    }
    fn drop(&mut self, _rep: Resource<person_bindings::example::plugin::imp::ResPersonMut>) -> wasmtime::Result<()> {
        self.person_table.remove(&_rep.rep());
        Ok(())
    }
}

/// Implement the methods from `Person` that should be available on the resource in the WebAssembly
/// component.
/// All implementations here expect the `Person` to be non-mutable (hence the name `HostResPerson`)
impl person_bindings::example::plugin::imp::HostResPerson for StatePerson<'_> {
    fn get_name(&mut self, self_: Resource<person_bindings::example::plugin::imp::ResPerson>) -> String {
        match self.person_table.get(&self_.rep()).unwrap() {
            MaybePerson::Mut(_) => unreachable!(),
            MaybePerson::NotMut(p) => p.get_name()
        }
    }
    fn get_age(&mut self, self_: Resource<person_bindings::example::plugin::imp::ResPerson>) -> u32 {
        match self.person_table.get(&self_.rep()).unwrap() {
            MaybePerson::Mut(_) => unreachable!(),
            MaybePerson::NotMut(p) => p.get_age()
        }
    }
    fn drop(&mut self, _rep: Resource<person_bindings::example::plugin::imp::ResPerson>) -> wasmtime::Result<()> {
        self.person_table.remove(&_rep.rep());
        Ok(())
    }
}

/// Implement traits needed by the WebAssembly component
impl person_bindings::example::plugin::imp::Host for StatePerson<'_> {}

impl person_bindings::example::plugin::printer::Host for StatePerson<'_> {
    fn print(&mut self, s: String) {
        println!("{}", s);
    }
}

fn main() {
    let bob = Person { name: "Bob".to_string(), age: 30 };
    let mut alice = Person { name: "Alice".to_string(), age: 50 };

    bob.greet();
    alice.greet();

    alice.rename();
    alice.greet();
}