use askama::Template;
use actix_web::{get, web, App, HttpServer, Responder, HttpResponse, middleware};
use actix_web::middleware::Logger;
extern crate pretty_env_logger;
#[macro_use] extern crate log;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,

}

#[derive(Template)]
#[template(path = "index.html")]
struct NothingTemplate {}

#[get("/")]
async fn nothing() -> impl Responder {
    let hm = NothingTemplate {};
    HttpResponse::Ok()
        .body(hm.render().unwrap())
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    let name = name.into_inner();
    println!("{}", &name);
    let hello = HelloTemplate { name: &name };

    HttpResponse::Ok()
        .body(hello.render().unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    HttpServer::new(|| App::new()
        .wrap(Logger::default())
        .wrap(Logger::new("%a %{User-Agent}i"))
        .wrap(middleware::Compress::default())
        .service(nothing)
        .service(hello)
    )
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}