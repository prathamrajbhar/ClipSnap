pub mod history_dialog;
pub mod overlay;
pub mod settings_dialog;

pub fn load_logo_pixbuf(size: i32) -> Option<gdk_pixbuf::Pixbuf> {
    let logo_path = std::path::PathBuf::from("assets/logo.png");
    if let Ok(pb) = gdk_pixbuf::Pixbuf::from_file(&logo_path) {
        return pb.scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear);
    }
    None
}
