/** policy.rs - 覆盖窗口的展示优先级。 */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OverlayWinner {
    ContextMenu,
    Chat,
    Reminder,
    Feedback,
    Info,
    None,
}

pub(super) fn overlay_winner(
    context_menu_visible: bool,
    chat_visible: bool,
    reminder_active: bool,
    feedback_active: bool,
    info_visible: bool,
) -> OverlayWinner {
    if context_menu_visible {
        OverlayWinner::ContextMenu
    } else if chat_visible {
        OverlayWinner::Chat
    } else if reminder_active {
        OverlayWinner::Reminder
    } else if feedback_active {
        OverlayWinner::Feedback
    } else if info_visible {
        OverlayWinner::Info
    } else {
        OverlayWinner::None
    }
}

#[cfg(test)]
mod tests {
    use super::{overlay_winner, OverlayWinner};

    #[test]
    fn selects_the_highest_priority_overlay() {
        assert_eq!(
            overlay_winner(true, true, true, true, true),
            OverlayWinner::ContextMenu
        );
        assert_eq!(
            overlay_winner(false, true, true, true, true),
            OverlayWinner::Chat
        );
        assert_eq!(
            overlay_winner(false, false, true, true, true),
            OverlayWinner::Reminder
        );
        assert_eq!(
            overlay_winner(false, false, false, true, true),
            OverlayWinner::Feedback
        );
        assert_eq!(
            overlay_winner(false, false, false, false, true),
            OverlayWinner::Info
        );
    }
}
