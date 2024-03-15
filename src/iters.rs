use select::{
    predicate::{
        Class,
        Name
    },
    predicate::Predicate,
    document::Document
};

pub fn img_iter<'a>(doc: &'a Document) -> impl Iterator<Item=(&'a str, &'a str)> + 'a {
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
                        .flat_map(move |wo|
                            wo
                            .find(Name("img"))
                            .filter_map(|img| Some((img.attr("src")?, img.attr("alt")?)))
                        )
                    )
                )
            )
        )
    )
}

pub fn book_iter<'a>(doc: &'a Document) -> impl Iterator<Item=(&'a str, &'a str)> + 'a {
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

pub fn task_iter<'a>(doc: &'a Document) -> impl Iterator<Item=(usize, &'a str)> + 'a {
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
                            .filter_map(|a| {
                                let title = a.attr("title")?;
                                // println!("title: {}, href: {}", title, a.attr("href")?);
                                if let Some(f) = title.chars().nth(0) {
                                    if f.eq(&'§') {
                                    // 5520 -> paragraph -> first letter -> p -> 'P' ascii code * 69
                                        let title = 5520 + title[2..]
                                            .split_whitespace()
                                            .map(|d| d.parse::<usize>().expect("Failed to convert to usize"))
                                            .sum::<usize>();
                                        Some((
                                            title,
                                            a.attr("href")?
                                        ))
                                    }
                                    else {
                                        Some((
                                            a.attr("title")?.parse::<usize>().expect("Failed to convert to usize"),
                                            a.attr("href")?
                                        ))
                                    }
                                } else {
                                    None
                                }
                            })
                        )
                    )
                )
            )
        )
    )
}
