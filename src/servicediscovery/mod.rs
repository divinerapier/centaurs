#[cfg(feature = "nacos-servicediscovery")]
pub mod nacos;

use std::sync::Mutex;

pub struct Instance {
    pub ip: String,
    pub port: u32,
    pub service_name: String,
}

impl Instance {
    pub fn new<S>(ip: S, port: u32, service_name: String) -> Option<Instance>
    where
        S: Into<Option<String>>,
    {
        Some(Instance {
            ip: ip.into()?,
            port,
            service_name,
        })
    }
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

pub type Factory = fn() -> Option<Box<dyn Registry>>;

lazy_static::lazy_static! {
    static ref FACTORIES: Mutex<Vec<Factory>> =  Mutex::new(Vec::new());
}

pub fn register_factory(f: Factory) {
    let mut factories = FACTORIES.lock().unwrap();
    factories.push(f);
}

pub fn init() -> Option<Discovery> {
    let registries = FACTORIES.lock().unwrap();
    for factory in registries.iter() {
        if let Some(registry) = factory() {
            return Some(Discovery { registry });
        }
    }
    None
}
