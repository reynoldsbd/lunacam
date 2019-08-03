use std::env;
use std::sync::{Arc, RwLock};
use actix_web::{App, HttpResponse, HttpServer, Responder};
use actix_web::web::{self, Data};
use hotwatch::{Event, Hotwatch};
use tera::{Context, Tera};

//#region Templates

struct Templates(Arc<RwLock<Tera>>);

impl Templates
{
    fn make_reloader(&self) -> impl Fn(Event) + Send + 'static
    {
        let templates = self.clone();

        move |event|
        {
            match event {

                Event::Create(_) | Event::Write(_) | Event::Remove(_) | Event::Rename(_, _) => {

                    let mut templates = templates.0.write()
                        .expect("Templates::make_reloader: failed to get write lock on templates");
                    templates.full_reload()
                        .expect("Templates::make_reloader: failed to reload templates");
                },

                Event::Error(err, _) => {

                    panic!("Templates::make_reloader: {}", err);
                },

                _ => {

                    println!("Templates::make_reloader: ignoring hotwatch event {:?}", event);
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

fn index(templates: Data<Templates>) -> impl Responder
{
    templates.render("index.html", Context::new())
}

fn main()
{
    let mut hotwatch = Hotwatch::new()
        .expect("main: failed to initialize Hotwatch");
    let templates = Data::new(Templates::load(&mut hotwatch));

    HttpServer::new(move || {
            App::new()
                .register_data(templates.clone())
                .route("/", web::get().to(index))
        })
        .bind("127.0.0.1:8000").unwrap()
        .run().unwrap()
}
