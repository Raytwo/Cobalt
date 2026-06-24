pub mod api;
pub mod classes;
pub mod hooks;
pub mod registry;

use unity::il2cpp::MethodInfo;
use unity::injection::InjectedClass;
use unity::{Class, ClassIdentity};

pub use unity::injection::{instantiate, instantiate_with_ctor};

pub fn register<T: InjectedClass>() -> Class {
    let class = unity::injection::build::<T>();

    let parent_ctor: Option<&'static MethodInfo> = T::Parent::class()
        .raw()
        .get_method_from_name(".ctor", 0)
        .map(|mi| &*mi);

    registry::insert(T::NAMESPACE, T::NAME, class, parent_ctor);
    T::fill_cache(class);
    class
}

pub fn register_all() {
    classes::register_all();
    // the il2cpp_class_from_name forwarder reads the registry directly for
    // plugins and cobalt, this hook covers the game's own lookups
    skyline::install_hook!(hooks::mono_class_from_assembly_namespace_name_hook);
}
