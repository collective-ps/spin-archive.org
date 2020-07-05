use rocket::request::{FlashMessage, Form};
use rocket::response::{Flash, Redirect};
use rocket::FromForm;
use rocket_contrib::templates::tera::Context as TeraContext;
use rocket_contrib::templates::Template;

use crate::context;
use crate::database::DatabaseConnection;
use crate::models::forum;
use crate::models::post;
use crate::models::thread;
use crate::models::user::User;
use crate::services::forum_service;

#[rocket::get("/")]
pub(crate) fn index(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: Option<&User>,
) -> Result<Template, Redirect> {
    let mut context = TeraContext::new();
    context::flash_context(&mut context, flash);
    context::user_context(&mut context, user);

    let forums = forum::all(&conn);
    context.insert("forums", &forums);

    Ok(Template::render("forum/index", &context))
}

#[rocket::get("/<forum_id>")]
pub(crate) fn forum(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: Option<&User>,
    forum_id: i64,
) -> Result<Template, Redirect> {
    let mut context = TeraContext::new();

    context::flash_context(&mut context, flash);
    context::user_context(&mut context, user);

    let forum = forum::by_id(&conn, forum_id);
    let threads = thread::by_forum_id(&conn, forum_id);

    if forum.is_some() {
        let forum = forum.unwrap();
        let can_post = forum.is_open || user.map(|user| user.is_moderator()).unwrap_or(false);

        context.insert("forum", &forum);
        context.insert("threads", &threads);
        context.insert("can_post", &can_post);
        Ok(Template::render("forum/forum", &context))
    } else {
        Err(Redirect::to("/forum"))
    }
}

#[rocket::get("/<forum_id>/thread/<thread_id>")]
pub(crate) fn thread(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: Option<&User>,
    forum_id: i64,
    thread_id: i64,
) -> Result<Template, Redirect> {
    let mut context = TeraContext::new();

    context::flash_context(&mut context, flash);
    context::user_context(&mut context, user);

    let forum = forum::by_id(&conn, forum_id);
    let thread = thread::by_id(&conn, thread_id);

    if forum.is_some() && thread.is_some() {
        let posts = post::by_thread_id(&conn, thread_id);
        let thread = thread.unwrap();
        let can_post = thread.0.is_open || user.map(|user| user.is_moderator()).unwrap_or(false);

        context.insert("forum", &forum.unwrap());
        context.insert("thread", &thread.0);
        context.insert("author", &thread.1);
        context.insert("posts", &posts);
        context.insert("can_post", &can_post);

        Ok(Template::render("forum/thread", &context))
    } else {
        Err(Redirect::to("/forum"))
    }
}

#[rocket::get("/<forum_id>/new")]
pub(crate) fn new_thread(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: &User,
    forum_id: i64,
) -> Result<Template, Redirect> {
    let mut context = TeraContext::new();

    context::flash_context(&mut context, flash);
    context::user_context(&mut context, Some(user));

    let forum = forum::by_id(&conn, forum_id);

    if forum.is_some() {
        context.insert("forum", &forum.unwrap());
        Ok(Template::render("forum/new", &context))
    } else {
        Err(Redirect::to("/forum"))
    }
}

#[derive(Debug, FromForm)]
pub struct NewThread {
    title: String,
    content: String,
}

#[rocket::post("/<forum_id>/new", data = "<thread_params>")]
pub(crate) fn handle_new_thread(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: &User,
    forum_id: i64,
    thread_params: Form<NewThread>,
) -> Flash<Redirect> {
    let mut context = TeraContext::new();

    context::flash_context(&mut context, flash);
    context::user_context(&mut context, Some(user));

    let forum = forum::by_id(&conn, forum_id);
    let forum_url = format!("/forum/{forum_id}", forum_id = forum_id);

    if forum.is_some() {
        let forum = forum.unwrap();
        context.insert("forum", &forum);

        if forum.is_open || user.is_moderator() {
            match forum_service::create_thread(
                &conn,
                user.id,
                forum_id,
                &thread_params.title,
                &thread_params.content,
            ) {
                Ok((thread, _post)) => {
                    let thread_url = format!(
                        "/forum/{forum_id}/thread/{thread_id}",
                        forum_id = forum_id,
                        thread_id = thread.id
                    );

                    Flash::success(Redirect::to(thread_url), "Created a new thread.")
                }
                Err(_) => Flash::error(Redirect::to(forum_url), "Could not create thread."),
            }
        } else {
            Flash::error(Redirect::to(forum_url), "Cannot post in a locked forum.")
        }
    } else {
        Flash::error(Redirect::to(forum_url), "Invalid forum")
    }
}

#[rocket::get("/<forum_id>/thread/<thread_id>/new")]
pub(crate) fn new_post(
    conn: DatabaseConnection,
    flash: Option<FlashMessage>,
    user: &User,
    forum_id: i64,
    thread_id: i64,
) -> Result<Template, Redirect> {
    let mut context = TeraContext::new();

    context::flash_context(&mut context, flash);
    context::user_context(&mut context, Some(user));

    let forum = forum::by_id(&conn, forum_id);
    let thread = thread::by_id(&conn, thread_id);

    if forum.is_some() && thread.is_some() {
        let forum = forum.unwrap();
        let thread = thread.unwrap();

        context.insert("forum", &forum);
        context.insert("thread", &thread.0);

        Ok(Template::render("forum/new_post", &context))
    } else {
        Err(Redirect::to("/forum"))
    }
}

#[derive(Debug, FromForm)]
pub struct NewPost {
    content: String,
}

#[rocket::post("/<forum_id>/thread/<thread_id>/new", data = "<post_params>")]
pub(crate) fn handle_new_post(
    conn: DatabaseConnection,
    user: &User,
    forum_id: i64,
    thread_id: i64,
    post_params: Form<NewPost>,
) -> Flash<Redirect> {
    let forum = forum::by_id(&conn, forum_id);
    let thread = thread::by_id(&conn, thread_id);
    let forum_url = format!("/forum/{forum_id}", forum_id = forum_id);
    let thread_url = format!(
        "/forum/{forum_id}/thread/{thread_id}",
        forum_id = forum_id,
        thread_id = thread_id
    );

    if forum.is_some() && thread.is_some() {
        let thread = thread.unwrap().0;

        if thread.is_open || user.is_moderator() {
            let new_post = post::NewPost {
                content: &post_params.content,
                thread_id,
                author_id: user.id,
            };

            match post::insert(&conn, &new_post) {
                Ok(_) => Flash::success(Redirect::to(thread_url), "Created a new post."),
                Err(_) => Flash::error(Redirect::to(forum_url), "Could not create new post."),
            }
        } else {
            Flash::error(Redirect::to(forum_url), "Cannot post in a locked thread.")
        }
    } else {
        Flash::error(Redirect::to(forum_url), "Could not create new post.")
    }
}

pub(crate) fn router() -> Vec<rocket::Route> {
    rocket::routes![
        index,
        forum,
        thread,
        new_thread,
        handle_new_thread,
        new_post,
        handle_new_post
    ]
}
