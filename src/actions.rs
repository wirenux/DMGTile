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
    ]
);

pub fn set_app_menus(cx: &mut App) {
    cx.set_menus(vec![
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
            ],
            disabled: false,
        },
        Menu {
            name: "Help".into(),
            items: vec![MenuItem::action("About", ShowAbout)],
            disabled: false,
        },
        Menu { // TODO: Remove it before Release (or add a flag)
            name: "Dev".into(),
            items: vec![
                MenuItem::action("ToastDev", ToastDev),
                MenuItem::action("ToastShiftDev", ToastShiftDev),
            ],
            disabled: false,
        },
    ]);
}
