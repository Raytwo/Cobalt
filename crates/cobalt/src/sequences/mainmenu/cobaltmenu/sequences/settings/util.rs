use std::{
    fs::File,
    io::{Read, Write},
    path::Path, str::FromStr,
};
use std::fs::OpenOptions;

pub fn write_to_path(path: &str, data: &str) {
    let path = Path::new(path);
    let mut file = match OpenOptions::new().write(true).truncate(true).create(true).open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    match file.write_all(data.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", path.display(), why),
        Ok(_) => (),
    }
}

pub fn read_from_path<T: FromStr>(path: &str) -> Option<T> {
    let path = Path::new(path);
    let mut file = match File::open(&path) {
        Err(why) => {
            println!("couldn't open {}: {}", path.display(), why);
            return None
        },
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => {
            println!("couldn't read {}: {}", path.display(), why);
            return None
        },
        Ok(_) => {
            match s.parse::<T>() {
                Ok(n) => Some(n),
                Err(_) => None,
            }
        },
    }
}