use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use maplit::hashmap;
use once_cell::sync::OnceCell;
use crate::app::entrance::Entrance;
use crate::app::namespace::Namespace;
use crate::app::program::Program;
use crate::core::callbacks::lookup::CallbackLookup;
use crate::core::callbacks::types::callback_with_user_ctx::AsyncCallbackWithUserCtx;
use crate::core::connector::connector::Connector;
use crate::parser::parser::parser::ASTParser;
use crate::prelude::{Graph};
use crate::core::result::Result;
use crate::core::error::Error;
use crate::server::test_context::TestContext;

pub struct AppCtx {
    entrance: Entrance,
    program: Program,
    parser: Box<ASTParser>,
    callbacks: CallbackLookup,
    connector: Option<Box<dyn Connector>>,
    graph: Graph,
    setup: Option<Arc<dyn AsyncCallbackWithUserCtx>>,
    ignore_callbacks: bool,
    test_context: Option<&'static TestContext>,
    static_files: HashMap<&'static str, &'static str>,
    main_namespace: Namespace,
}

impl Debug for AppCtx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("AppCtx")
    }
}

impl AppCtx {

    pub fn get() -> Result<&'static AppCtx> {
        unsafe {
            match CURRENT.get() {
                Some(ctx) => Ok({
                    let retval = ctx.lock().unwrap();
                    unsafe {
                        &*(retval.deref() as * const AppCtx)
                    }
                }),
                None => Err(Error::fatal("App ctx is accessed when app is not created.")),
            }
        }
    }

    pub(crate) fn get_mut() -> Result<&'static mut AppCtx> {
        unsafe {
            match CURRENT.get() {
                Some(ctx) => Ok({
                    let mut retval = ctx.lock().unwrap();
                    unsafe {
                        &mut *(retval.deref_mut() as * mut AppCtx)
                    }
                }),
                None => Err(Error::fatal("App ctx is mutably accessed when app is not created.")),
            }
        }
    }

    fn new() -> Self {
        Self {
            entrance: Entrance::APP,
            program: Program::Rust(env!("TEO_RUSTC_VERSION")),
            callbacks: CallbackLookup::new(),
            graph: Graph::new(),
            setup: None,
            ignore_callbacks: false,
            test_context: None,
            static_files: hashmap!{},
            main_namespace: Namespace::main(),
            parser: Box::new(ASTParser::new()),
            connector: None,
        }
    }

    fn reset(&mut self) {
        self.parser = Box::new(ASTParser::new());
        self.connector = None;
        self.static_files = hashmap!{};
        self.main_namespace = Namespace::new();
    }

    pub(in crate::app) fn create() -> bool {
        if CURRENT.get().is_none() {
            CURRENT.set(Arc::new(Mutex::new(Self::new()))).unwrap();
            true
        } else {
            false
        }
    }

    pub(in crate::app) fn drop() -> Result<()> {
        Ok(Self::get_mut()?.reset())
    }

    pub(crate) fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn set_entrance(&self, entrance: Entrance) -> Result<()> {
        Self::get_mut()?.entrance = entrance;
        Ok(())
    }

    pub fn set_program(&self, program: Program) -> Result<()> {
        Self::get_mut()?.program = program;
        Ok(())
    }

    pub(crate) fn callbacks(&self) -> &CallbackLookup {
        &self.callbacks
    }

    pub(crate) fn callbacks_mut(&self) -> &mut CallbackLookup {
        &mut Self::get_mut().unwrap().callbacks
    }

    pub(crate) fn parser(&self) -> &ASTParser {
        &self.parser
    }

    pub(crate) fn parser_mut(&self) -> &mut ASTParser {
        &mut AppCtx::get_mut().unwrap().parser.as_mut()
    }

    pub(crate) fn main_namespace(&self) -> &Namespace {
        &self.main_namespace
    }

    pub(crate) fn main_namespace_mut(&self) -> &mut Namespace {
        &mut AppCtx::get_mut().unwrap().main_namespace
    }

    pub(crate) fn set_setup(&self, setup: Arc<dyn AsyncCallbackWithUserCtx>) {
        AppCtx::get_mut().unwrap().setup = Some(setup);
    }

    pub(crate) fn setup(&self) -> Option<Arc<dyn AsyncCallbackWithUserCtx>> {
        self.setup.clone()
    }

    pub(crate) fn set_connector(&self, connector: Box<dyn Connector>) {
        AppCtx::get_mut().unwrap().connector = Some(connector);
    }

    pub(crate) fn connector(&self) -> Result<&dyn Connector> {
        match &self.connector {
            Some(connector) => Ok(connector.as_ref()),
            None => Err(Error::fatal("Connector is accessed while it's not set."))
        }
    }

    pub(crate) fn program(&self) -> &Program {
        &self.program
    }

    pub(crate) fn entrance(&self) -> &Entrance {
        &self.entrance
    }

    pub(crate) fn set_ignore_callbacks(&self, value: bool) {
        AppCtx::get_mut().unwrap().ignore_callbacks = value;
    }

    pub(crate) fn ignore_callbacks(&self) -> bool {
        self.ignore_callbacks
    }

    pub(crate) fn set_test_context(&self, test_context: Option<&'static TestContext>) -> Result<()> {
        AppCtx::get_mut()?.test_context = test_context;
        Ok(())
    }

    pub(crate) fn test_context(&self) -> Option<&'static TestContext> {
        self.test_context
    }

    pub(crate) fn insert_static_files(&self, path: &'static str, map: &'static str) -> Result<()> {
        AppCtx::get_mut()?.static_files.insert(path, map);
        Ok(())
    }

    pub(crate) fn static_files(&self) -> &HashMap<&'static str, &'static str> {
        &self.static_files
    }
}

static CURRENT: OnceCell<Arc<Mutex<AppCtx>>> = OnceCell::new();