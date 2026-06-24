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
    let Ok(files) = mods::read_layers(path) else {
        return;
    };

    // paths will be in format of patches/xml/[a].xml
    // extract [a]
    let book_name = path.strip_prefix("patches/xml/").unwrap().strip_suffix(".xml").unwrap();
    let dump = std::path::Path::new("sd:/engage_patches/").exists();

    let original_book = book.clone();

    for (idx, file) in files.into_iter().rev().enumerate() {
        let patch = Book::from_string(String::from_utf8(file).unwrap()).unwrap_or_else(|_| panic!("Could not apply patch"));
        book.patch(patch, &original_book);

        if dump {
            let new_book = book.to_string().unwrap();
            let _ = std::fs::write(
                format!("sd:/engage_patches/{book_name}#{idx}.xml"),
                prettify_xml(&new_book, book_name),
            );
        }
    }
}

pub fn string_merge(data: Array<u8>, path: &str) -> Option<String>
{
    let files = mods::read_layers(path).ok()?;

    // paths will be in format of patches/[a].xml
    // extract [a]
    let book_name = path.strip_prefix("patches/xml/").unwrap().strip_suffix(".xml").unwrap();
    let dump = std::path::Path::new("sd:/engage_patches/").exists();

    let patching = std::time::Instant::now();

    let base = std::str::from_utf8(data.as_slice()).unwrap_or_else(|_| panic!("{} XML is not properly UTF8 encoded", book_name));
    let patches: Vec<_> = files.iter()
        .map(|patch| std::str::from_utf8(patch).unwrap_or_else(|_| panic!("{} XML is not properly UTF8 encoded", book_name)))
        .collect();

    // avoid invalid leading characters like \ufeff up until <
    let base = base.trim_start_matches(|c| c != '<');

    if dump {
        let base_path = format!("sd:/engage_patches/base_{book_name}.xml");
        // write the unmodified file alongside the dumps the first time we see it
        if !std::path::Path::new(&base_path).exists() {
            let _ = std::fs::write(&base_path, base);
        }
    }

    let new_book = cobalt_xml_merge::merge_all(base, &patches);

    println!("Diffing and patching {book_name} took {}ms", patching.elapsed().as_millis());

    if dump {
        let _ = std::fs::write(format!("sd:/engage_patches/{book_name}.xml"), &new_book);
    }

    Some(new_book)
}

#[skyline::hook(offset = 0x35faa80)]
pub fn structdata_import(data: Array<u8>, path: Il2CppString, sheet: Il2CppString, method_info: OptionalMethod) {
    let data = common_patch(data, path).unwrap_or(data);

    call_original!(data, path, sheet, method_info);
}

#[skyline::hook(offset = 0x35f7870)]
pub fn structdataarray_import(
    data: Array<u8>,
    path: Il2CppString,
    sheet: Il2CppString,
    method_info: OptionalMethod,
) {
    let data = common_patch(data, path).unwrap_or(data);

    call_original!(data, path, sheet, method_info);
}

#[unity::hook("App", "Database", "Release")]
pub fn database_completed_hook(method_info: OptionalMethod) {
    println!("Database released");
    CACHE.lock().unwrap().clear();
    call_original!(method_info);
}

static CACHE: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn common_patch(data: Array<u8>, path: Il2CppString) -> Option<Array<u8>> {
    let key = path.to_string();

    if let Some(file) = CACHE.lock().unwrap().get(&key) {
        let array = Array::<u8>::of_len(file.len()).unwrap();
        array.copy_from_slice(file.as_bytes());
        return Some(array);
    }

    let file = string_merge(data, &format!("patches/xml/{key}.xml"))?;
    let array = Array::<u8>::of_len(file.len()).unwrap();
    array.copy_from_slice(file.as_bytes());

    CACHE.lock().unwrap().insert(key, file);

    Some(array)
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

    let result = std::str::from_utf8(&writer.into_inner())
        .expect("Failed to convert a slice of bytes to a string slice")
        .to_string();

    result
}
