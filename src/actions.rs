use gpui::*;

actions!(
    dmgtile,
    [
        Quit,
        NewFile,
        OpenFile,
        Save,
        Undo,
        Redo,
        Copy,
        Paste,
        Cut,
        ShowAbout,
        Eraser,
        Brush,
        Bucket,
        ShiftUp,
        ShiftDown,
        ShiftLeft,
        ShiftRight,
        FlipH,
        FlipV,
        Rotate,
        ToastDev,
        ToastShiftDev,
        EraseTile,
    ]
);

pub fn set_app_menus(cx: &mut App) {
    let mut menus = vec![
        Menu {
            name: "DMGTile".into(),
            items: vec![
                MenuItem::action("About DMGTile", ShowAbout),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ],
            disabled: false,
        },
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("New", NewFile),
                MenuItem::action("Open", OpenFile),
                MenuItem::action("Save", Save),
            ],
            disabled: false,
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::action("Undo", Undo),
                MenuItem::action("Redo", Redo),
                MenuItem::separator(),
                MenuItem::action("Erase Current Tile", EraseTile),
            ],
            disabled: false,
        },
        Menu {
            name: "Help".into(),
            items: vec![MenuItem::action("About", ShowAbout)],
            disabled: false,
        },
    ];

    if std::env::args().any(|arg| arg == "--dev") {
        menus.push(Menu {
            name: "Dev".into(),
            items: vec![
                MenuItem::action("ToastDev", ToastDev),
                MenuItem::action("ToastShiftDev", ToastShiftDev),
            ],
            disabled: false,
        });
    }

    cx.set_menus(menus);
}
