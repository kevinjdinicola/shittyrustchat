use std::fmt::Display;
use std::sync::Mutex;
use askama::Template;
use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse, middleware, HttpRequest};
use actix_web::cookie::Cookie;
use actix_web::middleware::Logger;
use actix_web::web::Form;
use serde::Deserialize;


#[derive(Template)]
#[template(path = "page.html", escape = "none")]
struct Page<'a> {
    title: &'a str,
    body: &'a dyn Display,
}

#[derive(Template)]
#[template(path = "start.html")]
struct Start {}

#[derive(Template)]
#[template(path = "chat.html")]
struct ChatTemplate<'a> {
    msgs: Vec<ChatMessageTemplate<'a>>
}

#[derive(Template)]
#[template(path = "chat-container.html")]
struct ChatContainerTemplate<'a> {
    msgs: Vec<ChatMessageTemplate<'a>>
}

struct ChatMessageTemplate<'a> {
    me: bool,
    user: &'a str,
    text: &'a str,
}

#[get("/")]
async fn start() -> impl Responder {
    let start_page = Start {};

    wrap_in_page("Start", &start_page)
}
#[derive(Deserialize)]
struct LoginParams {
    username: String,
}

#[derive(Deserialize)]
struct ChatParams {
    chat_input: String,
}

#[post("/enter")]
async fn chat_post(data: web::Data<AppChatState>, params: Form<LoginParams>) -> impl Responder {
    let app_state = data.chats.lock().unwrap();
    let username = params.into_inner().username;

    let msgs = generate_chat_templates(&username, &app_state);

    HttpResponse::Ok()
        .cookie(Cookie::new("username", &username))
        .body(msgs.render().unwrap())
}

#[get("/polling")]
async fn polling(req: HttpRequest, data: web::Data<AppChatState>) -> impl Responder {
    let app_state = data.chats.lock().unwrap();
    let cookie = req.cookie("username").unwrap();
    let msgs = fuckkk(cookie.value(), &app_state);

    HttpResponse::Ok()
        .body(msgs.render().unwrap())
}

#[post("/send")]
async fn chat_send(req: HttpRequest, data: web::Data<AppChatState>, params: Form<ChatParams>) -> impl Responder {
    let mut app_state = data.chats.lock().unwrap();
    let cookie = req.cookie("username").unwrap();
    let user = String::from(cookie.value());
    let new_msg = ChatMessage {
        user,
        message: String::from(&params.chat_input),
    };
    app_state.push(new_msg);

    let msgs = fuckkk(cookie.value(), &app_state);

    HttpResponse::Ok()
        .body(msgs.render().unwrap())
}


#[get("/chat")]
async fn chat(data: web::Data<AppChatState>) -> impl Responder {
    let chat_messages = data.chats.lock().unwrap();
    let user = "kevin";

    let msgs = generate_chat_templates(user, &chat_messages);
    wrap_in_page("whats up", &msgs)
}

fn generate_chat_templates<'a>(current_user: &'a str, chat_messages: &'a Vec<ChatMessage>) -> ChatTemplate<'a> {
    let chat_templates: Vec<ChatMessageTemplate> = chat_messages.iter()
        .map(|d| ChatMessageTemplate { me: d.user == current_user, user: &d.user, text: &d.message} )
        .collect();

    ChatTemplate {
        msgs: chat_templates,
    }
}

fn fuckkk<'a>(current_user: &'a str, chat_messages: &'a Vec<ChatMessage>) -> ChatContainerTemplate<'a> {
    let chat_templates: Vec<ChatMessageTemplate> = chat_messages.iter()
        .map(|d| ChatMessageTemplate { me: d.user == current_user, user: &d.user, text: &d.message} )
        .collect();

    ChatContainerTemplate {
        msgs: chat_templates,
    }
}

fn wrap_in_page(title: &str, template: &dyn Display) -> HttpResponse {
    let page = Page { title, body: template};
    HttpResponse::Ok()
        .body(page.render().unwrap())
}

struct ChatMessage {
    user: String,
    message: String,
}

struct AppChatState {
    chats: Mutex<Vec<ChatMessage>>
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let app_state = web::Data::new(AppChatState {
        chats: Mutex::new(vec![ChatMessage { user: String::from("SYSTEM"), message: String::from("Welcome to the chat!") }]),
    });

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));


    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(middleware::Compress::default())
            .service(start)
            .service(chat)
            .service(chat_post)
            .service(chat_send)
            .service(polling)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}