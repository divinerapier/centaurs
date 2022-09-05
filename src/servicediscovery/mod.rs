use std::{collections::HashMap, sync::Mutex};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("duplicated register with name: {0}")]
    DuplicatedRegister(String),
}

#[derive(derive_builder::Builder)]
pub struct Instance {
    pub ip: String,
    pub port: u32,
    pub service_name: String,
}

pub trait Registry: Send {
    fn register(&self, instance: &Instance);

    fn deregister(&self, instance: &Instance);
}

pub struct Discovery {
    registry: Box<dyn Registry>,
}

impl Discovery {
    pub fn register(self, instance: Option<Instance>) -> Option<RegisterGuard> {
        let registry = self.registry;
        let instance = instance?;
        registry.register(&instance);
        Some(RegisterGuard { registry, instance })
    }
}

pub struct RegisterGuard {
    pub(crate) registry: Box<dyn Registry>,
    pub(crate) instance: Instance,
}

impl Drop for RegisterGuard {
    fn drop(&mut self) {
        self.registry.deregister(&self.instance);
    }
}

pub type Factory = fn() -> Result<Box<dyn Registry>, Box<dyn std::error::Error>>;

lazy_static::lazy_static! {
    static ref FACTORIES: Mutex<HashMap <String,Factory>> =  Mutex::new(HashMap::new());
}

pub fn register_factory(name: String, f: Factory) -> Result<(), Error> {
    let mut factories = FACTORIES.lock().unwrap();
    factories
        .get(&name)
        .ok_or_else(|| Error::DuplicatedRegister(name.clone()))?;
    factories.insert(name, f);
    Ok(())
}

pub fn init() -> Option<Discovery> {
    let registries = FACTORIES.lock().unwrap();
    for (name, factory) in registries.iter() {
        match factory() {
            Ok(registry) => {
                tracing::info!("use register: {}", name);
                return Some(Discovery { registry });
            }
            Err(e) => {
                tracing::warn!("register {} with error {}", name, e);
            }
        }
    }
    None
}
