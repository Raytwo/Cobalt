use engage::app::basicmenu::{BasicMenu, IBasicMenuMethods};
use engage::app::basicmenuitem::BasicMenuItem;
use engage::app::proc::Proc;
use engage::app::procboolmethod::ProcBoolMethod;
use engage::app::procdesc::ProcDesc;
use engage::app::procinst::{IProcInstMethods, ProcInst};
use engage::app::procvoidmethod::ProcVoidMethod;
use engage::app::resourcemanager_2::ResourceManager_2 as ResourceManager;
use engage::app::shoptopmenucontent::ShopTopMenuContent;
use engage::system::collections::generic::list_1::{IList_1Methods as IListMethods, List_1 as List};
use unity::{Array, ClassIdentity, FromIlInstance, OptionalMethod};

use engage::{
    BasicMenuExt, BasicMenuItemExt, ProcBoolMethodExt, ProcExt, ProcVoidMethodExt, TitleBar,
};

pub mod sequences;

use sequences::{reloadmsbt::*, reloadxml::*, settings::*, update::*};

pub struct CobaltMenuSequence;

#[repr(i32)]
enum CobaltMenuSequenceLabel {
    Start = 1,
    Exit,
}

impl CobaltMenuSequence {
    fn build_procdescs(this: ProcInst) -> Array<ProcDesc> {
        let descs = [
            Proc::label(CobaltMenuSequenceLabel::Start as i32),
            Proc::call_method(
                ProcVoidMethod::from_fn(this.into(), Self::load_resources).unwrap(),
            ),
            Proc::wait_while_true_2(
                ProcBoolMethod::from_fn(this.into(), Self::is_loading_resources).unwrap(),
            ),
            Proc::call_method(
                ProcVoidMethod::from_fn(this.into(), Self::create_menu_bind).unwrap(),
            ),
            Proc::label(CobaltMenuSequenceLabel::Exit as i32),
            Proc::call_method(ProcVoidMethod::from_fn(this.into(), Self::exit).unwrap()),
            Proc::end(),
        ];

        let arr = Array::<ProcDesc>::new(<ProcDesc as ClassIdentity>::class().raw(), descs.len())
            .expect("CobaltMenuSequence: ProcDesc[] allocation failed");
        for (i, d) in descs.iter().enumerate() {
            arr.set(i, *d);
        }
        arr
    }

    pub fn bind(parent: ProcInst) {
        let proc = <ProcInst as FromIlInstance>::instantiate()
            .expect("CobaltMenuSequence: ProcInst allocation failed");
        let descs = Self::build_procdescs(proc);
        proc.create_bind(parent, descs, unity::Il2CppString::from("CobaltMenuSequence"));
    }

    pub extern "C" fn load_resources(_parent: ProcInst, _method_info: OptionalMethod) {
        ShopTopMenuContent::load_prefab_async();
    }

    pub extern "C" fn exit(_parent: ProcInst, _method_info: OptionalMethod) {
        TitleBar::close_header();
        ShopTopMenuContent::unload_prefab();
    }

    pub extern "C" fn is_loading_resources(_parent: ProcInst, _method_info: OptionalMethod) -> bool {
        ResourceManager::is_loading()
    }

    pub extern "C" fn create_menu_bind(parent: ProcInst, _method_info: OptionalMethod) {
        println!("CobaltMenuSequence::CreateMenuBind");

        let menu_content = ShopTopMenuContent::create();

        let menu_item_list = List::<BasicMenuItem>::new();
        menu_item_list.ensure_capacity(4);

        menu_item_list.add(BasicMenuItem::new_impl::<ReloadXmlMenuItem>());
        menu_item_list.add(BasicMenuItem::new_impl::<ReloadMsbtMenuItem>());
        menu_item_list.add(BasicMenuItem::new_impl::<GlobalConfigMenuItem>());

        if crate::updater::update_available() {
            menu_item_list.add(BasicMenuItem::new_impl::<UpdateAvailableMenuItem>());
        }

        let basic_menu = <BasicMenu as BasicMenuExt>::build(menu_item_list, menu_content.into());
        let descs = BasicMenuExt::create_default_desc(basic_menu);
        let proc: ProcInst = basic_menu.into();
        proc.create_bind(parent, descs, unity::Il2CppString::from("CobaltMenu"));
        basic_menu.bind_parent_menu();

        TitleBar::open_header(
            "Cobalt",
            localize::mess::get("cobalt_main_menu_title_bar_text"),
            "",
        );
    }
}
