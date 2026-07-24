/// Render a text description of the artwork state.
pub fn preview_text(status: &crate::tui::artwork::ArtworkStatus) -> String {
    match &status.embedded {
        crate::tui::artwork::EmbeddedState::Checking => "checking…".to_string(),
        crate::tui::artwork::EmbeddedState::None => {
            if let Some(sidecar) = &status.sidecar {
                format!("sidecar only: {}", sidecar.display())
            } else {
                "no artwork".to_string()
            }
        }
        crate::tui::artwork::EmbeddedState::Partial { with, total } => {
            format!("embedded: {}/{} tracks", with, total)
        }
        crate::tui::artwork::EmbeddedState::All { total } => {
            format!("embedded: all {} tracks", total)
        }
    }
}
