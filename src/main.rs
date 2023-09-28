
use native_windows_gui as nwg;
use native_windows_derive as nwd;

use nwg::NativeUi;

use std::env;
// use std::cell::RefCell;
use std::fs::File;
use std::io::Read;  // for read_to_string

use subparse::SrtFile;
use subparse::SubtitleFileInterface;




trait ListViewEx {
    fn insert_column_with_width (&self, n: &str, w: i32);
}

impl ListViewEx for nwg::ListView {
    fn insert_column_with_width (&self, n: &str, w: i32) {
        self.insert_column(nwg::InsertListViewColumn{
            index: None,
            fmt: None,
            width: Some(w),
            text: Some(n.to_string())
        })
    }
}


struct MyError(String);


#[derive(Default, nwd::NwgUi)]
pub struct QuietWordsApp {
    // The video that will be loaded dynamically
    //loaded_video: RefCell<Option<nwg::Bitmap>>,

    #[nwg_control(
        size: (1200, 500),
        title: "Quiet Words - Subtitles Editor"
    )]
    #[nwg_events(
        OnWindowClose: [QuietWordsApp::exit],
        OnInit: [QuietWordsApp::init]
    )]
    window: nwg::Window,

    #[nwg_layout(parent: window)]
    layout: nwg::GridLayout,

    #[nwg_resource(
        title: "Select Subtitles File",
        action: nwg::FileDialogAction::Open,
        filters: "SRT(*.srt)"
    )]
    dialog_load_subtitles: nwg::FileDialog,

    #[nwg_resource(
        title: "Save Subtitles File",
        action: nwg::FileDialogAction::Save,
        filters: "SRT(*.srt)"
    )]
    dialog_save_subtitles: nwg::FileDialog,

    #[nwg_resource(
        title: "Select Video File",
        action: nwg::FileDialogAction::Open,
        filters: "MP4(*.mp4)"
    )]
    dialog_select_video: nwg::FileDialog,


    #[nwg_control(text: "Subtitles:")]
    #[nwg_layout_item(layout: layout, col: 0, row: 0)]
    label_subtitles: nwg::Label,

    #[nwg_control(text: "Video:")]
    #[nwg_layout_item(layout: layout, col: 3, row: 0)]
    label_video: nwg::Label,


    #[nwg_control(text: "Load", focus: true)]
    #[nwg_layout_item(layout: layout, col: 0, row: 1)]
    #[nwg_events(OnButtonClick: [QuietWordsApp::load_subtitles])]
    btn_load_subtitles: nwg::Button,

    #[nwg_control(text: "Save")]
    #[nwg_layout_item(layout: layout, col: 1, row: 1)]
    #[nwg_events(OnButtonClick: [QuietWordsApp::save_subtitles])]
    btn_save_subtitles: nwg::Button,

    #[nwg_control(text: "Select")]
    #[nwg_layout_item(layout: layout, col: 3, row: 1)]
    #[nwg_events(OnButtonClick: [QuietWordsApp::select_video])]
    btn_select_video: nwg::Button,

    #[nwg_control(text: "Settings")]
    #[nwg_layout_item(layout: layout, col: 6, row: 0)]
    #[nwg_events(OnButtonClick: [QuietWordsApp::settings])]
    btn_settings: nwg::Button,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: layout, col: 0, row: 2, col_span: 3)]
    filename_subtitles: nwg::TextInput,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: layout, col: 4, row: 1, col_span: 3)]
    filename_video: nwg::TextInput,


    #[nwg_control(text: "00:00:00,000")]
    #[nwg_layout_item(layout: layout, col: 0, row: 3)]
    time_start: nwg::TextInput,

    #[nwg_control(text: "00:00:00,000")]
    #[nwg_layout_item(layout: layout, col: 0, row: 4)]
    time_end: nwg::TextInput,

    // https://learn.microsoft.com/en-us/windows/win32/controls/about-edit-controls#text-and-input-styles
    #[nwg_control(
        flags: "VISIBLE|AUTOVSCROLL|AUTOHSCROLL"
    )]
    #[nwg_layout_item(layout: layout, col: 1, row: 3, col_span: 2, row_span: 2)]
    subtitle_text: nwg::TextBox,

    #[nwg_control(text: "Cancel")]
    #[nwg_layout_item(layout: layout, col: 0, row: 5)]
    #[nwg_events(OnButtonClick: [QuietWordsApp::cancel_subtitle])]
    btn_cancel_subtitle: nwg::Button,

    #[nwg_control(text: "Save")]
    #[nwg_layout_item(layout: layout, col: 2, row: 5)]
    #[nwg_events(OnButtonClick: [QuietWordsApp::save_subtitle])]
    btn_save_subtitle: nwg::Button,

    // TODO: disable the inputs + buttons for subtitle when nothing selected, or after cancel -- deselect


    #[nwg_control(
        item_count: 10,
        size: (500, 350),
        list_style: nwg::ListViewStyle::Detailed,
        ex_flags:
            nwg::ListViewExFlags::GRID |
            nwg::ListViewExFlags::FULL_ROW_SELECT,
    )]
    #[nwg_layout_item(layout: layout, col: 0, row: 6, col_span: 3, row_span: 4)]
    subtitles_list: nwg::ListView,

    // https://github.com/gabdube/native-windows-gui/blob/master/native-windows-gui/examples/basic_drawing_d.rs
    // https://github.com/gabdube/native-windows-gui/blob/master/native-windows-gui/src/controls/extern_canvas.rs
    // #[nwg_control(parent: Some(&(data.window)))]
    #[nwg_control]
    #[nwg_events(
        OnPaint: [QuietWordsApp::paint(SELF, EVT_DATA)],
        OnMousePress: [QuietWordsApp::events(SELF, EVT)],
    )]
    #[nwg_layout_item(layout: layout, col: 3, row: 2, col_span: 4, row_span: 7)]
    // video: nwg::ExternCanvas,
    video: nwg::ImageFrame,

    #[nwg_control(text: "RW")]
    #[nwg_layout_item(layout: layout, col: 3, row: 9)]
    #[nwg_events(OnButtonClick: [QuietWordsApp::rewind])]
    btn_rewind: nwg::Button,

    #[nwg_control(text: "PP")]
    #[nwg_layout_item(layout: layout, col: 4, row: 9)]
    #[nwg_events(OnButtonClick: [QuietWordsApp::play_pause])]
    btn_play_pause: nwg::Button,

    #[nwg_control(text: "FF")]
    #[nwg_layout_item(layout: layout, col: 5, row: 9)]
    #[nwg_events(OnButtonClick: [QuietWordsApp::fast_forward])]
    btn_fast_forward: nwg::Button,

    #[nwg_control(
        text: "1000",
        flags: "VISIBLE|NUMBER",
    )]
    #[nwg_layout_item(layout: layout, col: 6, row: 9)]
    increment_ms: nwg::TextInput,


    // #[nwg_control(collection: vec!["Simple", "Details", "Icon", "Icon small"], selected_index: Some(1), font: Some(&data.arial))]
    // #[nwg_layout_item(layout: layout, col: 4, row: 1)]
    // #[nwg_events( OnComboxBoxSelection: [QuietWords::update_view] )]
    // view_style: nwg::ComboBox<&'static str>
}

impl QuietWordsApp {

    fn init (&self) {
        let dv = &self.subtitles_list;
        dv.set_headers_enabled(true);

        // TODO: highlight rows that are overlapping

        dv.insert_column_with_width("Idx", 40);
        dv.insert_column_with_width("Start", 80);
        dv.insert_column_with_width("End", 80);
        dv.insert_column_with_width("Duration", 80);
        dv.insert_column_with_width("Lines", 40);
        dv.insert_column_with_width("Text", 160);

        // dv.update_item(4, nwg::InsertListViewItem { image: Some(1), ..Default::default() });
    }

    fn load_subtitles (&self) {
        if let Ok(dir) = env::current_dir() {
            if let Some(dir) = dir.to_str() {
                self.dialog_load_subtitles.set_default_folder(dir).expect("Failed to set default folder.");
            }
        }

        if self.dialog_load_subtitles.run(Some(&self.window)) {
            if let Ok(directory) = self.dialog_load_subtitles.get_selected_item() {
                let dir = directory.into_string().unwrap();
                self.filename_subtitles.set_text(&dir);
                self.load_subtitles_impl();
            }
        }
    }

    // TODO replace expects with returning a result and showing an error dialog w `error_message`
    fn load_subtitles_impl (&self) {
        // https://doc.rust-lang.org/std/fs/struct.File.html
        let mut file = File::open(&self.filename_subtitles.text()).expect("File exists");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Could not read file");

        match SrtFile::parse(&contents) {
            Ok(srt) => {
                let subtitles = srt.get_subtitle_entries().expect("Should not fail to get subtitles from srt");

                let dv = &self.subtitles_list;
                dv.clear();

                for (row_idx, subtitle) in subtitles.iter().enumerate() {
                    let text = subtitle.line.as_ref().expect("Srt should always have a line");
                    let line_count = 1 + text.as_bytes().iter().filter(|&&c| c == b'\n').count();
                    dv.insert_items_row(None, &[
                        row_idx.to_string(),
                        subtitle.timespan.start.to_string(),
                        subtitle.timespan.end.to_string(),
                        format!("({})", subtitle.timespan.len()),
                        // https://llogiq.github.io/2016/09/24/newline.html
                        line_count.to_string(),
                        text.to_string()
                    ]);
                }
            },
            Err(e) => {
                nwg::error_message("SRT Error", format!("Parsing failed: {:?}", e).as_str());
            }
        }
    }

    fn save_subtitles (&self) {
    }

    fn select_video (&self) {
    }

    fn settings (&self) {
    }

    fn rewind (&self) {
    }

    fn fast_forward (&self) {
    }

    fn play_pause (&self) {
    }

    fn cancel_subtitle (&self) {
    }

    fn save_subtitle (&self) {
    }

    fn paint(&self, data: &nwg::EventData) {
    }

    fn events(&self, evt: nwg::Event) {
        use nwg::Event as E;
        use nwg::MousePressEvent as M;

        match evt {
            // E::OnMousePress(M::MousePressLeftUp) => { self.clicked.set(false); },
            // E::OnMousePress(M::MousePressLeftDown) => { self.clicked.set(true); },
            _ => {},
        }

        // self.video.invalidate();
    }

    // fn update_view(&self) {
    //     let value = self.view_style.selection_string();

    //     let style = match value.as_ref().map(|v| v as &str) {
    //         // Some("Icon") => nwg::ListViewStyle::Icon,
    //         // Some("Icon small") => nwg::ListViewStyle::SmallIcon,
    //         // Some("Details") => nwg::ListViewStyle::Detailed,
    //         None | Some(_) => nwg::ListViewStyle::Detailed,
    //     };

    //     self.data_view.set_list_style(style);
    // }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let _app = QuietWordsApp::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
