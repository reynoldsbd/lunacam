use std::env;
use std::sync::{Arc, RwLock};
use actix_files::{Files};
use actix_web::{App, HttpResponse, HttpServer, Responder};
use actix_web::web::{self, Data};
use env_logger::Env;
use hotwatch::{Event, Hotwatch};
use log::{error, trace};
use serde::{Serialize};
use tera::{Context, Tera};

//#region Templates

struct Templates(Arc<RwLock<Tera>>);

impl Templates
{
    fn make_reloader(&self) -> impl Fn(Event) + Send + 'static
    {
        let templates = self.clone();

        move |event| {

            match event {

                Event::Create(_) | Event::Write(_) | Event::Remove(_) | Event::Rename(_, _) => {
                    let mut templates = templates.0.write()
                        .expect("Templates::make_reloader: failed to get write lock on templates");
                    if let Err(err) = templates.full_reload() {
                        error!("Failed to reload templates: {}", err);
                    }
                },

                Event::Error(err, _) => {
                    error!("Error while watching template directory: {}", err);
                },

                _ => {
                    trace!("ignoring hotwatch event {:?}", event);
                },
            }
        }
    }

    fn load(hotwatch: &mut Hotwatch) -> Self
    {
        let mut path = env::current_dir()
            .expect("Templates::load: failed to get current directory");
        path.push("templates");

        let pattern = format!("{}/**/*", path.display());
        let templates = Tera::new(&pattern)
            .expect("Templates::load: failed to load templates");
        let templates = Templates(Arc::new(RwLock::new(templates)));

        hotwatch.watch(&path, templates.make_reloader())
            .expect("Templates::load: failed to watch template path");

        templates
    }

    fn render(&self, name: &str, context: Context) -> impl Responder
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

//#endregion

//#region Models

#[derive(Serialize)]
struct Camera
{
    name: String,
    id: String,
}

//#endregion

fn index(templates: Data<Templates>) -> impl Responder
{
    templates.render("index.html", Context::new())
}

fn admin(templates: Data<Templates>) -> impl Responder
{
    let mut context = Context::new();

    // TODO: get from db
    context.insert("cameras", &[
        Camera {
            name: "Living Room".into(),
            id: "living-room".into(),
        },
        Camera {
            name: "Bedroom".into(),
            id: "bedroom".into(),
        },
    ]);

    templates.render("admin.html", context)
}

fn main()
{
    let env = Env::default()
        .filter_or("LC_LOG", "info")
        .write_style("LC_LOG_STYLE");
    env_logger::init_from_env(env);

    let mut hotwatch = Hotwatch::new()
        .expect("main: failed to initialize Hotwatch");
    let templates = Data::new(Templates::load(&mut hotwatch));

    HttpServer::new(move || {
            App::new()
                .register_data(templates.clone())
                .service(Files::new("/static/js", "./js"))
                .service(Files::new("/static", "./static"))
                .route("/", web::get().to(index))
                .route("/admin/", web::get().to(admin))
        })
        .bind("127.0.0.1:8000").unwrap()
        .run().unwrap()
}
