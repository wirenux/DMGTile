use gpui::*;

actions!(
    dmgtile,
    [
        Quit,
        NewFile,
        OpenFile,
        Save,
        SaveAs,
        Undo,
        Redo,
        Copy,
        Paste,
        Cut,
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
        ExportBin,
        ExportC,
        OpenGithub,
        OpenStardance,
    ]
);

pub fn set_app_menus(cx: &mut App) {
    let mut menus = vec![
        Menu {
            name: "DMGTile".into(),
            items: vec![
                MenuItem::action("About DMGTile", OpenGithub),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ],
            disabled: false,
        },
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("New Project...", NewFile),
                MenuItem::action("Open Project...", OpenFile),

                MenuItem::separator(),

                MenuItem::action("Save Project", Save),
                MenuItem::action("Save Project As...", SaveAs),

                MenuItem::separator(),

                MenuItem::action("Export as .bin ...", ExportBin),
                MenuItem::action("Export as .c ...", ExportC),
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
            items: vec![
                MenuItem::action("GitHub", OpenGithub),
                MenuItem::action("Stardance", OpenStardance),
            ],
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
