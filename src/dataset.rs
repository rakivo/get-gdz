use std::{
    hash::Hash,
    fmt::Display,
    fs::File,
    io,
    collections::HashMap
};
use reqwest::blocking::Client;

macro_rules! pubstructT {
    ($name: ident<$($T: tt),*> {
       $($field: ident: $t:ty,) *
    }) => {
        #[derive(Debug)]
        pub struct $name<$($T),*> {
            $(pub $field: $t), *
        }
    }
}

pubstructT!(
    DataSet<K, V> {
        array: Vec<K>,
        arr_len: usize,
        user_choice: String,
        map: HashMap<K, V>,
    }
);

/*pub struct DataSet<K, V> {
    array: Vec<K>,
    arr_len: usize,
    user_choice: String,
    map: HashMap<K, V>,
}*/

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

    pub fn collect<'a, D, I>
    (
        &mut self,
        iterf: impl Fn(&'a D) -> I,
        arg: &'a D
    )
    where I: Iterator<Item=(K, V)> + 'a,
          K: Clone,
          K: Display,
          V: Display,
    {
        for (t, h) in iterf(arg) {
            // println!("Inserting: title: {t}, href: {h}, index: {al}", al = self.arr_len);
            self.map.insert(t.clone(), h);
            self.array.push(t);
            self.arr_len += 1;
        }
    }

    pub fn collect_imgs<'a, D, I>
    (
        &mut self,
        iterf: impl Fn(&'a D) -> I,
        arg: &'a D,
        cl: &'a Client,
        mut start: usize
    ) -> Result<(), reqwest::Error>
    where I: Iterator<Item=(K, V)> + 'a,
          K: Display,
          V: Display,
    {
        for (img_src, img_alt) in iterf(arg) {
            let img_resp = cl.get(format!("https:{img_src}")).send()?;
            if img_resp.status().is_success() {
                let image_data = img_resp.bytes()?;
                let file_name = format!("image{start}.jpg");

                println!("Saving: {img_src} with alt: {img_alt} to {file_name}");
                io::copy(&mut image_data.as_ref(),
                            &mut File::create(file_name).expect("Failed to create file")
                ).expect("Failed to save image");
            } else {
                println!("Failed to fetch image: {status}", status = img_resp.status());
            }
            start += 1;
        }
        Ok(())
    }
}
