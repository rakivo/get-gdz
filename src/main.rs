use std::io::{
    BufReader,
    BufRead
};

use reqwest::blocking::Client;
use select::document::Document;

mod iters;
mod dataset;

use dataset::*;
use iters::*;

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

fn main() -> Result<(), reqwest::Error> {
    let client = Client::new();

    let mut rbuf   = BufReader::new(std::io::stdin().lock());
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
        return Ok(());
    }
    println!("Enter a subject");
    read_buf!(rbuf => subj);

    let url = get_books_from_class!(degree, subj);
    // println!("URL: {url}");
    let gdz_books_response = client.get(url).send().expect("Failed to send request");

    if gdz_books_response.status().is_success() {
        let body = gdz_books_response.text().expect("Failed to get body of response");
        let document = Document::from(body.as_str());

        let mut books_ds = DataSet::<&str, &str>::new();
        books_ds.collect(book_iter, &document);

        println!("Books: {books:#?}", books = books_ds.array);
        println!("Which one is yours? from 0 to {books_len}", books_len = books_ds.arr_len);
        read_buf!(f rbuf => books_ds.user_choice);

        let parsed_choice = books_ds
            .user_choice
            .trim()
            .parse::<usize>()
            .expect(&format!("Failed to convert {book_choice} to usize", book_choice = books_ds.user_choice));
        let choosen = books_ds
            .array
            .get(parsed_choice)
            .expect("Index out of bounds: {book_choice}");
        let url = format!("{GDZ_URL}{url}", url = books_ds.map.get(choosen).expect("No such book in here"));
        println!("You chose: {url}");

        let gdz_tasks_response = client.get(url).send().expect("Failed to send request");
        if gdz_tasks_response.status().is_success() {
            let body = gdz_tasks_response.text().expect("Failed to get body of response");
            let document = Document::from(body.as_str());

            let mut tasks_ds = DataSet::<usize, &str>::new();
            tasks_ds.collect(task_iter, &document);

            println!("Now select task, from 0 to {tasks_len}", tasks_len = tasks_ds.arr_len);
            read_buf!(f rbuf => tasks_ds.user_choice);

            let parsed_choice = tasks_ds
                .user_choice
                .trim()
                .parse::<usize>()
                .expect("Failed to convert {book_choice} to usize");
            if parsed_choice >= tasks_ds.arr_len {
                println!("You can't even manage yourself to select a task within the given range. I'm sorry but I can not help you with that.");
                return Ok(());
            }

            let url = format!("{GDZ_URL}{url}", url = tasks_ds.map.get(&parsed_choice).expect("No such task in here"));
            println!("You chose: {url}, see solutions for this problem in current directory");

            let gdz_task_response = client.get(url.to_owned()).send().expect("Failed to send request");
            if gdz_task_response.status().is_success() {
                let body = gdz_task_response.text().expect("Failed to get body of response");
                let document = Document::from(body.as_str());

                let mut img_ds = DataSet::<&str, &str>::new();
                img_ds
                    .collect_imgs(img_iter, &document, &client, 0)
                    .map_err(|err| eprintln!("ERROR: {err}"))
                    .ok();
            } else {
                println!("ERROR: {status}", status = gdz_task_response.status());
            }
        } else {
            println!("ERROR: {status}", status = gdz_tasks_response.status());
        }
    } else {
        println!("ERROR: {status}", status = gdz_books_response.status());
    }

    Ok(())
}
