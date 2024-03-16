use std::{
    io::{
        self,
        Write
    },
    fs::File,
    hash::Hash,
    fmt::Display,
    collections::HashMap
};

macro_rules! pubstructLT {
    ($name: ident<$lt: lifetime, $($T: tt),*> {
       $($field: ident: $t:ty,) *
    }) => {
        pub struct $name<$lt, $($T),*> {
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

pubstructT!(
    DataSet<K, V> {
        buckets: HashMap<K, Vec<(K, V)>>,
        sizes: HashMap<K, usize>,
    }
);

impl<K, V> DataSet<K, V>
where K: Eq + Hash
{
    pub fn new () -> DataSet<K, V> {
        DataSet {
            buckets: HashMap::new(),
            sizes: HashMap::new(),
        }
    }

    pub fn collect<'a, D, I>
    (
        &mut self,
        iterf: impl Fn(&'a D) -> I,
        arg: &'a D,
        buck: K
    )
    where I: Iterator<Item=(K, V)> + 'a,
          K: Clone + Display,
          V: Display
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

    // this is also in doubtful way of doing that
    pub fn collect_imgs<'a, D, I>
    (
        &mut self,
        iterf: impl Fn(&'a D) -> I,
        arg: &'a D,
        mut start: usize
    ) -> Result<(), minreq::Error>
    where I: Iterator<Item=(K, V)> + 'a,
          K: Display,
          V: Display
    {
        for (img_src, img_alt) in iterf(arg) {
            let img_resp = minreq::get(format!("https:{img_src}")).send()?;
            if img_resp.status_code == 200 {
                let img_data = img_resp.into_bytes();
                let file_name = format!("image{start}.jpg");

                println!("Saving: {img_src} with alt: {img_alt} to {file_name}");
                let mut f = File::create(file_name).expect("Failed to create file");
                f.write_all(&img_data).map_err(|err| eprintln!("ERROR WRITING TO THE FILE: {err}")).ok();
            } else {
                println!("Failed to fetch image: {status}", status = img_resp.status_code);
            }
            start += 1;
        }
        Ok(())
    }
}
