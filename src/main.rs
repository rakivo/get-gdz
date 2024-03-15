use std::{
    fs::File,
    io::{
        self,
        BufReader,
        BufRead
    },
    collections::HashMap
};

use reqwest::blocking::Client;
use select::{
    predicate::{
        Class,
        Name
    },
    predicate::Predicate,
    node::Node,
    document::Document
};

const GDZ_URL: &str = "https://gdz.ru";

macro_rules! get_books_from_class  {
    ($class: expr, $subj: expr) => {
        format!("{GDZ_URL}/class-{cl}/{su}", cl = $class.to_lowercase(), su = $subj.to_lowercase())
    };
}

macro_rules! read_buf {
    ($buf: ident <- $rbuf: ident) => {
        $rbuf.read_line(&mut $buf).ok();
        let $buf = $buf.trim().to_owned();
    };
}

/* TODO:
    clean code
*/

fn main() -> Result<(), reqwest::Error> {
    let client = Client::new();

    let mut rbuf  = BufReader::new(std::io::stdin().lock());
    let mut class = String::new();
    let mut subj  = String::new();

    println!("Enter the class, from 7 to 11");
    read_buf!(class <- rbuf);
    println!("Enter the subject");
    read_buf!(subj  <- rbuf);

    let url = get_books_from_class!(class, subj);
    println!("Url: {url}");

    let gdz_books_response = client.get(url).send().expect("Failed to send request");

    if gdz_books_response.status().is_success() {
        let body = gdz_books_response.text().expect("Failed to get body of response");
        let document = Document::from(body.as_str());

        let mut books = Vec::new();
        let mut books_len = 0;
        let mut book_choice = String::new();
        let mut books_map = HashMap::new();

        for (title, href) in book_iter(&document) {
            books_map.insert(title, href);
            books.push(title);
            books_len += 1;
        }
        println!("Books: {books:#?}");
        println!("Which one is yours? from 0 to {books_len}");
        read_buf!(book_choice <- rbuf);
        let parsed_choice = book_choice
            .parse::<usize>()
            .expect("Failed to convert {book_choice} to usize");
        let choosen = books
            .get(parsed_choice)
            .expect("Index out of bounds: {book_choice}");
        let url = format!("{GDZ_URL}{url}", url = books_map.get(choosen).unwrap());
        println!("You choosed: {url}");

        let gdz_tasks_response = client.get(url).send().expect("Failed to send request");
        if gdz_tasks_response.status().is_success() {
            let body = gdz_tasks_response.text().expect("Failed to get body of response");
            let document = Document::from(body.as_str());

            let mut tasks = HashMap::new();
            let mut tasks_len = 0;
            let mut task_choice = String::new();
            for (href, no) in task_iter(&document) {
                println!("href: {href}, no: {no}");
                tasks.insert(no, href);
                tasks_len += 1;
            }
            println!("Now select task, from 0 to {tasks_len}");
            read_buf!(task_choice <- rbuf);

            let parsed_choice = task_choice
                .parse::<usize>()
                .expect("Failed to convert {book_choice} to usize");

            let url = format!("{GDZ_URL}{url}", url = tasks.get(&parsed_choice).expect("No such task in here"));
            println!("You selected: {url}, see solutions for this problem in curr. dir.");

            let gdz_task_response = client.get(url.to_owned()).send().expect("Failed to send request");
            if gdz_task_response.status().is_success() {
                let body = gdz_task_response.text().expect("Failed to get body of response");
                let document = Document::from(body.as_str());

                let mut index = 0;
                for img in img_iter(&document) {
                    if let Some(image_src) = img.attr("src") {
                        let image_response = client.get(format!("https:{image_src}")).send()?;
                        if image_response.status().is_success() {
                            let image_data = image_response.bytes()?;
                            let file_name = format!("image{index}.jpg");

                            println!("Saving: {image_src} to {file_name}");
                            io::copy(&mut image_data.as_ref(),
                                     &mut File::create(file_name).expect("Failed to create file")
                            ).expect("Failed to save image");
                        } else {
                            println!("Failed to fetch image: {status}", status = image_response.status());
                        }
                    }
                    index += 1;
                }
            } else {
                println!("ERROR: {status}", status = gdz_task_response.status());
            }
        } else {
            println!("ERROR: {status}", status = gdz_tasks_response.status());
        }
    }

    Ok(())
}

fn img_iter<'a>(doc: &'a Document) -> impl Iterator<Item=Node<'a>> {
    doc
    .find(Name("div").and(Class("layout")))
    .flat_map(|l|
        l
        .find(Class("page"))
        .flat_map(move |p|
            p
            .find(Name("main")
            .and(Class("content")))
            .flat_map(move |mc|
                mc
                .find(Name("figure"))
                .flat_map(move |f|
                    f
                    .find(Class("task-img-container"))
                    .flat_map(move |tic|
                        tic
                        .find(Class("with-overtask"))
                        .flat_map(move |wo| wo.find(Name("img")))
                    )
                )
            )
        )
    )
}

fn book_iter<'a>(doc: &'a Document) -> impl Iterator<Item = (&'a str, &'a str)> {
    doc
    .find(Name("div").and(Class("layout")))
    .flat_map(|l|
        l
        .find(Class("page"))
        .flat_map(move |p|
            p.find(Name("main").and(Class("content")))
            .flat_map(move |c|
                c
                .find(Class("book__list"))
                .flat_map(move |ul|
                    ul
                    .find(Class("book__item"))
                    .flat_map(move |bi|
                        bi
                        .find(Name("a")
                        .and(Class("book__link")))
                        .filter_map(|a| Some((a.attr("title")?, a.attr("href")?)))
                    )
                )
            )
        )
    )
}

fn task_iter<'a>(doc: &'a Document) -> impl Iterator<Item = (&'a str, usize)> + 'a {
    doc
    .find(Name("div").and(Class("layout")))
    .flat_map(|l|
        l
        .find(Name("div").and(Class("page")))
        .flat_map(move |p|
            p
            .find(Name("main").and(Class("content")))
            .flat_map(move |c|
                c.find(Class("task__list")
                 .and(Class("folded")))
                 .flat_map(move |tl|
                     tl
                     .find(Class("active").and(Class("section-task")))
                     .flat_map(move |s|
                        s
                        .find(Name("div"))
                        .flat_map(move |div|
                            div
                            .find(Name("a"))
                            .filter_map(|a|
                                Some((
                                    a.attr("href")?,
                                    a.attr("title")?.parse::<usize>()
                                     .expect("Failed to convert title to usize")
                                ))
                            )
                        )
                    )
                )
            )
        )
    )
}
