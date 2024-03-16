use std::{
    process::exit,
    io::{
        BufRead,
        BufReader,
    }
};
use select::document::Document;

mod iters;
mod dataset;

use iters::*;
use dataset::*;

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum Book {
    Algebra,
    English
}

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
        format!("{GDZ_URL}/class-{cl}/{su}", cl = $class.trim().to_lowercase(), su = $subj.trim().to_lowercase())
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

fn get_degree_subj<R: std::io::Read>(rbuf: &mut BufReader<R>) -> String {
    let mut degree = String::new();
    let mut subj   = String::new();

    println!("Enter a degree");
    read_buf!(rbuf => degree);
    let parsed_degree = degree
            .trim()
            .parse::<usize>()
            .expect(&format!("Failed to convert {degree} to usize"));
    if parsed_degree < 1 || parsed_degree > 11 {
        println!("haha, funny..");
        exit(1);
    }
    println!("Enter a subject");
    read_buf!(rbuf => subj);
    get_books_from_class!(degree, subj)
}

fn main() -> Result<(), minreq::Error> {
    let mut rbuf = BufReader::new(std::io::stdin().lock());

    let url = get_degree_subj(&mut rbuf);
    println!("URL: {url}");
    let books_resp = minreq::get(url).send()?;

    if books_resp.status_code == 200 {
        match books_resp.as_str() {
            Ok(body) => {
                let doc = Document::from(body);

                let mut books_ds = DataSet::<Book, &str, &str>::new();
                books_ds.collect(book_iter, &doc, Book::Algebra);

                let new_books = Vec::new();
                let books = books_ds.get_books(Book::Algebra).unwrap_or(&new_books);

                println!("Books: {books:#?}");
                println!("Which one is yours? from 0 to {books_len}", books_len = books_ds
                         .sizes
                         .get(&Book::Algebra)
                         .unwrap_or(&0));

                let mut user_choice = String::new();
                read_buf!(rbuf => user_choice);

                let parsed_choice = user_choice
                    .trim()
                    .parse::<usize>()
                    .expect(&format!("Failed to convert {user_choice} to usize"));
                let url = format!("{GDZ_URL}{url}", url = books_ds
                                .buckets
                                .get(&Book::Algebra)
                                .unwrap_or(&Vec::new())[parsed_choice].1);
                println!("You chose: {url}");
                unimplemented!("I'm currenty rewriting the app. fully. I decided to change all of the architecture, look on the bright side")
            }
            Err(err) => eprintln!("FAILED to convert response to string: {err}")
        }
    } else {
        eprintln!("ERROR status code: {status}", status = books_resp.status_code)
    }

    Ok(())
}
