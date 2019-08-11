//! Template management and rendering

use std::env;
use std::sync::{Arc, RwLock};
use actix_web::{HttpResponse, Responder};
use hotwatch::{Event, Hotwatch};
use log::{debug, error, trace};
use tera::{Context, Tera};


/// A collection of templates loaded from disk
///
/// `Templates` is a custom wrapper around `tera::Tera` that provides the following additional
/// features:
///
/// * Automatic reloading when templates change on disk
/// * Templates are rendered to `impl actix_web::Responder` instead of simply `String`
pub struct Templates(Arc<RwLock<Tera>>);

impl Templates
{
    /// Returns a Hotwatch callback that reloads this template collection
    fn reloader(&self) -> impl Fn(Event) + Send + 'static
    {
        let templates = self.clone();

        move |event|
        {
            match event
            {
                Event::Create(_) | Event::Write(_) | Event::Remove(_) | Event::Rename(_, _) =>
                {
                    debug!("reloading templates");
                    let mut templates = rwl_write!(templates.0);
                    if let Err(err) = templates.full_reload() {
                        error!("Failed to reload templates: {}", err);
                    }
                },

                Event::Error(err, _) =>
                {
                    error!("Error while watching template directory: {}", err);
                },

                _ =>
                {
                    trace!("ignoring hotwatch event {:?}", event);
                },
            }
        }
    }

    /// Loads templates from disk
    ///
    /// Templates are loaded from the location specified by the LC_TEMPLATES environment variable.
    /// `hotwatch` is configured to watch and automatically reload templates.
    pub fn load(hotwatch: &mut Hotwatch) -> Self
    {
        let path = env::var("LC_TEMPLATES")
            .expect("Templates::load: could not read LC_TEMPLATES");

        let templates = Tera::new(&format!("{}/**/*", path))
            .expect("Templates::load: failed to load templates");
        let templates = Templates(Arc::new(RwLock::new(templates)));

        hotwatch.watch(&path, templates.reloader())
            .expect("Templates::load: failed to watch template path");

        templates
    }

    /// Renders the specified template
    pub fn render(&self, name: &str, context: Context) -> impl Responder
    {
        let body = self.0.read()
            .expect("Templates::render: failed to get read lock on templates")
            .render(name, context)
            .expect("Templates::render: failed to render template");

        HttpResponse::Ok()
            .content_type("text/html")
            .body(body)
    }
}

impl Clone for Templates
{
    fn clone(&self) -> Self
    {
        Templates(self.0.clone())
    }
}
