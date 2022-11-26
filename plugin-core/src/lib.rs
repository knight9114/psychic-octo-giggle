use std::rc::Rc;
use std::collections::HashMap;
use std::ffi::OsStr;
use libloading::Library;


pub static CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub struct DataBuffer {
    pub data: f64,
}

pub trait PluginLoader {
    fn load_factory_from_bytes<'a>(&self, bytes: &'a [u8]) -> Box<dyn PluginFactory<'a> + 'a>;
}


pub trait PluginFactory<'a> {
    fn new_worker(&'a self) -> Box<dyn PluginWorker + 'a>;
}


pub trait PluginWorker: std::fmt::Display {
    fn consume_inputs(&mut self, buffer: DataBuffer) -> Result<(), String>;
}


pub struct PluginDeclaration {
    pub rustc_version: &'static str,
    pub core_version: &'static str,
    pub register: unsafe extern "C" fn(&mut dyn PluginRegistrar),
}


 pub trait PluginRegistrar {
    fn register_loader(&mut self, name: &str, loader: Box<dyn PluginLoader>);
 }


 #[macro_export]
 macro_rules! export_plugin {
     ( $register:expr ) => {
         #[doc(hidden)]
         #[no_mangle]
         pub static plugin_declaration: $crate::PluginDeclaration = $crate::PluginDeclaration {
            rustc_version: $crate::RUSTC_VERSION,
            core_version: $crate::CORE_VERSION,
            register: $register,
         };
     };
 }


pub struct ExternalPluginProxy {
    loader: Box<dyn PluginLoader>,
    _lib: Rc<Library>,
}

impl PluginLoader for ExternalPluginProxy {
    fn load_factory_from_bytes<'a>(&self, bytes: &'a [u8]) -> Box<dyn PluginFactory<'a> + 'a> {
        self.loader.load_factory_from_bytes(bytes)
    }
}


pub struct ExternalPluginRegistrar {
    loaders: Vec<ExternalPluginProxy>,
    mapping: HashMap<String, usize>,
    lib: Rc<Library>,
}

impl ExternalPluginRegistrar {
    pub fn new(lib: Rc<Library>) -> Self {
        Self { loaders: Default::default(), mapping: Default::default(), lib }
    }
}

impl PluginRegistrar for ExternalPluginRegistrar {
    fn register_loader(&mut self, name: &str, loader: Box<dyn PluginLoader>) {
        let proxy = ExternalPluginProxy {
            loader,
            _lib: Rc::clone(&self.lib),
        };
        let n = self.loaders.len();

        self.loaders.push(proxy);
        self.mapping.insert(name.to_string(), n);
    }
}


#[allow(dead_code)]
#[derive(Default)]
pub struct ExternalPlugins {
    pub loaders: Vec<ExternalPluginProxy>,
    pub mapping: HashMap<String, usize>,
    pub libraries: Vec<Rc<Library>>,
}

impl ExternalPlugins {
    pub unsafe fn load<P: AsRef<OsStr>>(&mut self, libpath: P) -> Result<(), String> {
        let library = Rc::new(
            Library::new(libpath)
                .map_err(|e| format!("{}",e ))?
        );

        let declaration = library
            .get::<*mut PluginDeclaration>(b"plugin_declaration\0")
            .map_err(|e| format!("{}",e ))?
            .read();

        if declaration.rustc_version != RUSTC_VERSION {
            return Err(
                format!(
                    "incompatible `rustc` version: got {}, expected {}",
                    declaration.rustc_version,
                    RUSTC_VERSION,
                ),
            )
        }
        if declaration.core_version != CORE_VERSION {
            return Err(
                format!(
                    "incompatible `core` version: got {}, expected {}",
                    declaration.core_version,
                    CORE_VERSION,
                ),
            )
        }

        let mut registrar = ExternalPluginRegistrar::new(Rc::clone(&library));
        (declaration.register)(&mut registrar);

        self.loaders.extend(registrar.loaders);
        self.libraries.push(library);

        Ok(())
    }
}