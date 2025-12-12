import pyxel as px
import random
import math
import time
import os
import json
import traceback
import threading

import PyxelUniversalFont as puf

from constants import *
from utils import *
from components import Chunk

class LanguageManager:
    def __init__(self, lang_folder=LANG_FOLDER, default_lang=DEFAULT_LANGUAGE):
        self.lang_folder = lang_folder
        self.languages = {}
        self.translations = {}
        self.current_lang_code = default_lang
        self._discover_languages()

        if not self.languages:
            self._create_default_lang_files_if_missing()
            self._discover_languages()

        if not self.load_language(self.current_lang_code):
            if self.languages:
                fallback_lang = list(self.languages.keys())[0]
                self.load_language(fallback_lang)
            else:
                self.translations = {"error_no_lang": "No languages loaded!"}

    def _create_default_lang_files_if_missing(self):
        if not os.path.exists(self.lang_folder):
            try:
                os.makedirs(self.lang_folder)
            except OSError:
                return

        en_path = os.path.join(self.lang_folder, "en_us.json")
        if not os.path.exists(en_path):
            try:
                import gen_translation
                en_data = gen_translation._generate_en_()
                with open(en_path, "w", encoding="utf-8") as f:
                    json.dump(en_data, f, ensure_ascii=False, indent=2)
            except (IOError, ImportError):
                pass

    def _discover_languages(self):
        self.languages = {}
        if not os.path.isdir(self.lang_folder):
            return

        for filename in os.listdir(self.lang_folder):
            if filename.endswith(".json"):
                lang_code = filename[:-5]
                file_path = os.path.join(self.lang_folder, filename)
                display_name = lang_code
                try:
                    with open(file_path, "r", encoding="utf-8") as f:
                        data = json.load(f)
                        if "_metadata" in data and "display_name" in data["_metadata"]:
                            display_name = data["_metadata"]["display_name"]
                except Exception:
                    pass
                self.languages[lang_code] = {"display_name": display_name, "path": file_path}

    def get_available_languages(self):
        return {code: data["display_name"] for code, data in self.languages.items()}

    def load_language(self, lang_code):
        if lang_code not in self.languages:
            return False

        lang_file_path = self.languages[lang_code]["path"]
        try:
            with open(lang_file_path, "r", encoding="utf-8") as f:
                self.translations = json.load(f)
            self.current_lang_code = lang_code
            return True
        except (FileNotFoundError, json.JSONDecodeError):
            self.translations = {"error_load_failed": f"Failed to load {lang_code}"}
            return False

    def get_string(self, key, **kwargs):
        val = self.translations.get(key, f"{key}")
        if kwargs:
            try:
                return val.format(**kwargs)
            except KeyError:
                return val
        return val

    def set_language(self, lang_code):
        return self.load_language(lang_code)

class NotificationManager:
    def __init__(self, font_writer: puf.Writer):
        self.notifications = []
        self.font = font_writer

    def add_notification(self, message, duration=NOTIFICATION_MAX_DISPLAY_TIME, msg_type="info"):
        from ui import Notification # Circular import
        effective_wrap_width = NOTIFICATION_MAX_WIDTH - NOTIFICATION_PADDING_X * 2
        new_notif = Notification(message, duration, msg_type, wrap_text_func=self._wrap_text, max_wrap_width=effective_wrap_width)
        self.notifications.append(new_notif)
        if len(self.notifications) > MAX_NOTIFICATIONS:
            self.notifications.pop(0)

    def _wrap_chars_only(self, text, max_width):
        wrapped_lines = []
        if not text:
            return [""]
        current_line = ""
        for char in text:
            test_line = current_line + char
            if estimate_text_width(test_line) > max_width and current_line:
                wrapped_lines.append(current_line)
                current_line = char
            else:
                current_line = test_line
        if current_line:
            wrapped_lines.append(current_line)
        return wrapped_lines

    def _wrap_text(self, text, max_width):
        final_lines = []
        for para in text.split('\n'):
            if not para:
                final_lines.append("")
                continue
            current_line = ""
            for word in para.split(' '):
                if estimate_text_width(word) > max_width:
                    if current_line:
                        final_lines.append(current_line)
                        current_line = ""
                    final_lines.extend(self._wrap_chars_only(word, max_width))
                elif not current_line:
                    current_line = word
                elif estimate_text_width(current_line + " " + word) <= max_width:
                    current_line += " " + word
                else:
                    final_lines.append(current_line)
                    current_line = word
            if current_line:
                final_lines.append(current_line)
        return final_lines

    def update(self):
        self.notifications = [n for n in self.notifications if n.is_alive]
        for notif in self.notifications:
            notif.update()

        current_target_y = NOTIFICATION_PADDING_Y
        for notif in reversed(self.notifications):
            if notif.is_alive and notif.state != "fading_out":
                notif.set_target_position(SCREEN_WIDTH - notif.get_box_dimensions()[0] - NOTIFICATION_PADDING_X, current_target_y)
                current_target_y += notif.get_box_dimensions()[1] + NOTIFICATION_INTER_ITEM_SPACING

    def draw(self):
        for notif in self.notifications:
            draw_pos = notif.get_draw_position()
            box_dims = notif.get_box_dimensions()
            if draw_pos and box_dims:
                box_x, box_y = draw_pos
                box_width, box_height = box_dims
                px.rect(int(box_x), int(box_y), int(box_width), int(box_height), NOTIFICATION_BG_COLOR)
                px.rectb(int(box_x), int(box_y), int(box_width), int(box_height), notif.get_text_color())
                line_y_offset = box_y + NOTIFICATION_PADDING_Y
                for line in notif.get_wrapped_lines():
                    self.font.draw(int(box_x + NOTIFICATION_PADDING_X), int(line_y_offset), line, FONT_SIZE, notif.get_text_color())
                    line_y_offset += FONT_SIZE + NOTIFICATION_LINE_SPACING

class ParticleManager:
    def __init__(self):
        self.active_particles = []

    def add_particles(self, new_particles):
        self.active_particles.extend(new_particles)

    def update(self, collidable_blocks):
        for particle in self.active_particles:
            particle.update(collidable_blocks)
        self.active_particles = [p for p in self.active_particles if p.alive]

    def draw(self):
        for particle in self.active_particles:
            particle.draw()

class WorldManager:
    def __init__(self, game):
        self.game = game
        self.chunks = {}
        self.generated_chunk_coords = set()
        self.world_seed_main = px.rndi(1, 2**31 - 1)
        self.world_seed_ore = px.rndi(1, 2**31 - 1)
        px.nseed(self.world_seed_main)
        px.rseed(self.world_seed_main)

    def ensure_chunk_generated_and_get(self, chunk_x, chunk_y):
        if (chunk_x, chunk_y) not in self.chunks:
            new_chunk = Chunk(chunk_x, chunk_y, self.game)
            self.chunks[(chunk_x, chunk_y)] = new_chunk

        chunk = self.chunks[(chunk_x, chunk_y)]
        if not chunk.is_generated:
            chunk._initialize_blocks()
            self.generated_chunk_coords.add((chunk_x, chunk_y))
        return chunk

    def generate_visible_chunks(self):
        cam_world_left = self.game.camera_x
        cam_world_right = self.game.camera_x + SCREEN_WIDTH
        cam_world_top = self.game.camera_y
        cam_world_bottom = self.game.camera_y + SCREEN_HEIGHT

        start_cx, start_cy = world_to_chunk_coords(cam_world_left, cam_world_top)
        end_cx, end_cy = world_to_chunk_coords(cam_world_right, cam_world_bottom)

        for cx in range(start_cx, end_cx + 1):
            for cy in range(start_cy, end_cy + 1):
                self.ensure_chunk_generated_and_get(cx, cy)

    def get_block_at_world_coords(self, world_x, world_y):
        chunk_x, chunk_y = world_to_chunk_coords(world_x, world_y)
        chunk = self.ensure_chunk_generated_and_get(chunk_x, chunk_y)
        if chunk:
            return chunk.get_block_by_world_coords(world_x, world_y)
        return None

    def get_active_blocks_in_view(self):
        active_blocks = []
        cam_world_left = self.game.camera_x - BLOCK_SIZE
        cam_world_right = self.game.camera_x + SCREEN_WIDTH + BLOCK_SIZE
        cam_world_top = self.game.camera_y - BLOCK_SIZE
        cam_world_bottom = self.game.camera_y + SCREEN_HEIGHT + BLOCK_SIZE

        start_cx, start_cy = world_to_chunk_coords(cam_world_left, cam_world_top)
        end_cx, end_cy = world_to_chunk_coords(cam_world_right, cam_world_bottom)

        for cx in range(start_cx, end_cx + 1):
            for cy in range(start_cy, end_cy + 1):
                chunk = self.chunks.get((cx, cy))
                if chunk and chunk.is_generated:
                    for block in chunk.get_all_active_blocks_in_chunk():
                        if (block.x + BLOCK_SIZE > self.game.camera_x and 
                            block.x < self.game.camera_x + SCREEN_WIDTH and 
                            block.y + BLOCK_SIZE > self.game.camera_y and 
                            block.y < self.game.camera_y + SCREEN_HEIGHT):
                            active_blocks.append(block)
        return active_blocks

    def regenerate_world_from_chunks_and_apply_mods(self, loaded_gen_chunk_coords_set, loaded_mod_chunks_data):
        self.chunks = {}
        self.generated_chunk_coords = set()

        for cx_cy_tuple in loaded_gen_chunk_coords_set:
            self.generated_chunk_coords.add(cx_cy_tuple)
        for cx, cy in self.generated_chunk_coords:
            self.ensure_chunk_generated_and_get(cx, cy)

        mod_chunks_map = {(cd["cx"], cd["cy"]): cd["modified_blocks"] for cd in loaded_mod_chunks_data}
        for chunk_coord, modified_blocks_list in mod_chunks_map.items():
            cx, cy = chunk_coord
            if (cx, cy) in self.chunks:
                self.chunks[(cx, cy)].apply_loaded_block_data(modified_blocks_list)

class PersistenceManager:
    def __init__(self, game):
        self.game = game
        self.save_thread = None
        self.load_thread = None
        self.save_result = None

    @property
    def is_saving(self):
        return self.save_thread and self.save_thread.is_alive()

    def _save_work(self):
        modified_chunks_data = []
        for chunk_coord, chunk_instance in self.game.world_manager.chunks.items():
            if chunk_instance.is_modified_in_session:
                chunk_save_data = chunk_instance.to_save_data()
                if chunk_save_data:
                    modified_chunks_data.append(chunk_save_data)

        save_data = {
            "camera_x": self.game.camera_x, "camera_y": self.game.camera_y,
            "se_on": self.game.se_on, "bgm_on": self.game.bgm_on,
            "world_seed_main": self.game.world_manager.world_seed_main,
            "world_seed_ore": self.game.world_manager.world_seed_ore,
            "generated_chunk_coords": [list(c) for c in self.game.world_manager.generated_chunk_coords],
            "modified_chunks": modified_chunks_data,
            "current_language": self.game.lang_manager.current_lang_code
        }

        message_prefix = "debug" if self.game.show_debug_info else "default"
        try:
            with open(SAVE_FILE_NAME, "w") as f:
                json.dump(save_data, f, indent=2)
            message = self.game.lang_manager.get_string(f"notification.save.success.{message_prefix}", filename=SAVE_FILE_NAME)
            self.save_result = (True, message)
        except IOError as e:
            message = self.game.lang_manager.get_string(f"notification.save.error.write.{message_prefix}", filename=SAVE_FILE_NAME, error=str(e))
            self.save_result = (False, message)
        except Exception as e:
            message = self.game.lang_manager.get_string(f"notification.save.error.unexpected.{message_prefix}", error=str(e))
            self.save_result = (False, message)
            traceback.print_exc()

    def save_game_state(self):
        if self.is_saving:
            self.game.notification_manager.add_notification(
                self.game.lang_manager.get_string("notification.save.in_progress")
            )
            return

        self.save_result = None
        self.save_thread = threading.Thread(target=self._save_work)
        self.save_thread.start()
        self.game.notification_manager.add_notification(
            self.game.lang_manager.get_string("notification.save.started")
        )

    def process_save_completion(self):
        if self.save_result:
            success, message = self.save_result
            msg_type = "success" if success else "error"
            self.game.notification_manager.add_notification(message, msg_type=msg_type)
            self.save_result = None

    def _load_work(self):
        message_prefix = "debug" if self.game.show_debug_info else "default"
        try:
            with open(SAVE_FILE_NAME, "r") as f:
                load_data = json.load(f)
            self.load_result = (True, load_data)
        except FileNotFoundError:
            message = self.game.lang_manager.get_string(f"notification.load.error.not_found.{message_prefix}", filename=SAVE_FILE_NAME)
            self.load_result = (False, message)
        except json.JSONDecodeError as e:
            message = self.game.lang_manager.get_string(f"notification.load.error.decode.{message_prefix}", filename=SAVE_FILE_NAME, error=str(e))
            self.load_result = (False, message)
        except Exception as e:
            message = self.game.lang_manager.get_string(f"notification.load.error.unexpected.{message_prefix}", error=str(e))
            self.load_result = (False, message)
            traceback.print_exc()

    @property
    def is_loading(self):
        return self.load_thread and self.load_thread.is_alive()

    def load_game_state(self):
        if self.is_loading:
            return # Already loading

        self.load_result = None
        self.load_thread = threading.Thread(target=self._load_work)
        self.load_thread.start()
        self.game.set_loading_state(True)

    def process_load_completion(self):
        if not self.load_result:
            return

        success, data_or_message = self.load_result
        self.load_result = None # Reset
        self.game.set_loading_state(False)

        if not success:
            self.game.notification_manager.add_notification(data_or_message, msg_type="error")
            return

        load_data = data_or_message
        try:
            self.game.camera_x = load_data.get("camera_x", 0)
            self.game.camera_y = load_data.get("camera_y", 0)
            self.game.se_on = load_data.get("se_on", True)
            self.game.bgm_on = load_data.get("bgm_on", True)

            self.game.world_manager.world_seed_main = load_data.get("world_seed_main", px.rndi(1, 2**31 - 1))
            self.game.world_manager.world_seed_ore = load_data.get("world_seed_ore", px.rndi(1, 2**31 - 1))
            px.nseed(self.game.world_manager.world_seed_main)
            px.rseed(self.game.world_manager.world_seed_main)

            loaded_lang_code = load_data.get("current_language", DEFAULT_LANGUAGE)
            if self.game.lang_manager.set_language(loaded_lang_code):
                self.game.current_language_code = loaded_lang_code
            else:
                self.game.current_language_code = self.game.lang_manager.current_lang_code
            self.game.update_window_title()

            loaded_gen_chunk_coords_list = load_data.get("generated_chunk_coords", [])
            loaded_gen_chunk_coords_set = set()
            for coord_pair_list in loaded_gen_chunk_coords_list:
                loaded_gen_chunk_coords_set.add(tuple(coord_pair_list))
            loaded_modified_chunks_data = load_data.get("modified_chunks", [])

            self.game.world_manager.regenerate_world_from_chunks_and_apply_mods(loaded_gen_chunk_coords_set, loaded_modified_chunks_data)

            self.game.particle_manager.active_particles = []
            self.game._initial_block_generation_done = True

            self.game.world_manager.generate_visible_chunks()
            message_prefix = "debug" if self.game.show_debug_info else "default"
            self.game.notification_manager.add_notification(
                self.game.lang_manager.get_string(f"notification.load.success.{message_prefix}", filename=SAVE_FILE_NAME),
                msg_type="success")
            self.game.on_title_screen = False;
            self.game.is_menu_visible = False
        except Exception as e:
            message_prefix = "debug" if self.game.show_debug_info else "default"
            self.game.notification_manager.add_notification(
                self.game.lang_manager.get_string(f"notification.load.error.unexpected.{message_prefix}", error=str(e)),
                msg_type="error"
            )
            traceback.print_exc()

class InputHandler:
    def __init__(self, game):
        self.game = game
        self._key_pressed_start_time = {}
        self._key_last_repeat_action_time = {}

    def process_inputs(self):
        if self.game.on_title_screen:
            return

        if px.btnr(px.KEY_ESCAPE):
            self.game.is_menu_visible = not self.game.is_menu_visible
            if not self.game.is_menu_visible:
                self.game.game_menu.is_lang_dropdown_open = False

        if self.game.is_menu_visible:
            self.game.game_menu.handle_input()
        else:
            self._handle_debug_keys()
            self.handle_camera_movement()
            self._handle_mouse_clicks()

    def _handle_mouse_clicks(self):
        if px.btnp(px.MOUSE_BUTTON_LEFT):
            world_mouse_x = px.mouse_x + self.game.camera_x
            world_mouse_y = px.mouse_y + self.game.camera_y

            clicked_block = self.game.world_manager.get_block_at_world_coords(world_mouse_x, world_mouse_y)
            if clicked_block:
                chunk_x, chunk_y = world_to_chunk_coords(clicked_block.x, clicked_block.y)
                if (chunk_x, chunk_y) in self.game.world_manager.chunks:
                    self.game.world_manager.chunks[(chunk_x, chunk_y)].mark_as_modified_in_session()

                new_particles = clicked_block.handle_click()
                if new_particles:
                    self.game.particle_manager.add_particles(new_particles)
                    self.game.play_se(0, 1)  # Play break sound
                else:
                    self.game.play_se(0, 0)  # Play damage sound

    def _handle_debug_keys(self):
        if px.btnp(px.KEY_F3):
            for key in self.game.combination_keys:
                self.game.combination_key_pressed_during_f3[key] = False

        if px.btn(px.KEY_F3):
            for key in self.game.combination_keys:
                if px.btnp(key):
                    self.game.combination_key_pressed_during_f3[key] = True

        for key in self.game.combination_keys:
            if px.btnr(key) and px.btn(px.KEY_F3):
                if key == px.KEY_B:
                    self.game.show_debug_blocks = not self.game.show_debug_blocks

        if px.btnr(px.KEY_F3):
            combination_occurred = any(self.game.combination_key_pressed_during_f3.values())
            if not combination_occurred:
                self.game.show_debug_info = not self.game.show_debug_info

    def handle_camera_movement(self):
        camera_moved_flag = False
        current_time = time.time()

        key_directions = {
            px.KEY_W: (0, -1, 'W'), px.KEY_A: (-1, 0, 'A'),
            px.KEY_S: (0, 1, 'S'),  px.KEY_D: (1, 0, 'D')
        }

        for key_code, (dx_mult, dy_mult, key_char) in key_directions.items():
            base_speed = self.game.CAMERA_SPEED_FAST if px.btn(px.KEY_SHIFT) else self.game.CAMERA_SPEED_NORMAL
            moved_this_key = False
            if px.btnp(key_code):
                moved_this_key = True
                self._key_pressed_start_time[key_char] = current_time
                self._key_last_repeat_action_time[key_char] = current_time
            elif px.btn(key_code):
                if key_char in self._key_pressed_start_time:
                    if current_time - self._key_pressed_start_time[key_char] >= self.game.CAMERA_KEY_REPEAT_DELAY_INITIAL:
                        if current_time - self._key_last_repeat_action_time[key_char] >= self.game.CAMERA_KEY_REPEAT_INTERVAL:
                            moved_this_key = True
                            self._key_last_repeat_action_time[key_char] = current_time
            else:
                if key_char in self._key_pressed_start_time:
                    del self._key_pressed_start_time[key_char]
                if key_char in self._key_last_repeat_action_time:
                    del self._key_last_repeat_action_time[key_char]

            if moved_this_key:
                self.game.camera_x += dx_mult * base_speed
                self.game.camera_y += dy_mult * base_speed
                camera_moved_flag = True

        if camera_moved_flag:
            self.game.world_manager.generate_visible_chunks()
