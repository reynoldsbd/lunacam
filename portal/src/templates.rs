//! Template management and rendering

use std::env;
use std::sync::{Arc, Mutex, RwLock};
use actix_web::{HttpResponse};
use hotwatch::{Event, Hotwatch};
use lazy_static::lazy_static;
use lunacam::{do_lock, do_read, do_write, Result};
use log::{debug, error, trace};
use tera::{Context, Tera};


/// A collection of templates ready to be rendered
pub trait TemplateCollection {

    /// Renders the specified template to a `String`
    fn render(&self, name: &str, context: Context) -> Result<String>;

    /// Renders the specified template to an `HttpResponse`
    fn response(&self, name: &str, context: Context) -> Result<HttpResponse> {

        let body = self.render(name, context)?;

        let response = HttpResponse::Ok()
            .content_type("text/html")
            .body(body);

        Ok(response)
    }
}


/// Template collection that dynamically reloads templates
struct WatchedTemplateCollection(RwLock<Tera>);

impl WatchedTemplateCollection {

    /// Creates a Hotwatch callback that reloads this collection.
    fn reloader(self: Arc<Self>) -> impl Fn(Event) + Send + 'static {

        move |event| {
            match event {

                Event::Create(_) | Event::Write(_) | Event::Remove(_) | Event::Rename(_, _) => {
                    debug!("reloading templates");
                    let mut templates = do_write!(self.0);
                    if let Err(err) = templates.full_reload() {
                        error!("failed to reload templates: {}", err);
                    }
                },

                Event::Error(err, _) => {
                    error!("error while watching template directory: {}", err);
                },

                _ => {
                    trace!("ignoring hotwatch event {:?}", event);
                },
            }
        }
    }

    /// Loads a new `WatchedTemplateCollection` from the specified directory. Templates are watched
    /// for changes and automatically reloaded using `hotwatch`.
    fn new(dir: &str, hotwatch: &mut Hotwatch) -> Result<Arc<WatchedTemplateCollection>> {

        let tera = Tera::new(&format!("{}/**/*", dir))?;
        let collection = Arc::new(WatchedTemplateCollection(RwLock::new(tera)));

        let reloader = collection.clone()
            .reloader();
        hotwatch.watch(dir, reloader)?;

        Ok(collection)
    }
}

impl TemplateCollection for WatchedTemplateCollection {

    fn render(&self, name: &str, context: Context) -> Result<String> {

        let templates = do_read!(self.0);

        Ok(templates.render(name, context)?)
    }
}


lazy_static! {
    /// Shared `Hotwatch` used by all instances of `WatchedTemplateCollection`
    ///
    /// Using a static/shared instance has a number of benefits:
    ///
    /// 1. Clients don't need to know about Hotwatch
    /// 2. Easy to ensure `Hotwatch` instance does not drop
    /// 3. Multiple template collections share a single watcher thread
    ///
    /// Note that the watcher thread owns an `Arc` to each template collection, meaning the only
    /// way to completely drop collections is to drop the `Hotwatch` itself.
    static ref HOTWATCH: Mutex<Option<Hotwatch>> = Mutex::new(None);
}


/// Loads templates from disk
///
/// Templates are loaded from the directory given by the LC_TEMPLATES environment variable and
/// automatically reloaded when changes are detected.
pub fn load() -> Result<Arc<impl TemplateCollection>> {

    // Get or create the static Hotwatch instance
    let mut hotwatch = do_lock!(HOTWATCH);
    if hotwatch.is_none() {
        hotwatch.replace(Hotwatch::new()?);
    }

    let dir = env::var("LC_TEMPLATES")?;

    WatchedTemplateCollection::new(&dir, hotwatch.as_mut().unwrap())
}
