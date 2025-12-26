use crate::constants::{
    MAX_NOTIFICATIONS, NOTIFICATION_INTER_ITEM_SPACING, NOTIFICATION_MAX_DISPLAY_TIME,
    NOTIFICATION_MAX_WIDTH, NOTIFICATION_PADDING_X, NOTIFICATION_PADDING_Y, SCREEN_WIDTH,
};
use crate::ui::{Notification, NotificationState};
use macroquad::prelude::*;

pub struct NotificationManager {
    pub notifications: Vec<Notification>,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationManager {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            notifications: Vec::new(),
        }
    }

    pub fn add_notification(&mut self, message: &str, msg_type: &str, font: Option<&Font>) {
        let max_width = NOTIFICATION_PADDING_X.mul_add(-2.0, NOTIFICATION_MAX_WIDTH);
        let notif = Notification::new(
            message,
            f64::from(NOTIFICATION_MAX_DISPLAY_TIME),
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

    pub fn draw_high_res(&self, font: Option<&Font>, scale_fac: f32, off_x: f32, off_y: f32) {
        for notif in &self.notifications {
            notif.draw_high_res(font, scale_fac, off_x, off_y);
        }
    }
}
