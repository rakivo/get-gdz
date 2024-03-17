use std::{
    fs::File,
    io::Write,
    hash::Hash,
    fmt::Display,
    collections::HashMap
};

#[allow(unused)]
macro_rules! pubstructLT {
    ($name: ident<$lt: lifetime, $($T: tt), *> {
       $($field: ident: $t:ty,) *
    }) => {
        pub struct $name<$lt, $($T), *> {
            $(pub $field: $t), *
        }
    }
}

macro_rules! pubstructT {
    ($name: ident<$($T: tt),*> {
       $($field: ident: $t:ty,) *
    }) => {
        pub struct $name<$($T),*> {
            $(pub $field: $t), *
        }
    }
}

/*
buckets:
    U is a bucket, most often this is just enums, like Book, Degree, Subject & etc.
    in bucket we have vector of (most often) titles, like name of book, no. of task,
    and (most often) hrefs to the item.

sizes:
    U is a bucket as well, just like in "buckets", in here we can
    have a length of the (title, href) vector of the given bucket.
*/

pubstructT!(
    DataSet<U, T, H> {
        buckets: HashMap<U, Vec<(T, H)>>,
        sizes: HashMap<U, usize>,
    }
);

impl<U, T, H> DataSet<U, T, H>
where U: Eq + Hash,
      T: Display,
      H: Display
{
    pub fn new() -> DataSet<U, T, H> {
        DataSet {
            buckets: HashMap::new(),
            sizes: HashMap::new(),
        }
    }

    pub fn collect<'a, F, D, I>
    (
        &mut self,
        iterf: F,
        arg: &'a D,
        buck: U
    )
    where I: Iterator<Item=(T, H)> + 'a,
          F: Fn(&'a D) -> I,
          U: Clone,
    {
        let mut bucket = Vec::new();
        let size = self.sizes.entry(buck.clone()).or_insert(0);
        for (t, h) in iterf(arg) {
            println!("Inserting: title: {t}, href: {h}");
            bucket.push((t, h));
            *size += 1;
        }
        self.buckets.entry(buck).or_insert(bucket);
    }

    // this is also definitely not the best way of doing that.
    pub fn collect_imgs<'a, F, D, I>
    (
        &mut self,
        iterf: F,
        arg: &'a D,
        mut start: usize
    ) -> Result<(), minreq::Error>
    where I: Iterator<Item=(T, H)> + 'a,
          F: Fn(&'a D) -> I,
    {
        for (img_src, img_alt) in iterf(arg) {
            let img_resp = minreq::get(format!("https:{img_src}")).send()?;
            if img_resp.status_code == 200 {
                let img_data = img_resp.into_bytes();
                let file_name = format!("image{start}.jpg");

                println!("Saving: {img_src} with alt: {img_alt} to {file_name}");
                let mut file = File::create(file_name).expect("ERROR: Failed to create file");
                file.write_all(&img_data)
                    .map_err(|err| eprintln!("ERROR WRITING TO THE FILE: {err}"))
                    .ok();
            } else {
                println!("ERROR: Failed to fetch image: {status}", status = img_resp.status_code);
            }
            start += 1;
        }
        Ok(())
    }

    #[inline]
    pub fn get_books(&self, buck: U) -> Option<&Vec<(T, H)>> {
        self.buckets.get(&buck)
    }
}
