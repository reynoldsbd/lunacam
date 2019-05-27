//! HTML template management


// TODO: error handling


//#region Usings

use std::sync::{Arc, RwLock};

use actix_web::{HttpRequest, HttpResponse};

use hotwatch::{Event, Hotwatch};

use log::{debug, error, trace};

use tera::{compile_templates, Context, Tera};

//#endregion


/// Reloads templates when underlying files change
fn reload(templates: Arc<RwLock<Tera>>) -> impl Fn(Event) + Send + 'static
{
    move |event| {
        match event {
            Event::Create(_) | Event::Write(_) | Event::Remove(_) | Event::Rename(_, _) => {
                debug!("reloading templates");
                rwl_write!(templates)
                    .full_reload()
                    .unwrap_or_else(|err| {
                        error!("Failed to reload templates: {}", err);
                    });
            },
            Event::Error(err, _) => {
                error!("Error watching templates: {}", err)
            },
            _ => {
                trace!("ignoring hotwatch event {:?}", event)
            },
        }
    }
}


/// Represents a collection of HTML templates
pub struct Templates
{
    templates: Arc<RwLock<Tera>>,
    watcher: Arc<Hotwatch>,
}

impl Templates
{
    /// Loads a new template collection from `path`
    ///
    /// `path` is interpreted as a directory, and all templates under `path` are parsed and made
    /// available for rendering. Changes to any of the templates will cause the collection ot be
    /// reloaded.
    pub fn new(path: &str) -> Templates
    {
        let templates = Arc::new(RwLock::new(compile_templates!(&format!("{}/**/*", path))));

        // Setup live reloading
        let mut watcher = Hotwatch::new()
            .expect("Failed to initialize watcher");
        watcher.watch(path, reload(templates.clone()))
            .expect("Failed to watch template path");

        Templates {
            templates: templates,
            watcher: Arc::new(watcher),
        }
    }
}

impl Clone for Templates
{
    fn clone(&self) -> Self
    {
        Templates {
            templates: self.templates.clone(),
            watcher: self.watcher.clone(),
        }
    }
}

/// Renders the specified template to an `HttpResponse`
///
/// This function returns a route handler suitable for passing to Actix's `Route::f` method. The
/// callback requires your application state `S` to implement `AsRef<Templates>`.
///
/// If an error occurs while rendering the template, the error message is logged and written to the
/// returned response.
pub fn render<S>(name: &'static str) -> impl Fn(&HttpRequest<S>) -> HttpResponse + 'static
where S: AsRef<Templates>
{
    move |request| {
        debug!("rendering template {}", name);
        let templates = rwl_read!(
            request.state()
                .as_ref()
                .templates
        );
        templates.render(name, &Context::new())
            .map(|body|
                HttpResponse::Ok()
                    .content_type("text/html")
                    .body(body)
            )
            .unwrap_or_else(|err| {
                error!("Failed to render template {}: {}", name, err);
                HttpResponse::InternalServerError()
                    .body(format!("Failed to render template {}: {}", name, err))
            })
    }
}
