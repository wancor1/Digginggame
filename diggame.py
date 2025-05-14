import pyxel as px
import random
import math
import time
import os
import json
import traceback
import PyxelUniversalFont as puf

# TODO: 鉱石の追加 [鉄、コバルト、銅、金、銀、ダイヤ]
# TODO: インベントリの追加
# TODO: マップの追加
# TODO: バイオームの追加
# TODO: 鉱石の高さによる出現頻度の変化
# TODO: 石の高さによる硬度の変化
# TODO: アップグレード等の追加
# TODO: 横幅50ブロック カメラの移動可能範囲を50+画面横幅
# TODO: 掘れる範囲の制限

SCREEN_WIDTH = 160
SCREEN_HEIGHT = 120
BLOCK_SIZE = 8
SPRITE_BANK_GAME = 0
SPRITE_BANK_UI = 1

SPRITE_SELECT_NORMAL = (SPRITE_BANK_GAME, 0, 0, 8, 8, 0)
SPRITE_SELECT_LARGE = (SPRITE_BANK_GAME, 0, 8, 10, 10, 0)
SPRITE_CURSOR = (SPRITE_BANK_GAME, 16, 0, 8, 8, 0)

SPRITE_BLOCK_ERROR = (SPRITE_BANK_UI, 8, 0, 8, 8, 1)
SPRITE_BLOCK_DIRT = (SPRITE_BANK_UI, 8, 0, 8, 8, 0)
SPRITE_BLOCK_GRASS = (SPRITE_BANK_UI, 16, 0, 8, 8, 0)
SPRITE_BLOCK_STONE = (SPRITE_BANK_UI, 8, 8, 8, 8, 0)
SPRITE_BLOCK_COAL = (SPRITE_BANK_UI, 16, 8, 8, 8, 1)

SPRITE_BREAK_ANIM_BANK = SPRITE_BANK_UI
SPRITE_BREAK_ANIM_U = 0
SPRITE_BREAK_ANIM_V_START = 0
SPRITE_BREAK_ANIM_COLKEY = 11

COLOR_BUTTON_BG = 13
COLOR_BUTTON_BORDER = 7
COLOR_BUTTON_TEXT = 7
COLOR_BUTTON_PRESSED_BG = 10
COLOR_BUTTON_PRESSED_TEXT = 7

MENU_ITEM_HEIGHT = 10
MENU_PADDING = 5
CHECKBOX_SIZE = 7
CHECKBOX_TEXT_GAP = 3
DROPDOWN_ARROW_WIDTH = 8
DROPDOWN_ARROW_HEIGHT = 5

SAVE_FILE_NAME = "savegame.json"

DEFAULT_FONT_PATH_CANDIDATES = [
    "misaki_gothic.ttf",
    "misaki_gothic2.ttf",
    "misaki_mincho.ttf"
]
LANG_FOLDER = "lang"
DEFAULT_LANGUAGE = "en_us"
FONT_SIZE = 8

def estimate_text_width(text):
    estimated_width = 0
    for char in text:
        if char == '\n':
            continue
        code_point = ord(char)
        if code_point < 128 or (0xFF61 <= code_point <= 0xFF9F):
            estimated_width += FONT_SIZE / 2
        else:
            estimated_width += FONT_SIZE
    return estimated_width

def calculate_text_center_position(box_width, box_height, text_content):
    if not text_content:
        return box_width / 2, (box_height - FONT_SIZE) / 2

    text_width_pixels = estimate_text_width(text_content)
    text_y = (box_height - FONT_SIZE) / 2
    text_x = (box_width - text_width_pixels) / 2
    return text_x, text_y

def numbers_to_notes(number_list):
    note_names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']

    result = ''
    for number in number_list:
        note_index = number % 12
        note_name = note_names[note_index]
        octave = number // 12

        result += note_name + str(octave)
    return result

class LanguageManager:
    def __init__(self, lang_folder=LANG_FOLDER, default_lang=DEFAULT_LANGUAGE):
        self.lang_folder = lang_folder
        self.languages = {}
        self.translations = {}
        self.current_lang_code = default_lang
        self._discover_languages()

        if not self.languages:
            print(f"No language files found in '{self.lang_folder}'. Attempting to create defaults.")
            self._create_default_lang_files_if_missing()
            self._discover_languages()

        if not self.load_language(self.current_lang_code):
            if self.languages:
                fallback_lang = list(self.languages.keys())[0]
                print(f"Failed to load default language '{default_lang}'. Falling back to '{fallback_lang}'.")
                self.load_language(fallback_lang)
            else:
                print(f"CRITICAL: No languages available even after attempting to create defaults.")
                self.translations = {"error_no_lang": "No languages loaded!"}

    def _create_default_lang_files_if_missing(self):
        if not os.path.exists(self.lang_folder):
            try:
                os.makedirs(self.lang_folder)
                print(f"Created language folder: {self.lang_folder}")
            except OSError as e:
                print(f"Error creating language folder {self.lang_folder}: {e}")
                return

        en_path = os.path.join(self.lang_folder, "en_us.json")
        if not os.path.exists(en_path):
            en_data = {
                "_metadata": {
                    "display_name": "English"
                },
                "game_title": "Digging Game Reforged",
                "menu_title": "MENU",
                "menu_se": "Sound Effects",
                "menu_music": "Music",
                "menu_language": "Language",
                "menu_save": "Save Game",
                "menu_load": "Load Game",
                "menu_quit": "Quit Game",
                "start_button": "Start",
                "start_button_pressed": "Click!",
                "debug_fps": "FPS: {fps}",
                "debug_cam": "Cam:({cam_x},{cam_y})",
                "debug_blk_pcl": "Blk:{blk_count} Pcl:{pcl_count}",
                "debug_sel": "Sel:{is_selected}",
                "save_success": "Game saved to {filename}",
                "save_error_write": "Error saving game: Could not write to file {filename}. {error}",
                "save_error_unexpected": "An unexpected error occurred during saving: {error}",
                "load_success": "Game loaded from {filename}",
                "load_error_not_found": "Save file not found: {filename}",
                "load_error_decode": "Error decoding save file ({filename}): Invalid JSON format. {error}",
                "load_error_unexpected": "An unexpected error occurred during loading: {error}"
            }
            try:
                with open(en_path, "w", encoding="utf-8") as f:
                    json.dump(en_data, f, ensure_ascii=False, indent=2)
                print(f"Created default language file: {en_path}")
            except IOError as e:
                print(f"Error creating default language file {en_path}: {e}")

    def _discover_languages(self):
        self.languages = {}
        if not os.path.isdir(self.lang_folder):
            print(f"Language folder '{self.lang_folder}' not found.")
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
                except Exception as e:
                    print(f"Could not read metadata from {filename}: {e}")
                self.languages[lang_code] = {"display_name": display_name, "path": file_path}

    def get_available_languages(self):
        return {code: data["display_name"] for code, data in self.languages.items()}

    def load_language(self, lang_code):
        if lang_code not in self.languages:
            print(f"Language code '{lang_code}' not found in available languages.")
            return False

        lang_file_path = self.languages[lang_code]["path"]
        try:
            with open(lang_file_path, "r", encoding="utf-8") as f:
                self.translations = json.load(f)
            self.current_lang_code = lang_code
            print(f"Loaded language: {lang_code} ('{self.languages[lang_code]['display_name']}') from {lang_file_path}")
            return True
        except FileNotFoundError:
            print(f"Language file not found: {lang_file_path}")
        except json.JSONDecodeError as e:
            print(f"Error decoding language file {lang_file_path}: {e}")
        except Exception as e:
            print(f"Unexpected error loading language {lang_code} from {lang_file_path}: {e}")

        self.translations = {"error_load_failed": f"Failed to load {lang_code}"}
        return False

    def get_string(self, key, **kwargs):
        val = self.translations.get(key, f"{key}")
        if kwargs:
            try:
                return val.format(**kwargs)
            except KeyError as e:
                print(f"Warning: Missing key '{e}' in format string for '{key}'")
                return val
        return val

    def set_language(self, lang_code):
        if self.load_language(lang_code):
            return True
        return False

class Toast:
    def __init__(self, text1='a', text2=None, duration=3):
        self.text1 = text1
        self.text2 = text2
        self.duration = duration
        self.elapsed_time = 0
        self.start_time = time.time()

    def get_height(self):
        return px.FONT_HEIGHT * (2 if self.text2 else 1) + 2

    def set_message(self, text1, text2=None):
        self.text1 = text1
        self.text2 = text2
        self.start_time = time.time()

    def update(self):
        self.elapsed_time = time.time() - self.start_time
        if self.elapsed_time >= self.duration:
            return False
        return True

    def draw(self, offset_y=0):
        if self.text2 is None:
            text_x = SCREEN_WIDTH - ((px.FONT_WIDTH) * len(self.text1))
        else:
            text_x = SCREEN_WIDTH - ((px.FONT_WIDTH) * max(len(self.text1), len(self.text2)))

        text_y = 1 + offset_y
        px.rect(text_x - 2, text_y - 1, SCREEN_WIDTH - text_x + 2, px.FONT_HEIGHT * (2 if self.text2 else 1) + 1, 5)
        px.rectb(text_x - 2, text_y - 1, SCREEN_WIDTH - text_x + 2, px.FONT_HEIGHT * (2 if self.text2 else 1) + 1, 1)
        #writer.draw(text_x, text_y, self.text1, FONT_SIZE, 7)
        if self.text2 is not None:
            print("", end="")
            #writer.draw(text_x, text_y + px.FONT_HEIGHT, self.text2, FONT_SIZE, 7)

class ToastManager:
    def __init__(self):
        self.toasts = []

    def add(self, text1, text2=None, duration=3):
        self.toasts.append(Toast(text1, text2, duration))

    def update(self):
        self.toasts = [t for t in self.toasts if t.update()]

    def draw(self):
        offset_y = 0
        for toast in self.toasts:
            toast.draw(offset_y=offset_y)
            offset_y += toast.get_height()

class SelectBlock:
    def __init__(self):
        self._selection_effect_start_time = 0
        self._is_effect_currently_active = False

    def update_selection_status(self, is_mouse_over_a_block):
        if is_mouse_over_a_block:
            if not self._is_effect_currently_active:
                self._is_effect_currently_active = True
                self._selection_effect_start_time = time.time()
        else:
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
            px.blt(grid_aligned_x, grid_aligned_y, *SPRITE_SELECT_NORMAL)

class Particle:
    GRAVITY = 0.19
    MAX_LIFESPAN_ON_GROUND_SEC = 5.0
    PARTICLE_SPEED_MIN = 20 / 60
    PARTICLE_SPEED_MAX = 60 / 60
    BOUNCE_DAMPENING_X = -0.4
    FRICTION_ON_GROUND = 0.85

    def __init__(self, x_start, y_start, block_max_hardness):
        self.x = x_start + BLOCK_SIZE / 2
        self.y = y_start + BLOCK_SIZE / 2

        angle = random.uniform(0, 2 * math.pi)
        speed = random.uniform(self.PARTICLE_SPEED_MIN, self.PARTICLE_SPEED_MAX)

        self.vx = math.cos(angle) * speed
        self.vy = math.sin(angle) * speed - 1.5

        self.alive = True
        self.time_landed = None

        if block_max_hardness <= 5:
            self.color = 9 if random.random() < 0.9 else 13
        elif block_max_hardness <= 10:
            self.color = 13 if random.random() < 0.9 else 6
        else:
            self.color = 0

    def update(self, collidable_blocks):
        if not self.alive:
            return

        self.vy += self.GRAVITY

        self.x += self.vx
        for block in collidable_blocks:
            if block.x <= self.x < block.x + BLOCK_SIZE and \
               block.y <= self.y < block.y + BLOCK_SIZE:
                if self.vx > 0:
                    self.x = block.x - 0.1
                else:
                    self.x = block.x + BLOCK_SIZE + 0.1
                self.vx *= self.BOUNCE_DAMPENING_X
                break

        self.y += self.vy
        is_on_ground_this_frame = False
        for block in collidable_blocks:
            if block.x <= self.x < block.x + BLOCK_SIZE and \
               block.y <= self.y < block.y + BLOCK_SIZE:
                if self.vy > 0:
                    self.y = block.y - 0.1
                    self.vy = 0
                    self.vx *= self.FRICTION_ON_GROUND
                    is_on_ground_this_frame = True
                elif self.vy < 0:
                    self.y = block.y + BLOCK_SIZE + 0.1
                    self.vy = 0
                break

        if is_on_ground_this_frame:
            if self.time_landed is None:
                self.time_landed = time.time()
            elif time.time() - self.time_landed > self.MAX_LIFESPAN_ON_GROUND_SEC:
                self.alive = False
        else:
            self.time_landed = None

        if self.y > SCREEN_HEIGHT + BLOCK_SIZE * 5:
            self.alive = False

    def draw(self):
        if self.alive:
            px.pset(self.x, self.y, self.color)

class Block:
    HARDNESS_MIN = 3
    HARDNESS_MAX = 10
    NOISE_SCALE_HARDNESS = 0.01
    NOISE_SCALE_ORE = 0.04
    ORE_THRESHOLD = 0.4

    SURFACE_Y_LEVEL_IN_BLOCKS = 7

    PARTICLES_MIN_ON_BREAK = 5
    PARTICLES_MAX_ON_BREAK = 15
    PARTICLES_MEAN_ON_BREAK = 10
    PARTICLES_STDDEV_ON_BREAK = 2

    def __init__(self, x, y, world_seed_noise, world_seed_ore):
        self.x = x
        self.y = y
        self.is_broken = False

        px.nseed(world_seed_noise)

        noise_val_hardness = px.noise(self.x * self.NOISE_SCALE_HARDNESS,
                                      self.y * self.NOISE_SCALE_HARDNESS,
                                      0)
        self.max_hp = int(math.floor((self.HARDNESS_MAX - self.HARDNESS_MIN) * abs(noise_val_hardness)) + self.HARDNESS_MIN)
        self.current_hp = self.max_hp

        if self.y // BLOCK_SIZE < self.SURFACE_Y_LEVEL_IN_BLOCKS:
            self.is_broken = True
            self.sprite_info = None
            return

        if self.y // BLOCK_SIZE == self.SURFACE_Y_LEVEL_IN_BLOCKS:
            self.sprite_info = SPRITE_BLOCK_GRASS
        elif self.max_hp <= 5:
            self.sprite_info = SPRITE_BLOCK_DIRT
        elif self.max_hp <= 10:
            self.sprite_info = SPRITE_BLOCK_STONE

            px.nseed(world_seed_ore)
            noise_val_ore = px.noise(self.x * self.NOISE_SCALE_ORE,
                                     self.y * self.NOISE_SCALE_ORE,
                                     0.5)
            if noise_val_ore >= self.ORE_THRESHOLD:
                self.sprite_info = SPRITE_BLOCK_COAL
        else:
            self.sprite_info = SPRITE_BLOCK_STONE

    def _get_break_animation_frame_index(self):
        if self.current_hp == self.max_hp or self.is_broken:
            return 0

        damage_taken = self.max_hp - self.current_hp
        num_visual_break_stages = 5

        damage_ratio = damage_taken / self.max_hp

        frame_index = math.ceil(damage_ratio * num_visual_break_stages)
        return max(1, min(frame_index, num_visual_break_stages))

    def draw(self, show_debug_info, font_writer: puf.Writer):
        if self.is_broken or self.sprite_info is None:
            return

        px.blt(self.x, self.y, *self.sprite_info)

        break_anim_idx = self._get_break_animation_frame_index()
        if break_anim_idx > 0:
            anim_v = SPRITE_BREAK_ANIM_V_START + (break_anim_idx - 1) * BLOCK_SIZE
            px.blt(self.x, self.y,
                   SPRITE_BREAK_ANIM_BANK,
                   SPRITE_BREAK_ANIM_U,
                   anim_v,
                   BLOCK_SIZE, BLOCK_SIZE,
                   SPRITE_BREAK_ANIM_COLKEY)

        if show_debug_info:
            hp_text = f'{self.current_hp}'
            text_x_offset, text_y_offset = calculate_text_center_position(BLOCK_SIZE, BLOCK_SIZE, hp_text)

            if self.current_hp != self.max_hp:
                bar_width_pixels = (self.current_hp / self.max_hp) * (BLOCK_SIZE - 2)
                px.rect(self.x + 1, self.y + BLOCK_SIZE - 2, BLOCK_SIZE - 2, 1, 13)
                px.rect(self.x + 1, self.y + BLOCK_SIZE - 2, bar_width_pixels, 1, 3)
                font_writer.draw(self.x + text_x_offset, self.y + text_y_offset -1, hp_text, FONT_SIZE, 7)
            else:
                font_writer.draw(self.x + text_x_offset, self.y + text_y_offset, hp_text, FONT_SIZE, 7)

    def handle_click(self):
        if self.is_broken:
            return []

        self.current_hp -= 1
        created_particles = []

        if self.current_hp <= 0:
            self.is_broken = True
            game_instance.play_se(0, 1)

            num_particles = int(min(self.PARTICLES_MAX_ON_BREAK,
                                    max(self.PARTICLES_MIN_ON_BREAK,
                                        random.gauss(self.PARTICLES_MEAN_ON_BREAK, self.PARTICLES_STDDEV_ON_BREAK))))
            created_particles = [Particle(self.x, self.y, self.max_hp) for _ in range(num_particles)]
        else:
            game_instance.play_se(0, 0)

        return created_particles

    def is_mouse_over(self, world_mouse_x, world_mouse_y):
        if self.is_broken:
            return False
        return (self.x <= world_mouse_x < self.x + BLOCK_SIZE and
                self.y <= world_mouse_y < self.y + BLOCK_SIZE)

    def to_save_data(self):
        return {"x": self.x, "y": self.y, "current_hp": self.current_hp}


class ButtonBox:
    def __init__(self, font_writer: puf.Writer, lang_manager: LanguageManager):
        self.font = font_writer
        self.lang_manager = lang_manager

    def _get_mouse_status_on_button(self, x, y, w, h):
        is_hover = (x <= px.mouse_x <= x + w - 1 and
                    y <= px.mouse_y <= y + h - 1)
        is_pressed_on_button = px.btn(px.MOUSE_BUTTON_LEFT) and is_hover
        is_released_on_button = px.btnr(px.MOUSE_BUTTON_LEFT) and is_hover
        return is_pressed_on_button, is_hover, is_released_on_button

    def draw_button(self, x, y, w, h, text_key='text', pressed_text_key='press'):
        is_being_pressed, _, is_released_on = self._get_mouse_status_on_button(x, y, w, h)

        current_text_key = pressed_text_key if is_being_pressed else text_key
        current_text_str = self.lang_manager.get_string(current_text_key)

        bg_color = COLOR_BUTTON_PRESSED_BG if is_being_pressed else COLOR_BUTTON_BG

        px.rect(x, y, w, h, bg_color)
        px.rectb(x, y, w, h, COLOR_BUTTON_BORDER)

        if not is_being_pressed:
            px.line(x + w -1 , y + 1, x + w -1, y + h - 2, 0)
            px.line(x + 1, y + h - 1, x + w - 2, y + h - 1, 0)

        text_x_offset, text_y_offset = calculate_text_center_position(w, h, current_text_str)
        self.font.draw(x + text_x_offset, y + text_y_offset, current_text_str, FONT_SIZE, COLOR_BUTTON_TEXT)

        return is_released_on

    def draw_static_box(self, x, y, w, h, text_key='text'):
        text_str = self.lang_manager.get_string(text_key)
        px.rect(x,y, w, h, COLOR_BUTTON_BG)
        px.rectb(x,y, w, h, COLOR_BUTTON_BORDER)
        text_x_offset, text_y_offset = calculate_text_center_position(w, h, text_str)
        self.font.draw(x + text_x_offset, y + text_y_offset, text_str, FONT_SIZE, COLOR_BUTTON_TEXT)

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
            {"key": "menu_se", "type": "checkbox", "setting_attr": "se_on"},
            {"key": "menu_music", "type": "checkbox", "setting_attr": "bgm_on"},
            {"key": "menu_language", "type": "dropdown"},
            {"key": "menu_save", "type": "button", "action_label": "Save Game"},
            {"key": "menu_load", "type": "button", "action_label": "Load Game"},
            {"key": "menu_quit", "type": "button", "action_label": "Quit Game"}
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

    def _draw_language_dropdown(self, x, y, item_width):
        self.lang_dropdown_options_rects = []

        current_lang_display_name = self.lang_manager.get_available_languages().get(
            self.lang_manager.current_lang_code, self.lang_manager.current_lang_code
        )

        px.rectb(x, y, item_width, MENU_ITEM_HEIGHT, 0)
        self.font.draw(x + MENU_PADDING, y + (MENU_ITEM_HEIGHT - FONT_SIZE) // 2, current_lang_display_name, FONT_SIZE, 0)
        px.tri(x + item_width - DROPDOWN_ARROW_WIDTH - 2, y + (MENU_ITEM_HEIGHT - DROPDOWN_ARROW_HEIGHT)//2,
               x + item_width - 2 - DROPDOWN_ARROW_WIDTH//2, y + (MENU_ITEM_HEIGHT + DROPDOWN_ARROW_HEIGHT)//2,
               x + item_width - 2, y + (MENU_ITEM_HEIGHT - DROPDOWN_ARROW_HEIGHT)//2, 0)

        self.lang_dropdown_trigger_rect = (x, y, item_width, MENU_ITEM_HEIGHT)

        if self.is_lang_dropdown_open:
            options_y = y + MENU_ITEM_HEIGHT
            available_langs = self.lang_manager.get_available_languages()

            max_option_width = item_width
            for lang_code, display_name in available_langs.items():
                max_option_width = max(max_option_width, estimate_text_width(display_name) + MENU_PADDING * 2)

            list_bg_x = x
            list_bg_y = options_y
            list_bg_w = max_option_width
            list_bg_h = len(available_langs) * MENU_ITEM_HEIGHT

            px.rect(list_bg_x, list_bg_y, list_bg_w, list_bg_h, 13)
            px.rectb(list_bg_x, list_bg_y, list_bg_w, list_bg_h, 0)

            for lang_code, display_name in available_langs.items():
                option_rect = (list_bg_x, options_y, list_bg_w, MENU_ITEM_HEIGHT)
                self.lang_dropdown_options_rects.append((*option_rect, lang_code))

                if option_rect[0] <= px.mouse_x < option_rect[0] + option_rect[2] and \
                   option_rect[1] <= px.mouse_y < option_rect[1] + option_rect[3]:
                    px.rect(option_rect[0], option_rect[1], option_rect[2], option_rect[3], 1)

                self.font.draw(list_bg_x + MENU_PADDING, options_y + (MENU_ITEM_HEIGHT - FONT_SIZE) // 2, display_name, FONT_SIZE, 0)
                options_y += MENU_ITEM_HEIGHT

    def handle_input(self):
        if not self.game.is_menu_visible:
            self.is_lang_dropdown_open = False
            return

        clicked_outside_dropdown = False
        if self.is_lang_dropdown_open and px.btnp(px.MOUSE_BUTTON_LEFT):
            clicked_on_option = False
            for opt_x, opt_y, opt_w, opt_h, lang_code in self.lang_dropdown_options_rects:
                if opt_x <= px.mouse_x < opt_x + opt_w and \
                   opt_y <= px.mouse_y < opt_y + opt_h:
                    self.lang_manager.set_language(lang_code)
                    self.game.current_language_code = lang_code
                    self.game.update_window_title()
                    self.is_lang_dropdown_open = False
                    clicked_on_option = True
                    break
            if not clicked_on_option:
                if self.lang_dropdown_trigger_rect and \
                   not (self.lang_dropdown_trigger_rect[0] <= px.mouse_x < self.lang_dropdown_trigger_rect[0] + self.lang_dropdown_trigger_rect[2] and \
                        self.lang_dropdown_trigger_rect[1] <= px.mouse_y < self.lang_dropdown_trigger_rect[1] + self.lang_dropdown_trigger_rect[3]):
                    clicked_outside_dropdown = True

        if px.btnp(px.MOUSE_BUTTON_LEFT) and (not self.is_lang_dropdown_open or clicked_outside_dropdown):
            current_y = self.y + MENU_ITEM_HEIGHT + MENU_PADDING * 2
            for item_def in self.menu_items_def:
                item_x = self.x + MENU_PADDING
                item_w = self.width - MENU_PADDING * 2

                # アイテム全体のクリック領域
                item_rect = (item_x, current_y, item_w, MENU_ITEM_HEIGHT)
                mouse_on_this_item = (item_rect[0] <= px.mouse_x < item_rect[0] + item_rect[2] and \
                                      item_rect[1] <= px.mouse_y < item_rect[1] + item_rect[3])

                if mouse_on_this_item:
                    if item_def["type"] == "checkbox":
                        is_checked_value = getattr(self.game, item_def["setting_attr"])
                        setattr(self.game, item_def["setting_attr"], not is_checked_value)
                        break
                    elif item_def["type"] == "dropdown":
                        if self.lang_dropdown_trigger_rect and \
                           self.lang_dropdown_trigger_rect[0] <= px.mouse_x < self.lang_dropdown_trigger_rect[0] + self.lang_dropdown_trigger_rect[2] and \
                           self.lang_dropdown_trigger_rect[1] <= px.mouse_y < self.lang_dropdown_trigger_rect[1] + self.lang_dropdown_trigger_rect[3]:
                            self.is_lang_dropdown_open = not self.is_lang_dropdown_open
                            break
                    elif item_def["type"] == "button":
                        pass
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

        menu_title_str = self.lang_manager.get_string("menu_title")
        title_x_offset, title_y_offset = calculate_text_center_position(self.width, MENU_ITEM_HEIGHT, menu_title_str)
        self.font.draw(self.x + title_x_offset, self.y + MENU_PADDING + title_y_offset, menu_title_str, FONT_SIZE, 0)

        current_y = self.y + MENU_ITEM_HEIGHT + MENU_PADDING * 2

        dropdown_draw_params = None # ドロップダウンリストの描画を後回しにするためのパラメータ

        for item_def in self.menu_items_def:
            item_x = self.x + MENU_PADDING
            item_w = self.width - MENU_PADDING * 2
            label_str = self.lang_manager.get_string(item_def["key"])

            if item_def["type"] == "checkbox":
                is_checked_value = getattr(self.game, item_def["setting_attr"])
                self._draw_checkbox(item_x, current_y, label_str, is_checked_value)
            elif item_def["type"] == "dropdown":
                lang_label_prefix = self.lang_manager.get_string(item_def["key"]) + ":"
                self.font.draw(item_x, current_y + (MENU_ITEM_HEIGHT - FONT_SIZE) // 2, lang_label_prefix, FONT_SIZE, 0)

                dropdown_button_x = item_x + estimate_text_width(lang_label_prefix) + MENU_PADDING
                dropdown_button_w = item_w - (estimate_text_width(lang_label_prefix) + MENU_PADDING)

                # ドロップダウンのトリガー部分だけを描画
                current_lang_display_name = self.lang_manager.get_available_languages().get(
                    self.lang_manager.current_lang_code, self.lang_manager.current_lang_code
                )
                px.rectb(dropdown_button_x, current_y, dropdown_button_w, MENU_ITEM_HEIGHT, 0)
                self.font.draw(dropdown_button_x + MENU_PADDING, current_y + (MENU_ITEM_HEIGHT - FONT_SIZE) // 2, current_lang_display_name, FONT_SIZE, 0)
                px.tri(dropdown_button_x + dropdown_button_w - DROPDOWN_ARROW_WIDTH - 2, current_y + (MENU_ITEM_HEIGHT - DROPDOWN_ARROW_HEIGHT)//2,
                       dropdown_button_x + dropdown_button_w - 2 - DROPDOWN_ARROW_WIDTH//2, current_y + (MENU_ITEM_HEIGHT + DROPDOWN_ARROW_HEIGHT)//2,
                       dropdown_button_x + dropdown_button_w - 2, current_y + (MENU_ITEM_HEIGHT - DROPDOWN_ARROW_HEIGHT)//2, 0)
                self.lang_dropdown_trigger_rect = (dropdown_button_x, current_y, dropdown_button_w, MENU_ITEM_HEIGHT)

                if self.is_lang_dropdown_open:
                    dropdown_draw_params = (dropdown_button_x, current_y, dropdown_button_w) # リスト描画用パラメータ

            elif item_def["type"] == "button":
                if self.button_handler.draw_button(item_x, current_y, item_w, MENU_ITEM_HEIGHT, item_def["key"], item_def["key"]):
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

            list_bg_x = dd_x
            list_bg_y = options_y_start
            list_bg_w = max_option_width
            list_bg_h = len(available_langs) * MENU_ITEM_HEIGHT

            px.rect(list_bg_x, list_bg_y, list_bg_w, list_bg_h, 13)
            px.rectb(list_bg_x, list_bg_y, list_bg_w, list_bg_h, 0)

            self.lang_dropdown_options_rects = []
            current_opt_y = options_y_start
            for lang_code, display_name in available_langs.items():
                option_rect_for_draw = (list_bg_x, current_opt_y, list_bg_w, MENU_ITEM_HEIGHT)
                self.lang_dropdown_options_rects.append((*option_rect_for_draw, lang_code))

                if option_rect_for_draw[0] <= px.mouse_x < option_rect_for_draw[0] + option_rect_for_draw[2] and \
                   option_rect_for_draw[1] <= px.mouse_y < option_rect_for_draw[1] + option_rect_for_draw[3]:
                    px.rect(option_rect_for_draw[0], option_rect_for_draw[1], option_rect_for_draw[2], option_rect_for_draw[3], 1)

                self.font.draw(list_bg_x + MENU_PADDING, current_opt_y + (MENU_ITEM_HEIGHT - FONT_SIZE) // 2, display_name, FONT_SIZE, 0)
                current_opt_y += MENU_ITEM_HEIGHT

        return self.selected_button_action

class DiggingGame:
    GAME_TITLE = 'Digging Game'
    GROUND_SURFACE_Y_BLOCK_INDEX = 7

    CAMERA_SPEED_NORMAL = 8
    CAMERA_SPEED_FAST = 16

    CAMERA_KEY_REPEAT_DELAY_INITIAL = 0.4
    CAMERA_KEY_REPEAT_INTERVAL = 0.05

    def __init__(self):
        px.init(SCREEN_WIDTH, SCREEN_HEIGHT, title="Digging Game", fps=60, quit_key=0)

        resource_file = 'sprite_sheet.pyxres'

        if os.path.exists(resource_file):
            try:
                print(f"Attempting to load {resource_file}...")
                px.load(resource_file)
                print(f"Successfully loaded {resource_file}.")
            except Exception as e:
                print(f"Warning: An error occurred while loading '{resource_file}': {e}")
                print("Creating dummy sprites as a fallback.")
                self._create_dummy_sprites_if_needed()
        else:
            print(f"Warning: Resource file '{resource_file}' not found.")
            print("Creating dummy sprites for basic functionality.")
            self._create_dummy_sprites_if_needed()

        self.font = puf.Writer("misaki_gothic.ttf")

        self.all_blocks = []
        self.active_particles = []
        self.generated_block_coordinates = set()

        self.select_block_highlighter = SelectBlock()
        self.lang_manager = LanguageManager()
        self.button_handler = ButtonBox(self.font, self.lang_manager)
        self.game_menu = GameMenu(self, self.lang_manager, self.font)
        self.toast_manager = ToastManager()

        self.current_language_code = self.lang_manager.current_lang_code
        self.update_window_title()

        self.on_title_screen = True
        self.is_menu_visible = False
        self.show_debug_overlay = False

        self.camera_x = 0
        self.camera_y = 0

        self._key_pressed_start_time = {}
        self._key_last_repeat_action_time = {}

        self.world_seed_main = px.rndi(1, 2**31 - 1)
        self.world_seed_ore = px.rndi(1, 2**31 - 1)
        px.nseed(self.world_seed_main)
        px.rseed(self.world_seed_main)

        self._initial_block_generation_done = False
        self._is_mouse_over_any_block = False

        self.current_hp = 10
        self.max_hp = 10

        self.frame_count = 0
        self.start_time = time.time()
        self.current_fps = 0
        self.last_calc_time = time.time()
        self.last_calc_frame = px.frame_count

        self.se_on = True
        self.bgm_on = True

    def update_window_title(self):
        self.game_title = self.lang_manager.get_string("game_title")
        px.title = self.game_title

    def play_se(self, ch, snd, loop=False):
        if self.se_on:
            px.play(ch, snd, loop=loop)

    def play_bgm(self, ch, snd, loop=True):
        if self.bgm_on:
            px.play(ch, snd, loop=loop)
        else:
            px.stop(ch)

    def _handle_camera_movement(self):
        camera_moved = False
        current_time = time.time()

        key_directions = {
            px.KEY_W: (0, -1, 'W'), px.KEY_A: (-1, 0, 'A'),
            px.KEY_S: (0, 1, 'S'),  px.KEY_D: (1, 0, 'D')
        }

        for key_code, (dx_mult, dy_mult, key_char) in key_directions.items():
            base_speed = self.CAMERA_SPEED_FAST if px.btn(px.KEY_SHIFT) else self.CAMERA_SPEED_NORMAL

            if px.btnp(key_code):
                self.camera_x += dx_mult * base_speed
                self.camera_y += dy_mult * base_speed
                camera_moved = True
                self._key_pressed_start_time[key_char] = current_time
                self._key_last_repeat_action_time[key_char] = current_time
            elif px.btn(key_code):
                if key_char in self._key_pressed_start_time:
                    if current_time - self._key_pressed_start_time[key_char] >= self.CAMERA_KEY_REPEAT_DELAY_INITIAL:
                        if current_time - self._key_last_repeat_action_time[key_char] >= self.CAMERA_KEY_REPEAT_INTERVAL:
                            self.camera_x += dx_mult * base_speed
                            self.camera_y += dy_mult * base_speed
                            camera_moved = True
                            self._key_last_repeat_action_time[key_char] = current_time
            else:
                if key_char in self._key_pressed_start_time:
                    del self._key_pressed_start_time[key_char]
                if key_char in self._key_last_repeat_action_time:
                    del self._key_last_repeat_action_time[key_char]

        if camera_moved:
            self._generate_visible_blocks()

    def _update_game_logic(self):
        world_mouse_x = px.mouse_x + self.camera_x
        world_mouse_y = px.mouse_y + self.camera_y

        self._is_mouse_over_any_block = False
        for block in self.all_blocks:
            if block.is_mouse_over(world_mouse_x, world_mouse_y):
                self._is_mouse_over_any_block = True
                break
        self.select_block_highlighter.update_selection_status(self._is_mouse_over_any_block)

        if px.btnp(px.MOUSE_BUTTON_LEFT):
            for block in self.all_blocks:
                if block.is_mouse_over(world_mouse_x, world_mouse_y):
                    new_particles = block.handle_click()
                    self.active_particles.extend(new_particles)
                    break

        collidable_blocks_for_particles = [b for b in self.all_blocks if not b.is_broken]
        for particle in self.active_particles:
            particle.update(collidable_blocks_for_particles)

        self.active_particles = [p for p in self.active_particles if p.alive]

    def _handle_menu_action(self, action):
        if action == "Save Game":
            self.save_game_state()
        elif action == "Load Game":
            self.load_game_state()
        elif action == "Quit Game":
            self.save_game_state()
            px.quit()

    def save_game_state(self):
        active_blocks_data = []
        for block in self.all_blocks:
            active_blocks_data.append(block.to_save_data())

        save_data = {
            "camera_x": self.camera_x, "camera_y": self.camera_y,
            "se_on": self.se_on, "bgm_on": self.bgm_on,
            "world_seed_main": self.world_seed_main, "world_seed_ore": self.world_seed_ore,
            "generated_coords": list(self.generated_block_coordinates),
            "active_blocks_states": [b.to_save_data() for b in self.all_blocks],
            "current_language": self.lang_manager.current_lang_code
        }
        try:
            with open(SAVE_FILE_NAME, "w") as f:
                json.dump(save_data, f, indent=2)
            print(self.lang_manager.get_string("save_success", filename=SAVE_FILE_NAME))
            # TODO: ユーザーに「Saved!」というフィードバックを画面に表示する
        except IOError as e:
            print(self.lang_manager.get_string("save_error_write", filename=SAVE_FILE_NAME, error=e))
        except Exception as e:
            print(self.lang_manager.get_string("save_error_unexpected", error=e))
            traceback.print_exc()

    def load_game_state(self):
        try:
            with open(SAVE_FILE_NAME, "r") as f:
                load_data = json.load(f)

            self.camera_x = load_data.get("camera_x", 0)
            self.camera_y = load_data.get("camera_y", 0)
            self.se_on = load_data.get("se_on", True)
            self.bgm_on = load_data.get("bgm_on", True)

            self.world_seed_main = load_data.get("world_seed_main", px.rndi(1, 2**31 - 1))
            self.world_seed_ore = load_data.get("world_seed_ore", px.rndi(1, 2**31 - 1))
            px.nseed(self.world_seed_main)
            px.rseed(self.world_seed_main)

            loaded_lang_code = load_data.get("current_language", DEFAULT_LANGUAGE)
            if self.lang_manager.set_language(loaded_lang_code):
                self.current_language_code = loaded_lang_code
                self.update_window_title()
            else:
                self.current_language_code = self.lang_manager.current_lang_code
                self.update_window_title()

            self.all_blocks = []
            self.generated_block_coordinates = set(tuple(coord) for coord in load_data.get("generated_coords", []))
            block_hp_map = {(bd["x"], bd["y"]): bd["current_hp"] for bd in load_data.get("active_blocks_states", [])}

            for world_x, world_y in list(self.generated_block_coordinates):
                if world_y // BLOCK_SIZE < self.GROUND_SURFACE_Y_BLOCK_INDEX: continue
                block = Block(world_x, world_y, self.world_seed_main, self.world_seed_ore)
                if (world_x, world_y) in block_hp_map:
                    block.current_hp = block_hp_map[(world_x, world_y)]
                if block.current_hp <= 0: block.is_broken = True; block.current_hp = 0
                else: block.is_broken = False
                if not block.is_broken: self.all_blocks.append(block)

            self.active_particles = []
            self._initial_block_generation_done = True
            print(self.lang_manager.get_string("load_success", filename=SAVE_FILE_NAME))
            self.on_title_screen = False
            self.is_menu_visible = False
            # if self.bgm_on: self.play_bgm(BGM_CHANNEL, BGM_SOUND_ID) else: px.stop(BGM_CHANNEL)

        except FileNotFoundError:
            print(self.lang_manager.get_string("load_error_not_found", filename=SAVE_FILE_NAME))
            self.toast_manager.add('セーブデータが見つかりませんでした', '新規ゲームを開始します', 5)
        except json.JSONDecodeError as e:
            print(self.lang_manager.get_string("load_error_decode", filename=SAVE_FILE_NAME, error=e))
            self.toast_manager.add('セーブデータの読み込みに失敗しました', 'データが壊れている可能性があります', 5)
        except Exception as e:
            print(self.lang_manager.get_string("load_error_unexpected", error=e))
            traceback.print_exc()
            self.toast_manager.add('セーブデータの読み込みに失敗しました', '不明なエラーが発生しました', 5)

    def _create_dummy_sprites_if_needed(self):
        img_bank0 = px.Image(32, 16)
        img_bank0.rect(0, 0, 8, 8, px.COLOR_RED)
        img_bank0.rect(0, 8, 10, 10, px.COLOR_CYAN)
        img_bank0.rect(16, 0, 8, 8, px.COLOR_LIME)
        px.images[SPRITE_BANK_GAME] = img_bank0

        img_bank1 = px.Image(32, 64)
        img_bank1.rect(8, 0, 8, 8, px.COLOR_BROWN)
        img_bank1.rect(16, 0, 8, 4, px.COLOR_GREEN)
        img_bank1.rect(16, 4, 8, 4, px.COLOR_BROWN)
        img_bank1.rect(8, 8, 8, 8, px.COLOR_GRAY)
        img_bank1.rect(16, 8, 8, 8, px.COLOR_BLACK)

        for i in range(5):
            img_bank1.rect(SPRITE_BREAK_ANIM_U, SPRITE_BREAK_ANIM_V_START + i * 8, 8, 8, px.COLOR_WHITE if i % 2 == 0 else px.COLOR_GRAY)
            img_bank1.text(SPRITE_BREAK_ANIM_U + 1, SPRITE_BREAK_ANIM_V_START + i * 8 + 1, str(i+1), px.COLOR_BLACK)
        px.images[SPRITE_BANK_UI] = img_bank1
        print("Dummy sprites created.")

    def _generate_visible_blocks(self):

        min_bx = math.floor(self.camera_x / BLOCK_SIZE)
        max_bx = math.ceil((self.camera_x + SCREEN_WIDTH) / BLOCK_SIZE)

        min_by_camera = math.floor(self.camera_y / BLOCK_SIZE)
        max_by_camera = math.ceil((self.camera_y + SCREEN_HEIGHT) / BLOCK_SIZE)

        start_generation_by = max(min_by_camera, self.GROUND_SURFACE_Y_BLOCK_INDEX)

        for current_bx in range(min_bx, max_bx + 1):
            for current_by in range(start_generation_by, max_by_camera + 1):
                block_world_x, block_world_y = current_bx * BLOCK_SIZE, current_by * BLOCK_SIZE
                coord_tuple = (block_world_x, block_world_y)

                if coord_tuple not in self.generated_block_coordinates:
                    if current_by >= self.GROUND_SURFACE_Y_BLOCK_INDEX:
                        new_block = Block(block_world_x, block_world_y, self.world_seed_main, self.world_seed_ore)
                        if not new_block.is_broken:
                            self.all_blocks.append(new_block)
                            self.generated_block_coordinates.add(coord_tuple)

    def _calc_fps(self):
        current_time = time.time()
        elapsed_time = current_time - self.last_calc_time

        if elapsed_time >= 1.0:
            elapsed_frames = px.frame_count - self.last_calc_frame
            self.current_fps = elapsed_frames / elapsed_time
            self.last_calc_time = current_time
            self.last_calc_frame = px.frame_count

    def update(self):
        if self.on_title_screen:
            pass
        else:
            if px.btnp(px.KEY_ESCAPE):
                self.is_menu_visible = not self.is_menu_visible
                if not self.is_menu_visible:
                    self.game_menu.is_lang_dropdown_open = False

            if self.is_menu_visible:
                self.game_menu.handle_input()
            else:
                if px.btnp(px.KEY_F3):
                    self.show_debug_overlay = not self.show_debug_overlay
                self._handle_camera_movement()
                self._update_game_logic()
        self.toast_manager.update()

    def _draw_title_screen(self):
        px.cls(5)
        px.camera(0, 0)

        title_str = self.lang_manager.get_string("game_title")
        t_tw, t_th = calculate_text_center_position(SCREEN_WIDTH, SCREEN_HEIGHT, title_str)
        self.font.draw(t_tw + 1, t_th * 0.5 + 1, title_str, FONT_SIZE, 3)
        self.font.draw(t_tw, t_th * 0.5, title_str, FONT_SIZE, 11)

        button_w, button_h = 60, 10
        button_x = (SCREEN_WIDTH - button_w) / 2
        button_y = (SCREEN_HEIGHT - button_h) / 2 * 1.25
        self.toast_manager.draw()

        if not hasattr(self, 'title_button_handler'):
            self.title_button_handler = ButtonBox(self.font, self.lang_manager)

        if self.button_handler.draw_button(button_x, button_y, button_w, button_h, 'Start', 'Click!'):
            self.load_game_state()
            self.on_title_screen = False
            if not self._initial_block_generation_done:
                self._generate_visible_blocks()
                self._initial_block_generation_done = True

    def _draw_game_world_elements(self):
        px.camera(self.camera_x, self.camera_y)
        px.cls(12)
        for block in self.all_blocks:
            if block.x + BLOCK_SIZE > self.camera_x and block.x < self.camera_x + SCREEN_WIDTH and \
               block.y + BLOCK_SIZE > self.camera_y and block.y < self.camera_y + SCREEN_HEIGHT:
                block.draw(self.show_debug_overlay, self.font)
        for particle in self.active_particles:
            particle.draw()

    def _draw_ui_and_overlays(self):
        px.camera(0, 0)
        self.select_block_highlighter.draw(px.mouse_x, px.mouse_y,
                                           not self.is_menu_visible and not self.on_title_screen)

        selected_button_action = self.game_menu.draw()
        if selected_button_action:
            self._handle_menu_action(selected_button_action)

        cursor_y_offset = 1 if px.btn(px.MOUSE_BUTTON_LEFT) else 0
        self.toast_manager.draw()
        px.blt(px.mouse_x, px.mouse_y + cursor_y_offset, *SPRITE_CURSOR)

        self._calc_fps()
        if self.show_debug_overlay and not self.is_menu_visible and not self.on_title_screen:
            debug_fps = self.lang_manager.get_string("debug_fps", fps=f"{self.current_fps:.2f}")
            debug_cam = self.lang_manager.get_string("debug_cam", cam_x=self.camera_x, cam_y=self.camera_y)
            debug_blk_pcl = self.lang_manager.get_string("debug_blk_pcl", blk_count=len(self.all_blocks), pcl_count=len(self.active_particles))
            debug_sel = self.lang_manager.get_string("debug_sel", is_selected=self._is_mouse_over_any_block)
            debug_list = [debug_fps, debug_cam, debug_blk_pcl, debug_sel]

            i = 0
            for dev_text in debug_list:
                px.rect(1, (2 + FONT_SIZE * i)-1, estimate_text_width(dev_text)+1, FONT_SIZE, 13)
                self.font.draw(2, 2 + FONT_SIZE * i, dev_text, FONT_SIZE, 7)
                i += 1

    def draw(self):
        if self.on_title_screen:
            self._draw_title_screen()
            px.camera(0,0)
            cursor_y_offset = 1 if px.btn(px.MOUSE_BUTTON_LEFT) else 0
            px.blt(px.mouse_x, px.mouse_y + cursor_y_offset, *SPRITE_CURSOR)
        else:
            self._draw_game_world_elements()
            self._draw_ui_and_overlays()

    def run(self):
        px.run(self.update, self.draw)

if __name__ == '__main__':
    game_instance = DiggingGame()
    game_instance.run()
