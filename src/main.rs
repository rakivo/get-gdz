use std::{
    str::FromStr,
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
pub enum Subject {
    Algebra,
    English
    // ...
}

impl FromStr for Subject {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "algebra" => Ok(Subject::Algebra),
            "english" => Ok(Subject::English),
            _         => Err(format!("Failed to convert '{s}' to Subject variant"))
        }
    }
}

const GDZ_URL: &str = "https://gdz.ru";

macro_rules! TEST__ {
    ($DOC: ident, $rbuf: ident, $books_ds: ident, $nos_ds: ident, $imgs_ds: ident, $book: expr, $start: expr) =>
    {
        {
            let url = ask_and_get_book(&$DOC, &mut $rbuf, &mut $books_ds, &$book);

            match get_document(&url) {
                Ok($DOC) => {
                    let url = ask_and_get_no
                    (
                        &$DOC, &mut $rbuf, &mut $nos_ds, &$book
                    ).map_err(|err| eprintln!("ERROR ASKING AND GETTING NO.: {err}"))
                     .ok()
                     .unwrap_or_default();

                    match get_document(&url) {
                        Ok($DOC) => {
                            $imgs_ds
                                .collect_imgs(img_iter, &$DOC, $start)
                                .map_err(|err| eprintln!("ERROR GETTING AND SAVING IMAGES: {err}"))
                                .ok();
                        }
                        Err(err) => eprintln!("Failed to get document: {err}")
                    }
                }
                Err(err) => eprintln!("Failed to get document: {err}")
            }
        }
    };
}

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

macro_rules! parse_choice {
    ($choice: expr) => {
        $choice.trim()
            .parse::<usize>()
            .expect(&format!("Failed to convert {choice} to usize", choice = $choice))
    };
    (m $($choice: expr), *) => {
        let choices = vec![$($choice), *];
        choices.iter_mut()
            .map(|x|
                 x.trim()
                 .parse::<usize>()
                 .expect(&format!("Failed to convert {choice} to usize", choice = $choice)))
    };
}

fn get_document(url: &str) -> Result<Document, minreq::Error> {
    let books_resp = minreq::get(url).send()?;
    if books_resp.status_code == 200 {
        Ok(Document::from(books_resp.as_str()?))
    } else {
        eprintln!("ERROR, STATUS CODE: {code}", code = books_resp.status_code);
        Err(minreq::Error::AddressNotFound)
    }
}

fn ask_and_get_degree_subj<R>(rbuf: &mut BufReader<R>) -> Result<(String, Subject), String>
where R: std::io::Read
{
    let mut degree = String::new();
    let mut subj   = String::new();

    println!("Enter a degree");
    read_buf!(rbuf => degree);
    let parsed_degree = parse_choice!(degree);
    if parsed_degree < 1 || parsed_degree > 11 {
        println!("haha, funny..");
        exit(1);
    }
    println!("Enter a subject");
    read_buf!(rbuf => subj);
    match subj.trim().parse::<Subject>() {
        Ok(subj_) => {
            Ok((get_books_from_class!(degree, subj), subj_))
        }
        Err(err) => Err(err)
    }
}

fn ask_and_get_book<'a, R>
(
    doc:      &'a Document,
    rbuf:     &'a mut BufReader<R>,
    books_ds: &'a mut DataSet::<Subject, &'a str, &'a str>,
    book:     &'a Subject
) -> String
where R: std::io::Read
{
    books_ds.collect(book_iter, doc, book);

    let _new_books = Vec::new();
    let books = books_ds.get_from_bucket(book).unwrap_or(&_new_books);
    let books_len = books_ds.sizes.get(book).unwrap_or(&0);

    println!("Subjects: {books:#?}");
    println!("Which one is yours? from 0 to {books_len}", );

    let mut user_choice = String::new();
    read_buf!(rbuf => user_choice);

    let parsed_choice = parse_choice!(user_choice);
    let url = books_ds.buckets.get(book).unwrap_or(&Vec::new())[parsed_choice - 1].1;
    let ret = format!("{GDZ_URL}{url}");
    println!("You chose: {ret}");
    ret
}

fn ask_and_get_no<'a, R: std::io::Read>
(
    doc:    &'a Document,
    rbuf:   &'a mut BufReader<R>,
    nos_ds: &'a mut DataSet::<Subject, usize, &'a str>,
    book:   &'a Subject
) -> Result<String, String> {
    nos_ds.collect(no_iter, &doc, book);

    let nos_len = nos_ds.sizes.get(&book).unwrap_or(&0);
    println!("Now select no., from 0 to {nos_len}");

    let mut user_choice = String::new();
    read_buf!(rbuf => user_choice);
    let parsed_choice = parse_choice!(user_choice);
    if &parsed_choice >= nos_len {
        return Err("You can't even manage yourself to select a no. within the given range. I'm sorry but I can not help you with that.".to_owned())
    }

    let url = nos_ds.buckets.get(book).unwrap_or(&Vec::new())[parsed_choice - 1].1;
    let ret = format!("{GDZ_URL}{url}");
    println!("You chose: {ret}, see solutions for this problem in current directory");
    Ok(ret)
}

fn main() -> Result<(), minreq::Error> {
    let mut rbuf = BufReader::new(std::io::stdin().lock());

    let (url, book) = match ask_and_get_degree_subj(&mut rbuf) {
        Ok((u, b)) => (u, b),
        Err(err)   => {
            eprintln!("ERROR PARSING SUBJECT TO ENUM: {err}");
            exit(1);
        }
    };
    println!("URL: {url}");

    let mut books_ds = DataSet::<Subject, &str,  &str>::new();
    let mut nos_ds   = DataSet::<Subject, usize, &str>::new();
    let     imgs_ds  = DataSet::<Subject, &str,  &str>::new();

/* TODO:
    Retrieve elements from buckets, use keys from a hashmap instead of indexing an array.
    Array indexing can fail sometimes, preventing you from obtaining the exact book or none at all.
*/
    match get_document(&url) {
        Ok(doc)  => {
            TEST__!(doc, rbuf, books_ds, nos_ds, imgs_ds, book, 1);
        }
        Err(err) => eprintln!("Failed to get document: {err}")
    }

    unimplemented!("I'm currenty rewriting the app. fully. I decided to change all of the architecture, look on the bright side")
}
