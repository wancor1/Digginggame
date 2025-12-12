use crate::constants::*;
use crate::ui::{Notification, NotificationState};
use macroquad::prelude::*;

pub struct NotificationManager {
    pub notifications: Vec<Notification>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
        }
    }

    pub fn add_notification(&mut self, message: String, msg_type: &str, font: Option<&Font>) {
        let max_width = NOTIFICATION_MAX_WIDTH - NOTIFICATION_PADDING_X * 2.0;
        let notif = Notification::new(
            message,
            NOTIFICATION_MAX_DISPLAY_TIME as f64,
            msg_type,
            max_width,
            font,
        );
        self.notifications.push(notif);
        if self.notifications.len() > MAX_NOTIFICATIONS {
            self.notifications.remove(0);
        }
    }

    pub fn update(&mut self) {
        self.notifications.retain(|n| n.is_alive);
        for n in &mut self.notifications {
            n.update();
        }

        // Stack logic
        let mut current_target_y = NOTIFICATION_PADDING_Y;
        for notif in self.notifications.iter_mut().rev() {
            if notif.is_alive && notif.state != NotificationState::FadingOut {
                let box_w = notif.get_box_width();
                let box_h = notif.get_box_height();
                let target_x = SCREEN_WIDTH - box_w - NOTIFICATION_PADDING_X;
                notif.set_target_position(target_x, current_target_y);
                current_target_y += box_h + NOTIFICATION_INTER_ITEM_SPACING;
            }
        }
    }

    pub fn draw(&self, font: Option<&Font>) {
        for notif in &self.notifications {
            notif.draw(font);
        }
    }
}
