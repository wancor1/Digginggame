import pyxel as px
import time
import math

import PyxelUniversalFont as puf

from constants import *
from utils import estimate_text_width, calculate_text_center_position
from managers import LanguageManager

class Notification:
    def __init__(self, message, duration=NOTIFICATION_MAX_DISPLAY_TIME, msg_type="info",
                 wrap_text_func=None, max_wrap_width=0):
        self.message = message
        self.start_time = time.time()
        self.duration = duration
        self.msg_type = msg_type
        self.is_alive = True

        self.state = "fading_in"
        self.current_x = None
        self.current_y = None
        self.target_x = None
        self.target_y = None
        self.vel_x = 0

        self._wrap_text = wrap_text_func
        self._max_wrap_width = max_wrap_width
        self._box_width = None
        self._box_height = None

    def get_text_color(self):
        if self.msg_type == "error":
            return NOTIFICATION_TEXT_COLOR_ERROR
        elif self.msg_type == "success":
            return NOTIFICATION_TEXT_COLOR_SUCCESS
        return NOTIFICATION_TEXT_COLOR_INFO

    def _calculate_dimensions(self):
        if not self._wrap_text:
            self._box_width = 0
            self._box_height = 0
            return

        lines = self._wrap_text(self.message, self._max_wrap_width)
        total_text_height = len(lines) * FONT_SIZE + (max(0, len(lines) - 1)) * NOTIFICATION_LINE_SPACING
        self._box_height = total_text_height + NOTIFICATION_PADDING_Y * 2

        max_line_width = max(estimate_text_width(line) for line in lines) if lines else 0
        self._box_width = min(NOTIFICATION_MAX_WIDTH, max_line_width + NOTIFICATION_PADDING_X * 2)

    def set_target_position(self, target_x, target_y):
        self.target_x = target_x
        self.target_y = target_y

        if self.current_y is None:
            self.current_x = self.target_x
            self.current_y = self.target_y + NOTIFICATION_FADE_IN_OFFSET_Y

    def update(self):
        if not self.is_alive:
            return

        if self._box_width is None:
            self._calculate_dimensions()
            if self._box_width == 0 or self._box_height == 0:
                self.is_alive = False
                return

        if self.current_y is not None and self.target_y is not None and abs(self.target_y - self.current_y) > NOTIFICATION_TARGET_Y_TOLERANCE:
            move_amount_y = NOTIFICATION_FADE_IN_AMOUNT_Y_PER_FRAME
            if self.current_y < self.target_y:
                self.current_y += min(move_amount_y, self.target_y - self.current_y)
            else:
                self.current_y -= min(move_amount_y, self.current_y - self.target_y)
        elif self.target_y is not None:
            self.current_y = self.target_y

        if self.state == "fading_in":
            if self.current_y is not None and self.target_y is not None and abs(self.target_y - self.current_y) <= NOTIFICATION_TARGET_Y_TOLERANCE:
                self.state = "visible"
        elif self.state == "visible":
            if time.time() - self.start_time > self.duration:
                self.state = "fading_out"
                self.vel_x = NOTIFICATION_FADE_OUT_INITIAL_AMOUNT_X_PER_FRAME
        elif self.state == "fading_out":
            self.vel_x += NOTIFICATION_FADE_OUT_ACCELERATION_X_PER_FRAME
            self.current_x += self.vel_x
            if self.current_x > SCREEN_WIDTH:
                self.is_alive = False

    def get_draw_position(self):
        return (self.current_x, self.current_y) if self.current_x is not None and self.current_y is not None else None

    def get_box_dimensions(self):
        return (self._box_width, self._box_height) if self._box_width is not None and self._box_height is not None else None

    def get_wrapped_lines(self):
        return self._wrap_text(self.message, self._max_wrap_width) if self._wrap_text and self._max_wrap_width > 0 else [self.message]

class SelectBlock:
    def __init__(self):
        self._selection_effect_start_time = 0
        self._is_effect_currently_active = False

    def update_selection_status(self, is_mouse_over_a_block):
        if is_mouse_over_a_block and not self._is_effect_currently_active:
            self._is_effect_currently_active = True
            self._selection_effect_start_time = time.time()
        elif not is_mouse_over_a_block:
            self._is_effect_currently_active = False

    def draw(self, mouse_x, mouse_y, is_game_active_and_not_menu):
        if not is_game_active_and_not_menu or not self._is_effect_currently_active:
            return

        grid_aligned_x = math.floor(mouse_x / BLOCK_SIZE) * BLOCK_SIZE
        grid_aligned_y = math.floor(mouse_y / BLOCK_SIZE) * BLOCK_SIZE
        elapsed_time = time.time() - self._selection_effect_start_time

        if elapsed_time <= 1.0:
            px.blt(grid_aligned_x, grid_aligned_y, *SPRITE_SELECT_NORMAL)
        elif elapsed_time <= 2.0:
            px.blt(grid_aligned_x - 1, grid_aligned_y - 1, *SPRITE_SELECT_LARGE)
        else:
            self._selection_effect_start_time = time.time()

class ButtonBox:
    def __init__(self, font_writer: puf.Writer, lang_manager: LanguageManager):
        self.font = font_writer
        self.lang_manager = lang_manager

    def _get_mouse_status_on_button(self, x, y, w, h):
        is_hover = x <= px.mouse_x <= x + w - 1 and y <= px.mouse_y <= y + h - 1
        is_pressed = is_hover and px.btn(px.MOUSE_BUTTON_LEFT)
        is_released = is_hover and px.btnr(px.MOUSE_BUTTON_LEFT)
        return is_pressed, is_hover, is_released

    def draw_button(self, x, y, w, h, text_key='text', pressed_text_key='press'):
        is_pressed, _, is_released = self._get_mouse_status_on_button(x, y, w, h)
        current_text_str = self.lang_manager.get_string(pressed_text_key if is_pressed else text_key)
        bg_color = COLOR_BUTTON_PRESSED_BG if is_pressed else COLOR_BUTTON_BG
        px.rect(x, y, w, h, bg_color)
        px.rectb(x, y, w, h, COLOR_BUTTON_BORDER)
        if not is_pressed:
            px.line(x + w - 1, y + 1, x + w - 1, y + h - 2, 0)
            px.line(x + 1, y + h - 1, x + w - 2, y + h - 1, 0)
        text_x_offset, text_y_offset = calculate_text_center_position(w, h, current_text_str)
        self.font.draw(x + text_x_offset, y + text_y_offset, current_text_str, FONT_SIZE, COLOR_BUTTON_TEXT)
        return is_released

class GameMenu:
    def __init__(self, game_instance_ref, lang_manager_ref: LanguageManager, font_writer_ref: puf.Writer):
        self.game = game_instance_ref
        self.lang_manager = lang_manager_ref
        self.font = font_writer_ref
        self.button_handler = ButtonBox(self.font, self.lang_manager)
        self.width = 100
        self.height = SCREEN_HEIGHT - (SCREEN_HEIGHT // 10 * 2) + MENU_ITEM_HEIGHT // 2 * 3
        self.x = (SCREEN_WIDTH - self.width) // 2
        self.y = (SCREEN_HEIGHT - self.height) // 2
        self.menu_items_def = [
            {"key": "menu.sound_effects", "type": "checkbox", "setting_attr": "se_on"},
            {"key": "menu.music", "type": "checkbox", "setting_attr": "bgm_on"},
            {"key": "menu.language", "type": "dropdown"},
            {"key": "button.menu.save.default", "press_key":"button.menu.save.pressed", "type": "button", "action_label": "Save Game"},
            {"key": "button.menu.load.default", "press_key":"button.menu.load.pressed", "type": "button", "action_label": "Load Game"},
            {"key": "button.menu.quit.default", "press_key":"button.menu.quit.pressed", "type": "button", "action_label": "Quit Game"}
        ]
        self.selected_button_action = None
        self.is_lang_dropdown_open = False
        self.lang_dropdown_trigger_rect = None
        self.lang_dropdown_options_rects = []

    def _draw_checkbox(self, x, y, label_text, is_checked):
        check_y = y + (MENU_ITEM_HEIGHT - CHECKBOX_SIZE) // 2
        px.rectb(x, check_y, CHECKBOX_SIZE, CHECKBOX_SIZE, 0)
        if is_checked:
            px.rect(x + 2, check_y + 2, CHECKBOX_SIZE - 4, CHECKBOX_SIZE - 4, 0)
        self.font.draw(x + CHECKBOX_SIZE + CHECKBOX_TEXT_GAP, y + (MENU_ITEM_HEIGHT - FONT_SIZE) // 2, label_text, FONT_SIZE, 0)

    def handle_input(self):
        if not self.game.is_menu_visible:
            self.is_lang_dropdown_open = False
            return

        clicked_outside_dropdown = False
        if self.is_lang_dropdown_open and px.btnp(px.MOUSE_BUTTON_LEFT):
            clicked_on_option = False
            for opt_x, opt_y, opt_w, opt_h, lang_code in self.lang_dropdown_options_rects:
                if opt_x <= px.mouse_x < opt_x + opt_w and opt_y <= px.mouse_y < opt_y + opt_h:
                    self.lang_manager.set_language(lang_code)
                    self.game.current_language_code = lang_code
                    self.game.update_window_title()
                    self.is_lang_dropdown_open = False
                    clicked_on_option = True
                    break
            if not clicked_on_option and self.lang_dropdown_trigger_rect and not (self.lang_dropdown_trigger_rect[0] <= px.mouse_x < self.lang_dropdown_trigger_rect[0] + self.lang_dropdown_trigger_rect[2] and self.lang_dropdown_trigger_rect[1] <= px.mouse_y < self.lang_dropdown_trigger_rect[1] + self.lang_dropdown_trigger_rect[3]):
                clicked_outside_dropdown = True

        if px.btnp(px.MOUSE_BUTTON_LEFT) and (not self.is_lang_dropdown_open or clicked_outside_dropdown):
            current_y = self.y + MENU_ITEM_HEIGHT + MENU_PADDING * 2
            for item_def in self.menu_items_def:
                item_x = self.x + MENU_PADDING
                item_w = self.width - MENU_PADDING * 2
                item_rect = (item_x, current_y, item_w, MENU_ITEM_HEIGHT)
                if item_rect[0] <= px.mouse_x < item_rect[0] + item_rect[2] and item_rect[1] <= px.mouse_y < item_rect[1] + item_rect[3]:
                    if item_def["type"] == "checkbox":
                        setattr(self.game, item_def["setting_attr"], not getattr(self.game, item_def["setting_attr"]))
                        break
                    elif item_def["type"] == "dropdown":
                        self.is_lang_dropdown_open = not self.is_lang_dropdown_open
                        break
                current_y += MENU_ITEM_HEIGHT + MENU_PADDING

        if clicked_outside_dropdown:
            self.is_lang_dropdown_open = False

    def draw(self):
        if not self.game.is_menu_visible:
            self.is_lang_dropdown_open = False
            return None

        self.selected_button_action = None
        px.rect(self.x, self.y, self.width, self.height, 9)
        px.rectb(self.x, self.y, self.width, self.height, 0)
        menu_title_str = self.lang_manager.get_string("menu.title")
        title_x_offset, title_y_offset = calculate_text_center_position(self.width, MENU_ITEM_HEIGHT, menu_title_str)
        self.font.draw(self.x + title_x_offset, self.y + MENU_PADDING + title_y_offset, menu_title_str, FONT_SIZE, 0)

        current_y = self.y + MENU_ITEM_HEIGHT + MENU_PADDING * 2
        dropdown_draw_params = None

        for item_def in self.menu_items_def:
            item_x = self.x + MENU_PADDING
            item_w = self.width - MENU_PADDING * 2
            if item_def["type"] == "checkbox":
                self._draw_checkbox(item_x, current_y, self.lang_manager.get_string(item_def["key"]), getattr(self.game, item_def["setting_attr"]))
            elif item_def["type"] == "dropdown":
                lang_label_prefix = self.lang_manager.get_string(item_def["key"]) + ":"
                self.font.draw(item_x, current_y + (MENU_ITEM_HEIGHT - FONT_SIZE) // 2, lang_label_prefix, FONT_SIZE, 0)
                dropdown_button_x = item_x + estimate_text_width(lang_label_prefix) + MENU_PADDING
                dropdown_button_w = item_w - (estimate_text_width(lang_label_prefix) + MENU_PADDING)
                current_lang_display_name = self.lang_manager.get_available_languages().get(self.lang_manager.current_lang_code, self.lang_manager.current_lang_code)
                px.rectb(dropdown_button_x, current_y, dropdown_button_w, MENU_ITEM_HEIGHT, 0)
                self.font.draw(dropdown_button_x + MENU_PADDING, current_y + (MENU_ITEM_HEIGHT - FONT_SIZE) // 2, current_lang_display_name, FONT_SIZE, 0)
                px.tri(dropdown_button_x + dropdown_button_w - DROPDOWN_ARROW_WIDTH - 2, current_y + (MENU_ITEM_HEIGHT - DROPDOWN_ARROW_HEIGHT)//2, dropdown_button_x + dropdown_button_w - 2 - DROPDOWN_ARROW_WIDTH//2, current_y + (MENU_ITEM_HEIGHT + DROPDOWN_ARROW_HEIGHT)//2, dropdown_button_x + dropdown_button_w - 2, current_y + (MENU_ITEM_HEIGHT - DROPDOWN_ARROW_HEIGHT)//2, 0)
                self.lang_dropdown_trigger_rect = (dropdown_button_x, current_y, dropdown_button_w, MENU_ITEM_HEIGHT)
                if self.is_lang_dropdown_open:
                    dropdown_draw_params = (dropdown_button_x, current_y, dropdown_button_w)
            elif item_def["type"] == "button":
                if self.button_handler.draw_button(item_x, current_y, item_w, MENU_ITEM_HEIGHT, item_def["key"], item_def["press_key"]):
                    if not self.is_lang_dropdown_open:
                        self.selected_button_action = item_def["action_label"]
            current_y += MENU_ITEM_HEIGHT + MENU_PADDING

        if dropdown_draw_params:
            dd_x, dd_y, dd_w = dropdown_draw_params
            options_y_start = dd_y + MENU_ITEM_HEIGHT
            available_langs = self.lang_manager.get_available_languages()
            max_option_width = dd_w
            for _, display_name_opt in available_langs.items():
                 max_option_width = max(max_option_width, estimate_text_width(display_name_opt) + MENU_PADDING * 2)
            list_bg_x, list_bg_y, list_bg_w, list_bg_h = dd_x, options_y_start, max_option_width, len(available_langs) * MENU_ITEM_HEIGHT
            px.rect(list_bg_x, list_bg_y, list_bg_w, list_bg_h, 13)
            px.rectb(list_bg_x, list_bg_y, list_bg_w, list_bg_h, 0)
            self.lang_dropdown_options_rects = []
            current_opt_y = options_y_start
            for lang_code, display_name in available_langs.items():
                option_rect_for_draw = (list_bg_x, current_opt_y, list_bg_w, MENU_ITEM_HEIGHT)
                self.lang_dropdown_options_rects.append((*option_rect_for_draw, lang_code))
                if option_rect_for_draw[0] <= px.mouse_x < option_rect_for_draw[0] + option_rect_for_draw[2] and option_rect_for_draw[1] <= px.mouse_y < option_rect_for_draw[1] + option_rect_for_draw[3]:
                    px.rect(*option_rect_for_draw, 1)
                self.font.draw(list_bg_x + MENU_PADDING, current_opt_y + (MENU_ITEM_HEIGHT - FONT_SIZE) // 2, display_name, FONT_SIZE, 0)
                current_opt_y += MENU_ITEM_HEIGHT
        return self.selected_button_action
