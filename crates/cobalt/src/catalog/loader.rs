use std::collections::HashMap;

use addressables_rs::{catalog::Catalog, lookup::ExtraId};
use engage::{
    unity_engine::addressable_assets::{
        addressables::Addressables,
        addressablesimpl::{AddressablesImpl, IAddressablesImplMethods},
        resource_locators::{
            contentcatalogdata::{ContentCatalogData, IContentCatalogDataMethods},
            iresourcelocator::IResourceLocator,
        },
    },
    unity_engine::{
        jsonutility::JsonUtility,
        resource_management::resource_locations::iresourcelocation::IResourceLocation,
    },
};
use unity::{prelude::*, SystemType};

use super::synthesize::{catalog_is_valid, catalog_write_path, synthesize_mod_catalog};

const BUNDLE_DIR: &str = "Data/StreamingAssets/aa/Switch";
const LUT_PATH: &str = "sd:/engage/cobalt/cache.lut";

// All mods that ship at least one bundle, sorted by id (root path fallback) for stable order.
fn collect_bundle_mods() -> Vec<(String, &'static mods::Mod)> {
    let mut entries: Vec<(String, &'static mods::Mod)> = mods::mods()
        .filter_map(|mod_ref| {
            let has_bundles = mod_ref.entries()
                .any(|p| p.starts_with(BUNDLE_DIR) && p.extension() == Some("bundle"));
            if !has_bundles {
                return None;
            }
            let sort_key = mod_ref.id()
                .map(|s| s.to_owned())
                .unwrap_or_else(|| mod_ref.root().as_str().to_owned());
            Some((sort_key, mod_ref))
        })
        .collect();

    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

// Cheap check, no base-catalog parse: does any bundle mod have a missing or stale catalog? When
// this is false the caller skips touching the base catalog entirely (the big warm-boot win).
pub fn synthesis_needed() -> bool {
    collect_bundle_mods().iter().any(|(_, m)| !catalog_is_valid(m))
}

// Parse the base catalog ONCE and build every invalid mod catalog from it (shared across all mods,
// so no per-mod re-parse). Returns false if the parsed catalog has no entries (a bad read, or this
// load wasn't the base catalog we're waiting for), so the caller keeps waiting.
pub fn synthesize_all(base_json: &str) -> bool {
    let parse_start = std::time::Instant::now();
    let base_catalog = match Catalog::from_slice(base_json.as_bytes()) {
        Ok(c) => c,
        Err(e) => {
            println!("[synthesize] failed to parse base catalog: {}", e);
            return false;
        }
    };
    // Grab any base bundle's options blob to use as a template for the mod's own bundle entries
    // (for_bundle keeps its type identity and defaults, overriding only name/size/crc). The first
    // entry works fine. An empty catalog means this load wasn't the base, so bail and keep waiting.
    let extra = match base_catalog.get_extra(ExtraId(0)).cloned() {
        Some(e) => e,
        None => return false,
    };
    println!("[catalog] base catalog parsed once in {}ms", parse_start.elapsed().as_millis());

    let lut_start = std::time::Instant::now();
    let lut_cache: HashMap<String, String> = std::fs::read_to_string(LUT_PATH)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    println!("[catalog] LUT cache {} entries ({}ms)", lut_cache.len(), lut_start.elapsed().as_millis());

    let synth_start = std::time::Instant::now();
    let mut built = 0;
    for (_, mod_ref) in collect_bundle_mods() {
        if !catalog_is_valid(mod_ref) {
            synthesize_mod_catalog(mod_ref, &base_catalog, &extra, &lut_cache);
            built += 1;
        }
    }
    println!("[catalog] synthesis phase: {} built, {}ms", built, synth_start.elapsed().as_millis());
    true
}

// Read each mod's catalog.json, deserialize it through Unity's JsonUtility, build the locator and
// register it. Runs every boot (cold and warm). The catalogs are already on disk by now.
pub fn load_and_register() {
    let total_start = std::time::Instant::now();
    let addressables = Addressables::get_instance();

    let entries = collect_bundle_mods();

    let sys_type = match SystemType::of::<ContentCatalogData>() {
        Some(t) => t,
        None => {
            println!("[loader] failed to get ContentCatalogData type");
            return;
        }
    };

    // add_resource_locator appends and GetResourceLocations aggregates across locators, so order
    // doesn't decide precedence here. We iterate in a stable order regardless.
    for (_, mod_ref) in entries.iter().rev() {
        let path = catalog_write_path(mod_ref);

        if !std::path::Path::new(path.as_str()).exists() {
            println!("[loader] catalog missing, skipping: {}", path);
            continue;
        }

        let json = match std::fs::read_to_string(path.as_str()) {
            Ok(s) => s,
            Err(e) => {
                println!("[loader] failed to read {}: {}", path, e);
                continue;
            }
        };

        // Deserialize via Unity's JsonUtility so ContentCatalogData's internal IL2CPP fields get
        // populated correctly. Passing null as objectToOverwrite lets Unity allocate the instance.
        let null_obj = unsafe { std::mem::zeroed::<engage::system::object::Object>() };
        let deser_start = std::time::Instant::now();
        let catalog_obj = JsonUtility::from_json_internal(json.as_str(), null_obj, sys_type);
        let deser_ms = deser_start.elapsed().as_millis();

        if catalog_obj.is_null() {
            println!("[loader] deserialization returned null for: {}", path);
            continue;
        }

        let catalog = ContentCatalogData::from_il_instance(catalog_obj.as_instance());
        let loc_start = std::time::Instant::now();
        let locator = catalog.create_locator(Il2CppString::null());
        let loc_ms = loc_start.elapsed().as_millis();

        if locator.is_null() {
            println!("[loader] create_locator returned null for: {}", path);
            continue;
        }

        let ilocator = IResourceLocator::from_il_instance(locator.as_instance());
        let null_location = unsafe { std::mem::zeroed::<IResourceLocation>() };

        addressables.add_resource_locator(ilocator, Il2CppString::null(), null_location);
        println!("[catalog] registered {} ({} KB json, deser {}ms, locator {}ms)", path, json.len() / 1024, deser_ms, loc_ms);
    }

    println!("[catalog] load+register phase: {}ms ({} mods)", total_start.elapsed().as_millis(), entries.len());
}
