use std::{
    fs::File,
    hash::Hash,
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

struct DataSet<K, V> {
    array: Vec<K>,
    arr_len: usize,
    user_choice: String,
    map: HashMap<K, V>
}

impl<K, V> DataSet<K, V>
where K: Eq + Hash
{
    pub fn new () -> DataSet<K, V> {
        DataSet {
            array: Vec::new(),
            arr_len: 0,
            user_choice: String::new(),
            map: HashMap::new()
        }
    }

    pub fn collect<'a, D, I>(&mut self, iterf: impl Fn(&'a D) -> I, arg: &'a D)
    where I: Iterator<Item=(K, V)> + 'a,
          K: Clone
    {
        for (t, h) in iterf(arg) {
            self.map.insert(t.clone(), h);
            self.array.push(t);
            self.arr_len += 1;
        }
    }

    pub fn collect_imgs<'a, I>
    (
        &mut self,
        iterf: impl Fn(&'a Document) -> I,
        arg: &'a Document,
        cl: &'a Client
    ) -> Result<(), reqwest::Error>
    where I: Iterator<Item=(K, V)> + 'a,
          K: Clone
    {
        let mut index = 0;
        for img in img_iter(arg) {
            if let Some(image_src) = img.attr("src") {
                let img_resp = cl.get(format!("https:{image_src}")).send()?;
                if img_resp.status().is_success() {
                    let image_data = img_resp.bytes()?;
                    let file_name = format!("image{index}.jpg");

                    println!("Saving: {image_src} to {file_name}");
                    io::copy(&mut image_data.as_ref(),
                             &mut File::create(file_name).expect("Failed to create file")
                    ).expect("Failed to save image");
                } else {
                    println!("Failed to fetch image: {status}", status = img_resp.status());
                }
            }
            index += 1;
        }
        Ok(())
    }
}

/*
...s -> array

..._len -> len int

...choice -> string

... map -> map

let mut books = Vec::new();
let mut books_len = 0;
let mut book_choice = String::new();
let mut books_map = HashMap::new();
*/

macro_rules! get_books_from_class  {
    ($class: expr, $subj: expr) => {
        format!("{GDZ_URL}/class-{cl}/{su}", cl = $class.to_lowercase(), su = $subj.to_lowercase())
    };
}

macro_rules! read_buf {
    ($rbuf: expr => $buf: ident) => {
        $rbuf.read_line(&mut $buf).ok();
        let $buf = $buf.trim().to_owned();
    };
    (f $rbuf: expr => $buf: ident.$field: ident) => {
        $rbuf.read_line(&mut $buf.$field).ok();
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
    read_buf!(rbuf => class);
    println!("Enter the subject");
    read_buf!(rbuf => subj);

    let url = get_books_from_class!(class, subj);
    println!("Url: {url}");

    let gdz_books_response = client.get(url).send().expect("Failed to send request");

    if gdz_books_response.status().is_success() {
        let body = gdz_books_response.text().expect("Failed to get body of response");
        let document = Document::from(body.as_str());

        /*
        ...s -> array

        ..._len -> len int

        ...choice -> string

        ... map -> map
        */

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
        let url = format!("{GDZ_URL}{url}", url = books_ds.map.get(choosen).unwrap());
        println!("You choosed: {url}");

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

            let url = format!("{GDZ_URL}{url}", url = tasks_ds.map.get(&parsed_choice).expect("No such task in here"));
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

fn book_iter<'a>(doc: &'a Document) -> impl Iterator<Item = (&'a str, &'a str)> + 'a {
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

fn task_iter<'a>(doc: &'a Document) -> impl Iterator<Item = (usize, &'a str)> + 'a {
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
                                    a.attr("title")?.parse::<usize>().unwrap(),
                                    a.attr("href")?
                                ))
                            )
                        )
                    )
                )
            )
        )
    )
}
