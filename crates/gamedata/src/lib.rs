#![feature(ptr_sub_ptr)]

use std::{
    sync::{LazyLock, Mutex},
    collections::HashMap,
};

use unity::prelude::*;

use quick_xml::{events::Event, Reader, Writer};

pub mod gamedata;
use gamedata::*;

pub fn merge<Book>(book: &mut Book, path: &str)
where
    Book: XmlPatch + astra_formats::AstraBook + Clone,
{
    let hashmap = mods::manager::Manager::get();
    let Ok(files) = hashmap.get_files(path) else {
        return;
    };

    // paths will be in format of patches/xml/[a].xml
    // extract [a]
    let book_name = path.strip_prefix("patches/xml/").unwrap().strip_suffix(".xml").unwrap();

    let original_book = book.clone();

    files.into_iter().rev().enumerate().for_each(|(idx, path)| {
        let patch = Book::from_string(String::from_utf8(path).unwrap()).expect(&format!("Could not apply patch"));
        book.patch(patch, &original_book);
        let new_book = book.to_string().unwrap();
        let _ = std::fs::write(&format!("sd:/engage_patches/{}#{idx}.xml", book_name), prettify_xml(&new_book, book_name));
    });
}

pub fn string_merge(data: &'static Il2CppArray<u8>, path: &str) -> Option<String>
{
    let hashmap = mods::manager::Manager::get();
    let files = hashmap.get_files(path).ok()?;

    // paths will be in format of patches/[a].xml
    // extract [a]
    let book_name = path.strip_prefix("patches/xml/").unwrap().strip_suffix(".xml").unwrap();

    let patching = std::time::Instant::now();

    let base = std::str::from_utf8(&data).expect(&format!("{} XML is not properly UTF8 encoded", book_name));
    let patches: Vec<_> = files.iter()
        .map(|patch| std::str::from_utf8(&patch).expect(&format!("{} XML is not properly UTF8 encoded", book_name)))
        .collect();

    // avoid invalid leading characters like \ufeff up until <
    let base = base.trim_start_matches(|c| c != '<');

    // quickly grab base file
    #[cfg(debug_assertions)]
    {
        println!("save base {book_name}");
        let path = format!("sd:/engage_patches/base_{}.xml", book_name);
        // write if it doesn't exist
        if !std::path::Path::new(&path).exists() {
            let _ = std::fs::write(&path, base);
        }
    }

    let new_book = cobalt_xml_merge::merge_all(base, &patches);

    println!("Diffing and patching {book_name} took {}ms", patching.elapsed().as_millis());

    let write_path = format!("sd:/engage_patches/{}.xml", book_name);
    let _ = std::fs::write(write_path, &new_book);

    Some(new_book)
}

#[skyline::hook(offset = 0x35faa80)]
pub fn structdata_import(data: &'static Il2CppArray<u8>, path: &'static Il2CppString, sheet: &'static Il2CppString, method_info: OptionalMethod) {
    // println!("StructData path: {}", path.get_string().unwrap());

    // let now = std::time::Instant::now();

    let data = common_patch(data, path).unwrap_or(data);

    call_original!(data, path, sheet, method_info);

    // println!("{}.xml took {}ms", path.get_string().unwrap(), now.elapsed().as_millis());
}

#[skyline::hook(offset = 0x35f7870)]
pub fn structdataarray_import(
    data: &'static Il2CppArray<u8>,
    path: &'static Il2CppString,
    sheet: &'static Il2CppString,
    method_info: OptionalMethod,
) {
    // println!("StructDataArray path: {}", path.get_string().unwrap());
    // let now = std::time::Instant::now();

    let data = common_patch(data, path).unwrap_or(data);

    call_original!(data, path, sheet, method_info);

    // println!("{}.xml took {}ms", path.get_string().unwrap(), now.elapsed().as_millis());
}

#[unity::hook("App", "Database", "Completed")]
pub fn database_completed_hook(method_info: OptionalMethod) {
    CACHE.lock().unwrap().clear();
    call_original!(method_info);
}

static CACHE: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn common_patch(data: &'static Il2CppArray<u8>, path: &Il2CppString) -> Option<&'static Il2CppArray<u8>> {
    // Check if we already have this file in the cache, instead of patching over and over
    // TODO: The cache needs to be cleared on database release, so a new hook is needed
    let mut cache = CACHE.lock().unwrap();

    match cache.get(&path.to_string()) {
        Some(file) => {
            let array = Il2CppArray::<u8>::new(file.len()).unwrap();
            array.copy_from_slice(&file.as_bytes());
            Some(array)
        },
        None => {
            // The file hasn't been patched yet

            // We ignore files that are not supported or the user doesn't have patches for, so let's check if this is a file we patched
            if let Some(file) = string_merge(data, &format!("patches/xml/{}.xml", path.to_string().as_str())) {
                // Insert it into the cache so we don't patch it again if queried later
                let _ = cache.insert(path.to_string(), file.to_owned());

                let array = Il2CppArray::<u8>::new(file.len()).unwrap();
                array.copy_from_slice(&file.as_bytes());

                Some(array)
            } else {
                None
            }
        },
    }
}

// https://gist.github.com/lwilli/14fb3178bd9adac3a64edfbc11f42e0d
pub fn prettify_xml(xml: &str, book_name: &str) -> String {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    // romfs xml files use \t for indentation
    let mut writer = Writer::new_with_indent(Vec::new(), b'\t', 1);

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Ok(event) => {
                writer.write_event(event).unwrap();
            },
            Err(e) => panic!("Error at position {}: {:?} at {book_name}.xml", reader.buffer_position(), e),
        }
    }

    let result = std::str::from_utf8(&*writer.into_inner())
        .expect("Failed to convert a slice of bytes to a string slice")
        .to_string();

    result
}
