use serde::{Deserialize, Serialize};
use unity::prelude::*;

use std::collections::HashMap;

use addressables_rs::{
    catalog::{Catalog, CatalogError},
    lookup::ExtraId,
};

use camino::Utf8PathBuf;
use astra_formats::Asset;

use crate::api::events::{publish_system_event, SystemEvent};

#[derive(miniserde::Serialize, miniserde::Deserialize, Default)]
pub struct CachedCatalogEntry {
    internal_id: String,
    container_internal_id: String,
    primary_key: String,
    internal_path: String,
    dependencies: Vec<String>,
    asset_type: i32,
    last_modified: u64,
}

#[skyline::hook(offset = 0x2586040)]
pub fn from_json_hook(json: &Il2CppString, method_info: OptionalMethod) -> *const u8 {
    let manager = mods::manager::Manager::get();

    if let Ok(dir) = manager.get_directory("Data/StreamingAssets/aa/Switch") {

        // Lookup Table to do the glue between modded bundles and the official Catalog
        let lut_cache: HashMap<String, String> = miniserde::json::from_str(
            &std::fs::read_to_string("sd:/engage/cache.lut").expect("Could not read `sd:/engage/cache.lut`, make sure to download it on Github"),
        ).unwrap();

        // Catalog cache of the files that exist on the SD
        let mut cache: HashMap<String, CachedCatalogEntry> = if let Ok(lut) = std::fs::read_to_string("sd:/engage/catalog.lut") {
            miniserde::json::from_str(&lut).unwrap_or_default()
        } else {
            HashMap::new()
        };
    
        let mut cache_modified = false;

        // Timer for the process of parsing the catalog, appending the new entries and writing it back
        let timer = std::time::Instant::now();

        let mut catalog = Catalog::from_str(json.to_string()).unwrap();

        // Copy the ExtraId here once and for all, since there is no need for us to make our own for every file.
        let extra = catalog.get_extra(ExtraId(200)).expect("Couldn't get ExtraId").to_owned();

        // Timer for the process of processing bundles or adding them from the cache
        let bundle_timer = std::time::Instant::now();

        manager
        .get_files_in_directory_and_subdir(dir)
        .unwrap()
        .iter()
        .filter(|relative| relative.extension().unwrap() == "bundle")
        .for_each(|rel_path| {
            println!("Processing '{}'", rel_path);

            // Check if we already have this entry cached
            if let Some(entry) = cache.get(&rel_path.to_string()) {
                // Make sure this entry doesn't already exist in the Catalog, because if it does, we don't need to append
                if catalog.get_internal_id_index(&entry.internal_id).is_none() {
                    // Our cache entry is still relevant, insert the entry in the Catalog and return right away
                    if entry.last_modified == manager.get_last_modified(rel_path).unwrap() {
                        // Add the bundle to the Catalog
                        if let Err(err) = catalog.add_bundle(&entry.internal_id, &entry.primary_key, extra.clone()) {
                            print_catalog_error(entry.internal_id.as_str(), err)
                        }

                        // Add the prefab, and return on success
                        match catalog.add_prefab(entry.asset_type, entry.container_internal_id.clone(), entry.internal_path.clone(), &entry.dependencies) {
                            Ok(_) => return,
                            Err(err) => print_catalog_error(entry.container_internal_id.as_str(), err),
                        }
                    } else {
                        // The timestamp doesn't match, we'll have to scrub the entry
                        println!("Timestamp does not match, invalidating entry");
                    }
                }
            }

            println!("File not found in cache");

            let primary_key = rel_path.strip_prefix("Data/StreamingAssets/aa/Switch").unwrap().to_path_buf();

            let internal_id = Utf8PathBuf::from("{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch")
                .join(primary_key.as_str().to_lowercase());

            // println!("InternalId: {}", internal_id);
            // println!("Primary Key: {}", primary_key);

            // Check if the file already exists in the game, and ignore it if it does.
            if catalog.get_internal_id_index(&internal_id).is_none() {
                if let Err(err) = catalog.add_bundle(&internal_id, &primary_key, extra.clone()) {
                    print_catalog_error(internal_id.as_str(), err)
                }

                let bundle_file = manager
                    .get_file(&rel_path)
                    .unwrap_or_else(|err| panic!("Couldn't read bundle `{}`: {err}", rel_path));

    
                let bundle = astra_formats::Bundle::from_slice(&bundle_file)
                    .unwrap_or_else(|err| panic!("Astra-formats couldn't parse bundle `{}`: {err}", rel_path));

                // Only look for the first AssetFile, as we do not care about the raw sections 
                let asset = bundle
                    .files()
                    .filter_map(|(_, file)| match file {
                        astra_formats::BundleFile::Assets(asset) => Some(asset),
                        _ => None,
                    })
                    .next()
                    .unwrap_or_else(|| panic!("Could not find an Asset section in bundle '{rel_path}'"));

                // Find the AssetBundle asset in the file.
                let assetbundle = asset.assets
                    .iter()
                    .filter_map(|asset| {
                        match asset {
                            Asset::Bundle(bundle) => Some(bundle),
                            _ => None
                        }
                    })
                    .next()
                    // This technically can't ever happen, but modders be modders.
                    .unwrap_or_else(|| panic!("Could not find AssetBundle entry in bundle `{rel_path}`"));

                let mut dependencies: Vec<String> = asset
                    .externals
                    .iter()
                    .map(|external| {
                        // TODO: Handle this better, because Library:/ entries exist too and would crash this code.
                        let path = external.path.to_string();
                        // println!("Dependency Path: {}", path);

                        if path.starts_with("archive") {
                            // let path = path.strip_prefix("archive:/").unwrap();
                            let cab = path[9..36+9].to_lowercase();
                            lut_cache.get(&cab).unwrap_or_else(|| panic!("Bundle `{rel_path}` has a dependency that is Addressable but does not exist in the original game files.")).to_owned()
                        } else {
                            // Deal with the Library here
                            // cache.get(&path.to_lowercase()).unwrap().to_owned()
                            path.to_lowercase()
                        }
                    })
                    .collect();

                // Add the file as its own dependency, as expected for the Catalog.
                dependencies.insert(0, internal_id.to_string());
    
                // println!("Dependencies: {:#?}", dependencies);
                
                // Grab the last entry because it better represents the type we're seeking (like Texture2D -> Sprite)
                let (container_internal_id, assetinfo) = assetbundle.container_map.last().unwrap();
    
                // This will supposedly help support file addition for every type of bundle more easily than checking for specific paths.
                let asset_type = if let Some(found_asset) = asset.get_asset_by_path_id(assetinfo.asset.path_id) {
                    match found_asset {
                        Asset::Texture2D(_, _) => 1,
                        Asset::Sprite(_) => 2,
                        Asset::Text(_) => 12,
                        Asset::Terrain(_) => 13,
                        Asset::Unparsed(_) => {
                            // We handle any type that isn't parsed by Astra-formats here
                            match found_asset.type_hash() {
                                // AnimationClip
                                -80937412517696055409803870673809846754 => 36,
                                _ => 4
                            }
                        },
                        _ => 4,
                    }
                } else {
                    10
                };
    
                // println!("Prefab InternalId: {}, type: {}", internal_id, asset_type);

                let prefixes = [
                    "Assets/Share/Addressables/",
                    "Assets/Project/Addressables/",
                    "Assets/Share/Scenes/Map/",
                    "Patch/Patch0/",
                    "Patch/Patch1/",
                    "Patch/Patch2/",
                    "Patch/Patch3/",
                ];

                let internal_path = prefixes.iter().fold(container_internal_id.as_str(), |path, prefix| {
                    path.trim_start_matches(prefix)
                });

                let suffixes = [
                    ".prefab",
                    ".png",
                    ".asset",
                    ".unity",
                    ".fbx",
                ];

                let internal_path = suffixes.iter().fold(internal_path, |path, suffix| {
                    path.trim_end_matches(suffix)
                });
    
                // println!("Trimmed InternalId: {}", internal_path);
    
                if let Err(err) = catalog.add_prefab(asset_type, container_internal_id.as_str().to_owned(), internal_path.to_owned(), &dependencies) {
                    print_catalog_error(container_internal_id.as_str(), err)
                }

                // Write entry to cache

                let entry = cache.entry(rel_path.to_string()).or_insert(CachedCatalogEntry::default());
                entry.internal_id = internal_id.as_str().to_string();
                entry.container_internal_id = container_internal_id.as_str().to_string();
                entry.primary_key = primary_key.to_string();
                entry.internal_path = internal_path.to_string();
                entry.dependencies = dependencies;
                entry.asset_type = asset_type;
                entry.last_modified = manager.get_last_modified(rel_path).unwrap();

                cache_modified = true;
            } else {
                println!("File already exists in the catalog, skipping");
            }
        });
        
        println!("Processing bundles took {}ms", bundle_timer.elapsed().as_millis());

        let new_catalog = serde_json::to_string(&catalog).unwrap().into();
        println!("Catalog patching took {}ms", timer.elapsed().as_millis());

        if cache_modified {
            let out_cache = miniserde::json::to_string(&cache);
            std::fs::write("sd:/engage/catalog.lut", out_cache).unwrap();
            println!("Wrote catalog cache to SD");
        }


        let result = call_original!(new_catalog, method_info);
        publish_system_event(SystemEvent::CatalogLoaded);
        result

    } else {
        println!("No Data directory found, skipping file addition.");

        let result = call_original!(json, method_info);
        publish_system_event(SystemEvent::CatalogLoaded);
        result
    }
}

fn print_catalog_error(internal_id: &str, err: CatalogError) {
    match err {
        CatalogError::Io(io) => panic!("A file related issue happened: {}", io),
        CatalogError::Json(json) => panic!("A json related error happened: {}", json),
        CatalogError::Base64Decode(_) => panic!("Your catalog.bundle file is malformed. Delete it and restart the game."),
        CatalogError::DuplicateInternalId => panic!("InternalId `{}` already exists in catalog.bundle.\nThis can happen if you have a bundle with the same container path as an existing one.", internal_id),
        CatalogError::MissingInternalId => panic!("The following InternalId appears to be missing: {}", internal_id),
    }
}