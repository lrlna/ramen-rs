extern crate serde;
extern crate serde_json;

#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;

#[macro_use]
extern crate serde_derive;

use bson::Bson;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use serde_json::Error;
use std::env;
use std::fs::File;
use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
struct RamenShop {
    name: String,
    address: String,
    ramen_Types: Vec<String>,
}

fn main() {
    let client = Client::connect("localhost", 27017)
        .expect("Coul dbnot initialize client");

    let coll = client.db("ramen").collection("berlin");

    let doc = parse_json();

    coll.insert_one(doc, None)
        .ok().expect("Could not insert document");

    let mut cursor = coll.find(Some(doc), None)
        .ok().expect("Could not find document");

    let item = cursor.next();

    match item {
        Some(Ok(doc)) => match doc.get("name") {
            Some(&Bson::String(ref name)) => println!("{}", name),
            _ => panic! ("Expected name to be a string"),
        },
        Some(Err(_)) => panic!("Could not get next cursor"),
        None => panic!("There are no results"),
    }
}

fn parse_json() -> RamenShop<(), Error> {
    let mut file = File::open("./ramen.json")
        .expect("File not found.");

    let mut json = String::new();
    file.read_to_string(&mut json)
        .expect("Could not read file.");

    println!("json file: {}", json);

    let r: RamenShop = serde_json::from_str(&json)?;

    return r;
}
