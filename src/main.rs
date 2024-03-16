use std::io::{
    BufReader,
    BufRead
};
use select::document::Document;

mod iters;
mod dataset;

use iters::*;
use dataset::*;

const GDZ_URL: &str = "https://gdz.ru";

/*
...s                -> array
..._len             -> len int
..._choice          -> string
map                 -> map

let mut books       = Vec::new();
let mut books_len   = 0;
let mut book_choice = String::new();
let mut books_map   = HashMap::new();
*/

macro_rules! get_books_from_class  {
    ($class: expr, $subj: expr) => {
        format!("{GDZ_URL}/class-{cl}/{su}", cl = $class.to_lowercase(), su = $subj.to_lowercase())
    };
}

macro_rules! read_buf {
    ($rbuf: expr => $buf: ident) => {
        $rbuf.read_line(&mut $buf).ok();
    };
    (f $rbuf: expr => $buf: ident.$($field: ident).*) => {
        $rbuf.read_line(&mut $buf.$($field).*).ok();
    };
}

/* TODO:
    clean code
    simplify code
    more abstractions
    more abilities
    more power
    faster
    better

    distinguish between probs. sections
    within the 'active section-task', there is an h3 tag that belongs to the category of the probs.
    you can create kinda buckets for that
*/

fn main() -> std::io::Result<()> {
    println!("I'm currenty rewriting the app. fully. I decided to change all of the architecture, look on the bright side");
    Ok(())
}
