#![allow(unused_imports, unused_variables, dead_code)]


use std::sync::Arc;
use plugin_core::*;

pub struct Loader {}

impl PluginLoader for Loader {
    fn load_factory_from_bytes<'a>(&self, bytes: &'a [u8]) -> Box<dyn PluginFactory<'a> + 'a> {
        let core: Core<'a> = Core { data: bytes };
        Box::new(Factory{ core: Arc::new(core) })
    }
}


struct Core<'a> {
    data: &'a [u8],
}

pub struct Factory<'a> {
    core: Arc<Core<'a>>,
}

impl<'a> PluginFactory<'a> for Factory<'a> {
    fn new_worker(&'a self) -> Box<dyn PluginWorker + 'a> {
        Box::new(Worker { core: Arc::clone(&self.core), x_bar_previous: 0f64, m_2_previous: 0f64, n: 0f64})
    }
}


pub struct Worker<'a> {
    core: Arc<Core<'a>>,
    pub x_bar_previous: f64,
    pub m_2_previous: f64,
    pub n: f64,
}


impl<'a> std::fmt::Display for Worker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x_bar_previous, self.m_2_previous / self.n)
    }
}


impl<'a> PluginWorker for Worker<'a> {
    fn consume_inputs(&mut self, buffer: DataBuffer) -> Result<(), String> {
        let buffer = buffer;
        let x_n = buffer.data;
        let x_bar_current = self.x_bar_previous + ((x_n - self.x_bar_previous) / (self.n + 1f64));
        let m_2_current = self.m_2_previous + ((x_n - self.x_bar_previous) * (x_n - x_bar_current));
        self.x_bar_previous = x_bar_current;
        self.m_2_previous = m_2_current;
        self.n += 1f64;

        Ok(())
    }
}

plugin_core::export_plugin!(register);
#[allow(improper_ctypes_definitions)]
extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_loader("loader", Box::new(Loader{}))
}