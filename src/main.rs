extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;
extern crate bson;
extern crate mongodb;


use bson::{to_bson, Bson, Document};
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use serde_json::Error;
use std::iter;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Ramen {
    pub name: String,
    pub address: String,
    pub ramen_types: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Shop {
    pub ramen: Vec<Ramen>
}

struct ShopIterator<'b> {
    pub cursor: usize,
    pub inner: &'b Shop,
}

impl Shop {
    pub fn iter(&self) -> ShopIterator {
        ShopIterator {
            inner: self,
            cursor: 0,
        }
    }
}

impl<'b> iter::Iterator for ShopIterator<'b> {
    type Item = &'b Ramen;
  
    fn next(&mut self) -> Option<Self::Item> {
        let cursor = self.cursor;
        self.cursor += 1;
    
        if cursor >= self.inner.ramen.len() {
            None
        } else {
            Some(&self.inner.ramen[cursor])
        }
      }
}

impl<'b> iter::IntoIterator for &'b Shop {
    type Item = &'b Ramen;
    type IntoIter = ShopIterator<'b>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            cursor: 0,
            inner: self,
        }
    }
}

fn main() {
    let client = Client::connect("localhost", 27017)
        .expect("Could not initialize client");

    let ramen_shops = parse_json().unwrap();
    for (city, shops) in &ramen_shops {
        let coll = client.db("ramen").collection(city);
        let mut doc = Document::new();

        for shop in shops {
            let name = to_bson(&shop.name).unwrap();
            let address = to_bson(&shop.address).unwrap();
            let ramen_types = to_bson(&shop.ramen_types).unwrap();

            doc.insert_bson(String::from("name"), name);
            doc.insert_bson(String::from("address"), address);
            doc.insert_bson(String::from("ramen_types"), ramen_types);
        }

        println!("doc: {}", doc);

        coll.insert_one(doc.clone(), None)
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
}

fn parse_json() -> Result<HashMap<String, Shop>, Error> {
    let mut file = File::open("./ramen.json")
        .expect("File not found.");

    let mut json = String::new();
    file.read_to_string(&mut json)
        .expect("Could not read file.");

    let r: HashMap<String, Shop> = serde_json::from_str(&json)?;

    Ok(r)
}
