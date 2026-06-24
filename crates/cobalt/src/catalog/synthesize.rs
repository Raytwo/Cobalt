use std::collections::{BTreeMap, HashMap};

use addressables_rs::{catalog::Catalog, lookup::ExtraValue};
use astra_formats::Asset;
use camino::Utf8PathBuf;
use mods::Mod;

const BUNDLE_DIR: &str = "Data/StreamingAssets/aa/Switch";

// Directory on SD for catalogs that can't live inside a read-only (zipped) mod
const COBALT_CATALOGS_DIR: &str = "sd:/engage/cobalt/catalogs";

// Bump this whenever the synthesized catalog.json layout or the data we put in it changes. A
// cached catalog whose sidecar records an older version is regenerated automatically, so we never
// have to hand-delete stale caches after a synthesizer change again.
const CATALOG_FORMAT_VERSION: u64 = 1;

// A cached catalog is valid only if its sidecar `.meta` matches the current format version and the
// mod's bundle set + mtimes are unchanged. The game is pinned to 2.0.0 (version check in main), so
// the base catalog can't change under us, no need to fingerprint it.
pub fn catalog_is_valid(mod_ref: &Mod) -> bool {
    if !std::path::Path::new(catalog_write_path(mod_ref).as_str()).exists() {
        return false;
    }

    let meta = match std::fs::read_to_string(meta_path(mod_ref).as_str()) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let meta: serde_json::Value = match serde_json::from_str(&meta) {
        Ok(v) => v,
        Err(_) => return false,
    };

    if meta.get("format_version").and_then(|v| v.as_u64()) != Some(CATALOG_FORMAT_VERSION) {
        return false;
    }

    let stored: BTreeMap<String, u64> = match meta.get("bundles").cloned().and_then(|b| serde_json::from_value(b).ok()) {
        Some(m) => m,
        None => return false,
    };

    stored == current_bundle_mtimes(mod_ref)
}

// Build this mod's catalog from the already-parsed base catalog, write it, and stamp its sidecar.
// The base is parsed once by the caller and shared across every mod (no per-mod re-parse).
pub fn synthesize_mod_catalog(mod_ref: &'static Mod, base_catalog: &Catalog, base_extra: &ExtraValue, lut_cache: &HashMap<String, String>) {
    let out_path = catalog_write_path(mod_ref);
    let start = std::time::Instant::now();

    match synthesize_for_mod(mod_ref, base_catalog, base_extra.clone(), lut_cache) {
        Ok(catalog) => match catalog.serialize() {
            Ok(json) => {
                if let Some(parent) = out_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = std::fs::write(&out_path, &json) {
                    println!("[synthesize] failed to write catalog for {}: {}", mod_display(mod_ref), e);
                } else {
                    write_meta(mod_ref);
                    println!("[synthesize] {} generated in {}ms ({} KB)", mod_display(mod_ref), start.elapsed().as_millis(), json.len() / 1024);
                }
            },
            Err(e) => println!("[synthesize] failed to serialize catalog for {}: {}", mod_display(mod_ref), e),
        },
        Err(e) => println!("[synthesize] failed to synthesize catalog for {}: {}", mod_display(mod_ref), e),
    }
}

// Sidecar next to the catalog: catalog.json -> catalog.meta
fn meta_path(mod_ref: &Mod) -> Utf8PathBuf {
    catalog_write_path(mod_ref).with_extension("meta")
}

// The fingerprint we invalidate on: each bundle's relative path -> last-modified time.
fn current_bundle_mtimes(mod_ref: &Mod) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    for p in mod_ref.entries() {
        if p.starts_with(BUNDLE_DIR) && p.extension() == Some("bundle") {
            if let Ok(mtime) = mod_ref.last_modified(&p) {
                map.insert(p.to_string(), mtime);
            }
        }
    }
    map
}

fn write_meta(mod_ref: &Mod) {
    let mut obj = serde_json::Map::new();
    obj.insert("format_version".to_owned(), serde_json::Value::from(CATALOG_FORMAT_VERSION));
    obj.insert("bundles".to_owned(), serde_json::to_value(current_bundle_mtimes(mod_ref)).unwrap_or(serde_json::Value::Null));
    if let Ok(text) = serde_json::to_string(&serde_json::Value::Object(obj)) {
        let _ = std::fs::write(meta_path(mod_ref).as_str(), text);
    }
}

fn synthesize_for_mod<'a>(
    mod_ref: &Mod,
    base_catalog: &Catalog<'a>,
    base_extra: ExtraValue,
    lut_cache: &HashMap<String, String>,
) -> Result<Catalog<'a>, Box<dyn std::error::Error>> {
    let locator_id = mod_ref.id()
        .map(|s| s.to_owned())
        .unwrap_or_else(|| mod_ref.root().as_str().to_owned());

    let mut catalog = Catalog::empty_from_base(base_catalog, locator_id);

    for rel_path in mod_ref.entries() {
        if !rel_path.starts_with(BUNDLE_DIR) || rel_path.extension() != Some("bundle") {
            continue;
        }

        let primary_key = rel_path.strip_prefix(BUNDLE_DIR)
            .map_err(|e| format!("strip_prefix failed for {}: {}", rel_path, e))?
            .to_path_buf();

        let internal_id = Utf8PathBuf::from("{UnityEngine.AddressableAssets.Addressables.RuntimePath}/Switch")
            .join(primary_key.as_str().to_lowercase());

        // Skip if this bundle already appears in the base game catalog
        if base_catalog.get_internal_id_index(&internal_id).is_some() {
            continue;
        }

        let bundle_file = match mod_ref.read(rel_path) {
            Ok(b) => b,
            Err(e) => {
                println!("[synthesize] failed to read bundle {}: {}", rel_path, e);
                continue;
            }
        };

        // Build a proper ExtraValue with this bundle's name and size instead of cloning
        // the base game's entry (which would have the wrong metadata in json_text).
        let bundle_name = primary_key.file_name().unwrap_or(primary_key.as_str());
        let entry_extra = ExtraValue::for_bundle(&base_extra, bundle_name, bundle_file.len() as u64);

        if let Err(e) = catalog.add_bundle(&internal_id, &primary_key, entry_extra) {
            println!("[synthesize] add_bundle failed for {}: {:?}", internal_id, e);
            continue;
        }

        let bundle = match astra_formats::Bundle::from_slice(&bundle_file) {
            Ok(b) => b,
            Err(e) => {
                println!("[synthesize] failed to parse bundle {}: {}", rel_path, e);
                continue;
            }
        };

        let asset = match bundle.files()
            .filter_map(|(_, file)| match file {
                astra_formats::BundleFile::Assets(a) => Some(a),
                _ => None,
            })
            .next()
        {
            Some(a) => a,
            None => {
                println!("[synthesize] no asset section in bundle {}", rel_path);
                continue;
            }
        };

        let assetbundle = match asset.assets.iter()
            .filter_map(|a| match a { Asset::Bundle(b) => Some(b), _ => None })
            .next()
        {
            Some(b) => b,
            None => {
                println!("[synthesize] no AssetBundle entry in bundle {}", rel_path);
                continue;
            }
        };

        let mut dependencies: Vec<String> = asset.externals.iter()
            .map(|ext| {
                let path = ext.path.to_string();
                if path.starts_with("archive") {
                    let cab = path[9..36 + 9].to_lowercase();
                    lut_cache.get(&cab)
                        .cloned()
                        .unwrap_or_else(|| {
                            println!("[synthesize] bundle {} has unknown dependency cab {}", rel_path, cab);
                            cab
                        })
                } else {
                    path.to_lowercase()
                }
            })
            .collect();

        dependencies.insert(0, internal_id.to_string());

        // Pull in the base-game bundles this prefab depends on. Dependency refs are entry
        // indices into THIS catalog, so a shared base bundle the asset reuses has to exist as an
        // entry here, or add_prefab drops the dependency and the asset loads incomplete (the mod
        // ships only its own bundles, the shared ones live in the base game).
        for dep in &dependencies {
            if catalog.get_internal_id_index(dep).is_some() {
                continue; // the mod's own bundle, or a base dep already copied for another prefab
            }
            match catalog.copy_bundle_from_base(base_catalog, dep) {
                Ok(true) => {},
                Ok(false) => println!("[synthesize] dependency not in base catalog, {} may load incomplete: {}", mod_display(mod_ref), dep),
                Err(e) => println!("[synthesize] failed to copy base dependency {}: {:?}", dep, e),
            }
        }

        let (container_internal_id, assetinfo) = match assetbundle.container_map.last() {
            Some(pair) => pair,
            None => {
                println!("[synthesize] empty container_map in bundle {}", rel_path);
                continue;
            }
        };

        let asset_type = if let Some(found) = asset.get_asset_by_path_id(assetinfo.asset.path_id) {
            match found {
                Asset::Texture2D(_, _) => 1,
                Asset::Sprite(_) => 2,
                Asset::Text(_) => 12,
                Asset::Terrain(_) => 13,
                Asset::AnimationClip(_) => 36,
                Asset::AnimatorOverrideController(_) => 47,
                Asset::SpriteAtlas(_) => 7,
                _ => 4,
            }
        } else {
            10
        };

        let prefixes = [
            "Assets/Share/Addressables/",
            "Assets/Project/Addressables/",
            "Assets/Share/Scenes/Map/",
            "Patch/Patch0/",
            "Patch/Patch1/",
            "Patch/Patch2/",
            "Patch/Patch3/",
        ];

        let internal_path = prefixes.iter().fold(container_internal_id.as_str(), |p, prefix| {
            p.trim_start_matches(prefix)
        });

        let suffixes = [
            ".prefab", ".png", ".asset", ".unity", ".fbx", ".overrideController", ".spriteatlas",
        ];

        let internal_path = suffixes.iter().fold(internal_path, |p, suffix| {
            p.trim_end_matches(suffix)
        });

        if let Err(e) = catalog.add_prefab(asset_type, container_internal_id.as_str().to_owned(), internal_path.to_owned(), &dependencies) {
            println!("[synthesize] add_prefab failed for {}: {:?}", container_internal_id, e);
        }
    }

    Ok(catalog)
}

// Where the synthesized catalog.json should live for this mod
pub fn catalog_write_path(mod_ref: &Mod) -> Utf8PathBuf {
    if mod_ref.is_writeable() {
        // ModDir: write next to the bundle files
        mod_ref.root().join(BUNDLE_DIR).join("catalog.json")
    } else {
        // ZippedMod: write to the shared catalogs dir, keyed by mod id or zip stem
        let key = mod_ref.id()
            .map(|s| s.to_owned())
            .unwrap_or_else(|| {
                mod_ref.root()
                    .file_stem()
                    .unwrap_or("unknown")
                    .replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_")
            });
        Utf8PathBuf::from(COBALT_CATALOGS_DIR).join(format!("{}.json", key))
    }
}

fn mod_display(mod_ref: &Mod) -> &str {
    mod_ref.id().unwrap_or_else(|| mod_ref.root().as_str())
}
