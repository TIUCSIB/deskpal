use std::sync::Mutex;

/** state.rs - 浮窗可见性协调状态 */
#[derive(Default)]
pub struct OverlayState {
    info_requested_visible: Mutex<bool>,
}

impl OverlayState {
    pub fn set_info_requested_visible(&self, visible: bool) -> Result<(), String> {
        let mut requested = self
            .info_requested_visible
            .lock()
            .map_err(|_| "浮窗状态暂时不可用".to_string())?;
        *requested = visible;
        Ok(())
    }

    pub fn info_requested_visible(&self) -> bool {
        self.info_requested_visible
            .lock()
            .map(|visible| *visible)
            .unwrap_or(false)
    }
}
