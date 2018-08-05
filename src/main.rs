#![feature(plugin, decl_macro)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
use rocket_contrib::Template;
use rocket::request::Form;
use rocket::response::Redirect;

extern crate philipshue;
use philipshue::bridge;
use philipshue::hue::Discovery;
use philipshue::hue::LightCommand;

use std::env;
use std::collections::HashMap;
use std::collections::BTreeMap;

#[derive(Debug, FromForm)]
struct FormLight {
    id: usize,
    state: bool,
}

#[get("/")]
fn get_lights() -> Template {
    let bridge = match connect_to_bridge() {
        Ok(bridge) => bridge,
        Err(error) => panic!(error),
    };
    let lights = match bridge.get_all_lights() {
        Ok(lights) => lights,
        Err(_) => panic!("Error while getting lights information."),
    };

    let mut context: HashMap<&str, BTreeMap<_,_>> = HashMap::new();
    context.insert("lights", lights);
    Template::render("index", &context)
}

#[post("/", data = "<light>")]
fn post_light<'r>(light: Form<'r, FormLight>) -> Redirect {
    let bridge = match connect_to_bridge() {
        Ok(bridge) => bridge,
        Err(error) => panic!(error),
    };
    let cmd = LightCommand::default();
    let cmd = match &light.get().state {
        true => cmd.on(),
        false => cmd.off(),
    };

    if let Err(_) = bridge.set_light_state(light.get().id, &cmd) {
        panic!("Error while changing light state.");
    };

    Redirect::to("/")
}

fn connect_to_bridge() -> Result<bridge::Bridge, String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Please enter a valid username.");
    }
    let bridges: Option<Vec<_>> = match bridge::discover() {
        Ok(x) => Some(x.into_iter().map(Discovery::into_ip).collect()),
        Err(_) => None
    };
    match bridges {
        Some(mut x) => Ok(bridge::Bridge::new(x.pop().unwrap(), &*args[1])),
        None => Err(String::from("Unable to reach a bridge."))
    }
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![get_lights, post_light])
        .attach(Template::fairing())
}

fn main() {
    rocket().launch();
}
