import pyxel as px
import random
import math
import time
import os
import json
import traceback
import PyxelUniversalFont as puf

import gen_translation

"""
--- player ---
    # TODO: インベントリの追加
    # TODO: アップグレード等の追加
--- map ---
    # TODO: マップの追加
    # TODO: バイオームの追加
--- block ---
    # TODO: 鉱石の高さによる出現頻度の変化
    # TODO: 鉱石の追加 [鉄、コバルト、銅、金、銀、ダイヤ]
    # TODO: 石の高さによる硬度の変化
    # TODO: 横幅50ブロック カメラの移動可能範囲を50+画面横幅
    # TODO: 掘れる範囲の制限
"""

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

LANG_FOLDER = "lang"
DEFAULT_LANGUAGE = "en_us"
FONT_SIZE = 8

NOTIFICATION_PADDING_X = 5
NOTIFICATION_PADDING_Y = 3
NOTIFICATION_LINE_SPACING = 2
NOTIFICATION_INTER_ITEM_SPACING = 2
NOTIFICATION_MAX_WIDTH = 115
NOTIFICATION_MAX_DISPLAY_TIME = 2
MAX_NOTIFICATIONS = 3

NOTIFICATION_FADE_IN_AMOUNT_Y_PER_FRAME = 5
NOTIFICATION_FADE_OUT_INITIAL_AMOUNT_X_PER_FRAME = 2
NOTIFICATION_FADE_OUT_ACCELERATION_X_PER_FRAME = 0.5
NOTIFICATION_FADE_IN_OFFSET_Y = -30
NOTIFICATION_TARGET_Y_TOLERANCE = 1

NOTIFICATION_BG_COLOR = 13
NOTIFICATION_TEXT_COLOR_INFO = 0
NOTIFICATION_TEXT_COLOR_ERROR = 8
NOTIFICATION_TEXT_COLOR_SUCCESS = 5

CHUNK_SIZE_X_BLOCKS = 16
CHUNK_SIZE_Y_BLOCKS = 16

SPRITE_ID_MAP = {
    "air": None,
    "dirt": SPRITE_BLOCK_DIRT,
    "grass": SPRITE_BLOCK_GRASS,
    "stone": SPRITE_BLOCK_STONE,
    "coal": SPRITE_BLOCK_COAL,
}
ID_SPRITE_MAP = {v_str: k_tuple for k_tuple, v_str in SPRITE_ID_MAP.items() if isinstance(k_tuple, str)}
for k_str, v_tuple in SPRITE_ID_MAP.items():
    if isinstance(v_tuple, tuple):
        ID_SPRITE_MAP[k_str] = v_tuple
    elif v_tuple is None and k_str == "air":
        ID_SPRITE_MAP[k_str] = None

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

def world_to_chunk_coords(world_x, world_y):
    """ワールド座標をそれが属するチャンクの座標に変換"""
    chunk_x = math.floor(world_x / (CHUNK_SIZE_X_BLOCKS * BLOCK_SIZE))
    chunk_y = math.floor(world_y / (CHUNK_SIZE_Y_BLOCKS * BLOCK_SIZE))
    return chunk_x, chunk_y

def world_to_relative_in_chunk_coords(world_x, world_y):
    """ワールド座標をチャンク内相対ブロック座標(0-15, 0-15)に変換"""
    cx, cy = world_to_chunk_coords(world_x, world_y)
    rel_bx = (world_x // BLOCK_SIZE) - cx * CHUNK_SIZE_X_BLOCKS
    rel_by = (world_y // BLOCK_SIZE) - cy * CHUNK_SIZE_Y_BLOCKS
    return rel_bx, rel_by

def chunk_coords_to_world_origin(chunk_x, chunk_y):
    """チャンク座標からそのチャンクのワールド原点座標(左上)を計算"""
    world_x = chunk_x * CHUNK_SIZE_X_BLOCKS * BLOCK_SIZE
    world_y = chunk_y * CHUNK_SIZE_Y_BLOCKS * BLOCK_SIZE
    return world_x, world_y

"""
def numbers_to_notes(number_list):
    note_names = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']

    result = ''
    for number in number_list:
        note_index = number % 12
        note_name = note_names[note_index]
        octave = number // 12

        result += note_name + str(octave)
    return result
"""

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
            try:
                en_data = gen_translation._generate_en_()
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

class Notification:
    def __init__(self, message, duration=NOTIFICATION_MAX_DISPLAY_TIME, msg_type="info",
                 wrap_text_func=None, max_wrap_width=0):
        self.message = message
        self.start_time = time.time()
        self.duration = duration
        self.msg_type = msg_type
        self.is_alive = True

        self.state = "fading_in" # "fading_in", "visible", "fading_out"
        self.current_x = None
        self.current_y = None
        self.target_x = None
        self.target_y = None
        self.vel_x = 0

        self.fade_out_timer = 0

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
            print("Warning: Cannot calculate notification dimensions without text utility functions.")
            self._box_width = 0
            self._box_height = 0
            return

        lines = self._wrap_text(self.message, self._max_wrap_width)

        total_text_height = len(lines) * FONT_SIZE + (max(0, len(lines) - 1)) * NOTIFICATION_LINE_SPACING
        self._box_height = total_text_height + NOTIFICATION_PADDING_Y * 2

        max_line_width = 0
        for line in lines:
            max_line_width = max(max_line_width, estimate_text_width(line))

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

        if self._box_width is None or self._box_height is None:
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
        else:
            if self.target_y is not None:
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
        if self._wrap_text and self._max_wrap_width > 0:
            return self._wrap_text(self.message, self._max_wrap_width)
        return [self.message]

class NotificationManager:
    def __init__(self, font_writer: puf.Writer):
        self.notifications = []
        self.font = font_writer

    def add_notification(self, message, duration=NOTIFICATION_MAX_DISPLAY_TIME, msg_type="info"):
        effective_wrap_width = NOTIFICATION_MAX_WIDTH - NOTIFICATION_PADDING_X * 2
        new_notif = Notification(
            message, duration, msg_type,
            wrap_text_func=self._wrap_text,
            max_wrap_width=effective_wrap_width,
        )
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
            test_width = estimate_text_width(test_line)

            if test_width > max_width and current_line:
                wrapped_lines.append(current_line)
                current_line = char
            else:
                current_line = test_line

        if current_line:
            wrapped_lines.append(current_line)
        return wrapped_lines

    def _wrap_text(self, text, max_width):
        temp_wrapped_lines = []
        paragraphs = text.split('\n')

        for para in paragraphs:
            if not para:
                temp_wrapped_lines.append("")
                continue

            current_line = ""
            words = para.split(' ')

            for i, word in enumerate(words):
                test_line_with_space = current_line + (" " if current_line else "") + word
                test_width_with_space = estimate_text_width(test_line_with_space)
                if test_width_with_space <= max_width:
                    current_line = test_line_with_space
                else:
                    if current_line:
                        temp_wrapped_lines.append(current_line)
                        current_line = word
                    else:
                        char_wrapped_word_lines = self._wrap_chars_only(word, max_width)
                        temp_wrapped_lines.extend(char_wrapped_word_lines)
                        current_line = ""

            if current_line:
                temp_wrapped_lines.append(current_line)

            final_wrapped_result = []
            for line in temp_wrapped_lines:
                if not line:
                    final_wrapped_result.append(line)
                    continue

                if estimate_text_width(line) > max_width:
                    char_wrapped_sublines = self._wrap_chars_only(line, max_width)
                    final_wrapped_result.extend(char_wrapped_sublines)
                else:
                    final_wrapped_result.append(line)

        return final_wrapped_result

    def update(self):
        for notif in self.notifications:
            notif.update()

        self.notifications = [n for n in self.notifications if n.is_alive]

        current_target_y = NOTIFICATION_PADDING_Y
        #positioned_notifications = [n for n in self.notifications if n.state != "fading_out"]

        for notif in reversed(self.notifications):
            if notif.is_alive and notif.state != "fading_out":
                effective_wrap_width = NOTIFICATION_MAX_WIDTH - NOTIFICATION_PADDING_X * 2
                lines = self._wrap_text(notif.message, effective_wrap_width)
                total_text_height = len(lines) * FONT_SIZE + (max(0, len(lines) - 1)) * NOTIFICATION_LINE_SPACING
                box_height = total_text_height + NOTIFICATION_PADDING_Y * 2

                max_line_width = 0
                for line in lines:
                    max_line_width = max(max_line_width, estimate_text_width(line))

                box_width = min(NOTIFICATION_MAX_WIDTH, max_line_width + NOTIFICATION_PADDING_X * 2)

                target_x = SCREEN_WIDTH - box_width - NOTIFICATION_PADDING_X
                target_y = current_target_y
                notif.set_target_position(target_x, target_y)

                current_target_y += box_height + NOTIFICATION_INTER_ITEM_SPACING

    def draw(self):
        if not self.notifications:
            return

        for notif in reversed(self.notifications):
            draw_pos = notif.get_draw_position()
            box_dims = notif.get_box_dimensions()

            if draw_pos is None or box_dims is None:
                continue

            box_x, box_y = draw_pos
            box_width, box_height = box_dims

            px.rect(int(box_x), int(box_y), int(box_width), int(box_height), NOTIFICATION_BG_COLOR)
            px.rectb(int(box_x), int(box_y), int(box_width), int(box_height), notif.get_text_color())

            text_color = notif.get_text_color()
            line_y_offset = box_y + NOTIFICATION_PADDING_Y
            lines = notif.get_wrapped_lines()

            for line in lines:
                self.font.draw(int(box_x + NOTIFICATION_PADDING_X), int(line_y_offset), line, FONT_SIZE, text_color)
                line_y_offset += FONT_SIZE + NOTIFICATION_LINE_SPACING

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

class Chunk:
    def __init__(self, chunk_x, chunk_y, game_ref):
        self.chunk_x = chunk_x
        self.chunk_y = chunk_y
        self.game = game_ref
        self.blocks = [[None for _ in range(CHUNK_SIZE_Y_BLOCKS)] for _ in range(CHUNK_SIZE_X_BLOCKS)]
        self.is_generated = False
        self.is_modified_in_session = False
        # self.has_modified_blocks = False # 将来使用する可能性あり

    def _initialize_blocks(self):
        if self.is_generated:
            return

        world_origin_x, world_origin_y = chunk_coords_to_world_origin(self.chunk_x, self.chunk_y)

        for rel_bx in range(CHUNK_SIZE_X_BLOCKS):
            for rel_by in range(CHUNK_SIZE_Y_BLOCKS):
                block_world_x = world_origin_x + rel_bx * BLOCK_SIZE
                block_world_y = world_origin_y + rel_by * BLOCK_SIZE

                if block_world_y // BLOCK_SIZE < self.game.GROUND_SURFACE_Y_BLOCK_INDEX:
                    air_block = Block(block_world_x, block_world_y, self.game.world_seed_main, self.game.world_seed_ore)
                    self.blocks[rel_bx][rel_by] = air_block
                else:
                    self.blocks[rel_bx][rel_by] = Block(
                        block_world_x, block_world_y,
                        self.game.world_seed_main, self.game.world_seed_ore
                    )
        self.is_generated = True

    def get_block(self, rel_bx, rel_by):
        if not self.is_generated:
            self._initialize_blocks()

        if 0 <= rel_bx < CHUNK_SIZE_X_BLOCKS and 0 <= rel_by < CHUNK_SIZE_Y_BLOCKS:
            return self.blocks[rel_bx][rel_by]
        return None

    def get_block_by_world_coords(self, world_x, world_y):
        rel_bx, rel_by = world_to_relative_in_chunk_coords(world_x, world_y)
        return self.get_block(rel_bx, rel_by)

    def mark_as_modified_in_session(self):
        self.is_modified_in_session = True

    def to_save_data(self):
        if not self.is_generated:
            return None

        modified_blocks_in_chunk = []
        has_any_modification = False
        for rel_bx in range(CHUNK_SIZE_X_BLOCKS):
            for rel_by in range(CHUNK_SIZE_Y_BLOCKS):
                block = self.blocks[rel_bx][rel_by]
                if block and block.is_modified:
                    modified_blocks_in_chunk.append(block.to_save_data())
                    has_any_modification = True

        if has_any_modification:
            return {
                "cx": self.chunk_x,
                "cy": self.chunk_y,
                "modified_blocks": modified_blocks_in_chunk
            }
        return None

    def apply_loaded_block_data(self, block_data_list):
        if not self.is_generated:
            self._initialize_blocks()

        for mod_block_save_data in block_data_list:
            world_x, world_y = mod_block_save_data["x"], mod_block_save_data["y"]
            loaded_hp = mod_block_save_data["current_hp"]
            loaded_sprite_id = mod_block_save_data.get("sprite_id")

            rel_bx, rel_by = world_to_relative_in_chunk_coords(world_x, world_y)
            block_to_update = self.get_block(rel_bx, rel_by)
            if block_to_update:
                block_to_update.current_hp = loaded_hp
                block_to_update.is_modified = True
                if loaded_hp <= 0:
                    block_to_update.is_broken = True
                    block_to_update.current_hp = 0
                else:
                    block_to_update.is_broken = False

                if loaded_sprite_id and loaded_sprite_id in ID_SPRITE_MAP:
                    block_to_update.sprite_info = ID_SPRITE_MAP[loaded_sprite_id]

        self.is_modified_in_session = True

    def get_all_active_blocks_in_chunk(self):
        if not self.is_generated:
            return []

        active_blocks = []
        for row in self.blocks:
            for block in row:
                if block and not block.is_broken:
                    active_blocks.append(block)
        return active_blocks

class Block:
    HARDNESS_MIN = 3
    NOISE_SCALE_HARDNESS = 0.005
    NOISE_SCALE_ORE = 0.04
    ORE_THRESHOLD = 0.4

    SURFACE_Y_LEVEL_IN_BLOCKS = 7

    PARTICLES_MIN_ON_BREAK = 5
    PARTICLES_MAX_ON_BREAK = 15
    PARTICLES_MEAN_ON_BREAK = 10
    PARTICLES_STDDEV_ON_BREAK = 2

    HARDNESS_INCREASE_PER_BLOCK_BELOW_SURFACE = 0.1
    NOISE_HARDNESS_VARIATION_RANGE = 20
    NOISE_VARIATION_TRANSITION_DEPTH_BLOCKS = 50
    NEGATIVE_NOISE_IMPACT_FACTOR = 0.25

    def __init__(self, x, y, world_seed_noise, world_seed_ore):
        self.x = x
        self.y = y
        self.is_broken = False
        self.is_modified = False

        y_block = self.y // BLOCK_SIZE

        if y_block < self.SURFACE_Y_LEVEL_IN_BLOCKS:
            self.is_broken = True
            self.sprite_info = None
            self.max_hp = 0
            self.current_hp = 0
            return

        if y_block == self.SURFACE_Y_LEVEL_IN_BLOCKS:
            self.sprite_info = SPRITE_BLOCK_GRASS
            self.max_hp = self.HARDNESS_MIN
            self.current_hp = self.max_hp
            return

        y_start_solid = self.SURFACE_Y_LEVEL_IN_BLOCKS + 1
        depth_below_surface_solid = y_block - y_start_solid
        base_hardness_y = self.HARDNESS_MIN + depth_below_surface_solid * self.HARDNESS_INCREASE_PER_BLOCK_BELOW_SURFACE

        px.nseed(world_seed_noise)
        noise_val_hardness = px.noise(self.x * self.NOISE_SCALE_HARDNESS,
                                      self.y * self.NOISE_SCALE_HARDNESS, 0)

        if self.NOISE_VARIATION_TRANSITION_DEPTH_BLOCKS <= 0:
            depth_scale_for_noise = 1.0
        else:
            depth_scale_for_noise = min(1.0, depth_below_surface_solid / self.NOISE_VARIATION_TRANSITION_DEPTH_BLOCKS)

        effective_noise_range = self.NOISE_HARDNESS_VARIATION_RANGE * depth_scale_for_noise

        noise_contribution = 0.0
        if noise_val_hardness >= 0:
            noise_contribution = noise_val_hardness * effective_noise_range
        else:
            noise_contribution = noise_val_hardness * effective_noise_range * self.NEGATIVE_NOISE_IMPACT_FACTOR

        combined_hardness = base_hardness_y + noise_contribution

        self.max_hp = math.floor(max(self.HARDNESS_MIN, combined_hardness))
        self.current_hp = self.max_hp

        if self.max_hp <= 10:
            self.sprite_info = SPRITE_BLOCK_DIRT
        else:
            px.nseed(world_seed_ore)
            noise_val_ore = px.noise(self.x * self.NOISE_SCALE_ORE,
                                     self.y * self.NOISE_SCALE_ORE, 256)
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
                font_writer.draw(self.x + text_x_offset, self.y + text_y_offset -1, hp_text, 8, 7)
            else:
                font_writer.draw(self.x + text_x_offset, self.y + text_y_offset, hp_text, 8, 7)

    def handle_click(self):
        if self.is_broken:
            return []

        if self.current_hp == self.max_hp:
            self.is_modified = True

        self.current_hp -= 1
        created_particles = []

        if self.current_hp <= 0:
            self.is_broken = True
            self.current_hp = 0
            self.is_modified = True
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
        sprite_id_to_save = "unknown"
        for id_name, sprite_tuple in ID_SPRITE_MAP.items():
            if self.sprite_info == sprite_tuple:
                sprite_id_to_save = id_name
                break

        return {"x": self.x, "y": self.y, "current_hp": self.current_hp, "sprite_id": sprite_id_to_save}

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

        menu_title_str = self.lang_manager.get_string("menu.title")
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

        self.chunks = {}
        self.generated_chunk_coords = set()
        self.active_particles = []

        self.select_block_highlighter = SelectBlock()
        self.lang_manager = LanguageManager()
        self.button_handler = ButtonBox(self.font, self.lang_manager)
        self.game_menu = GameMenu(self, self.lang_manager, self.font)
        self.notification_manager = NotificationManager(self.font)

        self.current_language_code = self.lang_manager.current_lang_code
        self.update_window_title()

        self.on_title_screen = True
        self.is_menu_visible = False
        self.show_debug_info = False
        self.show_debug_blocks = False

        self.combination_keys = [
            px.KEY_B,
        ]
        self.combination_key_pressed_during_f3 = {key: False for key in self.combination_keys}

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
        self.game_title = self.lang_manager.get_string("game.title")
        px.title = self.game_title

    def play_se(self, ch, snd, loop=False):
        if self.se_on:
            px.play(ch, snd, loop=loop)

    def play_bgm(self, ch, snd, loop=True):
        if self.bgm_on:
            px.play(ch, snd, loop=loop)
        else:
            px.stop(ch)

    def _ensure_chunk_generated_and_get(self, chunk_x, chunk_y):
        if (chunk_x, chunk_y) not in self.chunks:
            self.chunks[(chunk_x, chunk_y)] = Chunk(chunk_x, chunk_y, self)

        chunk = self.chunks[(chunk_x, chunk_y)]
        if not chunk.is_generated:
            chunk._initialize_blocks()
            self.generated_chunk_coords.add((chunk_x, chunk_y))
        return chunk

    def _generate_visible_chunks(self):
        cam_world_left = self.camera_x
        cam_world_right = self.camera_x + SCREEN_WIDTH
        cam_world_top = self.camera_y
        cam_world_bottom = self.camera_y + SCREEN_HEIGHT

        start_cx, start_cy = world_to_chunk_coords(cam_world_left, cam_world_top)
        end_cx, end_cy = world_to_chunk_coords(cam_world_right, cam_world_bottom)

        for cx in range(start_cx, end_cx + 1):
            for cy in range(start_cy, end_cy + 1):
                self._ensure_chunk_generated_and_get(cx, cy)

    def get_block_at_world_coords(self, world_x, world_y):
        chunk_x, chunk_y = world_to_chunk_coords(world_x, world_y)
        chunk = self._ensure_chunk_generated_and_get(chunk_x, chunk_y)
        if chunk:
            return chunk.get_block_by_world_coords(world_x, world_y)
        return None

    def _get_active_blocks_in_view(self):
        active_blocks = []
        cam_world_left = self.camera_x - BLOCK_SIZE
        cam_world_right = self.camera_x + SCREEN_WIDTH + BLOCK_SIZE
        cam_world_top = self.camera_y - BLOCK_SIZE
        cam_world_bottom = self.camera_y + SCREEN_HEIGHT + BLOCK_SIZE

        start_cx, start_cy = world_to_chunk_coords(cam_world_left, cam_world_top)
        end_cx, end_cy = world_to_chunk_coords(cam_world_right, cam_world_bottom)

        for cx in range(start_cx, end_cx + 1):
            for cy in range(start_cy, end_cy + 1):
                chunk = self.chunks.get((cx, cy))
                if chunk and chunk.is_generated:
                    for block in chunk.get_all_active_blocks_in_chunk():
                        if block.x + BLOCK_SIZE > self.camera_x and block.x < self.camera_x + SCREEN_WIDTH and \
                           block.y + BLOCK_SIZE > self.camera_y and block.y < self.camera_y + SCREEN_HEIGHT:
                            active_blocks.append(block)
        return active_blocks

    def _handle_camera_movement(self):
        camera_moved_flag = False
        current_time = time.time()

        key_directions = {
            px.KEY_W: (0, -1, 'W'), px.KEY_A: (-1, 0, 'A'),
            px.KEY_S: (0, 1, 'S'),  px.KEY_D: (1, 0, 'D')
        }

        for key_code, (dx_mult, dy_mult, key_char) in key_directions.items():
            base_speed = self.CAMERA_SPEED_FAST if px.btn(px.KEY_SHIFT) else self.CAMERA_SPEED_NORMAL
            moved_this_key = False
            if px.btnp(key_code):
                moved_this_key = True
                self._key_pressed_start_time[key_char] = current_time
                self._key_last_repeat_action_time[key_char] = current_time
            elif px.btn(key_code):
                if key_char in self._key_pressed_start_time:
                    if current_time - self._key_pressed_start_time[key_char] >= self.CAMERA_KEY_REPEAT_DELAY_INITIAL:
                        if current_time - self._key_last_repeat_action_time[key_char] >= self.CAMERA_KEY_REPEAT_INTERVAL:
                            moved_this_key = True
                            self._key_last_repeat_action_time[key_char] = current_time
            else:
                if key_char in self._key_pressed_start_time:
                    del self._key_pressed_start_time[key_char]
                if key_char in self._key_last_repeat_action_time:
                    del self._key_last_repeat_action_time[key_char]

            if moved_this_key:
                self.camera_x += dx_mult * base_speed
                self.camera_y += dy_mult * base_speed
                camera_moved_flag = True

        if camera_moved_flag:
            self._generate_visible_chunks()

    def _update_game_logic(self):
        world_mouse_x = px.mouse_x + self.camera_x
        world_mouse_y = px.mouse_y + self.camera_y

        self._is_mouse_over_any_block = False
        hovered_block = self.get_block_at_world_coords(world_mouse_x, world_mouse_y)
        if hovered_block and not hovered_block.is_broken:
            self._is_mouse_over_any_block = True
        self.select_block_highlighter.update_selection_status(self._is_mouse_over_any_block)

        if px.btnp(px.MOUSE_BUTTON_LEFT):
            clicked_block = self.get_block_at_world_coords(world_mouse_x, world_mouse_y)
            if clicked_block:
                chunk_x, chunk_y = world_to_chunk_coords(clicked_block.x, clicked_block.y)
                if (chunk_x, chunk_y) in self.chunks:
                    self.chunks[(chunk_x, chunk_y)].mark_as_modified_in_session()

                new_particles = clicked_block.handle_click()
                self.active_particles.extend(new_particles)

        temp_collidable_blocks = []
        collidable_blocks_for_particles = self._get_active_blocks_in_view()

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
        modified_chunks_data = []
        for chunk_coord, chunk_instance in self.chunks.items():
            if chunk_instance.is_modified_in_session:
                chunk_save_data = chunk_instance.to_save_data()
                if chunk_save_data:
                    modified_chunks_data.append(chunk_save_data)

        save_data = {
            "camera_x": self.camera_x, "camera_y": self.camera_y,
            "se_on": self.se_on, "bgm_on": self.bgm_on,
            "world_seed_main": self.world_seed_main, "world_seed_ore": self.world_seed_ore,
            "generated_chunk_coords": [list(c) for c in self.generated_chunk_coords],
            "modified_chunks": modified_chunks_data,
            "current_language": self.lang_manager.current_lang_code
        }

        message_prefix = "debug" if self.show_debug_info else "default"
        try:
            with open(SAVE_FILE_NAME, "w") as f:
                json.dump(save_data, f, indent=2)
            self.notification_manager.add_notification(
                self.lang_manager.get_string(f"notification.save.success.{message_prefix}", filename=SAVE_FILE_NAME),
                msg_type="success"
            )
        except IOError as e:
            self.notification_manager.add_notification(
                self.lang_manager.get_string(f"notification.save.error.write.{message_prefix}", filename=SAVE_FILE_NAME, error=str(e)),
                msg_type="error"
            )
        except Exception as e:
            self.notification_manager.add_notification(
                self.lang_manager.get_string(f"notification.save.error.unexpected.{message_prefix}", error=str(e)),
                msg_type="error"
            )
            traceback.print_exc()

    def _regenerate_world_from_chunks_and_apply_mods(self, loaded_gen_chunk_coords_set, loaded_mod_chunks_data):
        self.chunks = {}
        self.generated_chunk_coords = set()

        for cx_cy_tuple in loaded_gen_chunk_coords_set:
            self.generated_chunk_coords.add(cx_cy_tuple)
        for cx, cy in self.generated_chunk_coords:
            self._ensure_chunk_generated_and_get(cx, cy)

        mod_chunks_map = {(cd["cx"], cd["cy"]): cd["modified_blocks"] for cd in loaded_mod_chunks_data}
        for chunk_coord, modified_blocks_list in mod_chunks_map.items():
            cx, cy = chunk_coord
            if (cx, cy) in self.chunks:
                self.chunks[(cx, cy)].apply_loaded_block_data(modified_blocks_list)

    def load_game_state(self, start=False):
        message_prefix = "debug" if self.show_debug_info else "default"

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
            else:
                self.current_language_code = self.lang_manager.current_lang_code
            self.update_window_title()

            loaded_gen_chunk_coords_list = load_data.get("generated_chunk_coords", [])
            loaded_gen_chunk_coords_set = set()
            for coord_pair_list in loaded_gen_chunk_coords_list:
                loaded_gen_chunk_coords_set.add(tuple(coord_pair_list))
            loaded_modified_chunks_data = load_data.get("modified_chunks", [])

            self._regenerate_world_from_chunks_and_apply_mods(loaded_gen_chunk_coords_set, loaded_modified_chunks_data)

            self.active_particles = []
            self._initial_block_generation_done = True

            self._generate_visible_chunks()
            self.notification_manager.add_notification(
                self.lang_manager.get_string(f"notification.load.success.{message_prefix}", filename=SAVE_FILE_NAME),
                msg_type="success")
            self.on_title_screen = False;
            self.is_menu_visible = False
            # if self.bgm_on: self.play_bgm(BGM_CHANNEL, BGM_SOUND_ID) else: px.stop(BGM_CHANNEL)

        except FileNotFoundError:
            if not start:
                self.notification_manager.add_notification(
                self.lang_manager.get_string(f"notification.load.error.not_found.{message_prefix}", filename=SAVE_FILE_NAME),
                msg_type="error"
                )
        except json.JSONDecodeError as e:
            self.notification_manager.add_notification(
                self.lang_manager.get_string(f"notification.load.error.decode.{message_prefix}", filename=SAVE_FILE_NAME, error=str(e)),
                msg_type="error"
            )
        except Exception as e:
            self.notification_manager.add_notification(
                self.lang_manager.get_string(f"notification.load.error.unexpected.{message_prefix}", error=str(e)),
                msg_type="error"
            )
            traceback.print_exc()

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
            if px.btnr(px.KEY_ESCAPE):
                self.is_menu_visible = not self.is_menu_visible
                if not self.is_menu_visible:
                    self.game_menu.is_lang_dropdown_open = False

            if self.is_menu_visible:
                self.game_menu.handle_input()
            else:
                if px.btnp(px.KEY_F3):
                    for key in self.combination_keys:
                        self.combination_key_pressed_during_f3[key] = False

                if px.btn(px.KEY_F3):
                    for key in self.combination_keys:
                        if px.btnp(key):
                            self.combination_key_pressed_during_f3[key] = True

                for key in self.combination_keys:
                    if px.btnr(key) and px.btn(px.KEY_F3):
                        if key == px.KEY_B:
                            self.show_debug_blocks = not self.show_debug_blocks

                if px.btnr(px.KEY_F3):
                    combination_occurred_during_f3_press = any(self.combination_key_pressed_during_f3.values())
                    if not combination_occurred_during_f3_press:
                        self.show_debug_info = not self.show_debug_info

                self._handle_camera_movement()
                self._update_game_logic()
        self.notification_manager.update()

    def _draw_title_screen(self):
        px.cls(5)
        px.camera(0, 0)

        title_str = self.lang_manager.get_string("game.title")
        t_tw, t_th = calculate_text_center_position(SCREEN_WIDTH, SCREEN_HEIGHT, title_str)
        self.font.draw(t_tw + 1, t_th * 0.5 + 1, title_str, FONT_SIZE, 3)
        self.font.draw(t_tw, t_th * 0.5, title_str, FONT_SIZE, 11)

        button_w, button_h = 60, 10
        button_x = (SCREEN_WIDTH - button_w) / 2
        button_y = (SCREEN_HEIGHT - button_h) / 2 * 1.25

        if not hasattr(self, 'title_button_handler'):
            self.title_button_handler = ButtonBox(self.font, self.lang_manager)

        if self.button_handler.draw_button(button_x, button_y, button_w, button_h,
                                           self.lang_manager.get_string('button.title_screen.start.default'),
                                           self.lang_manager.get_string('button.title_screen.start.pressed')):
            self.load_game_state(True)
            self.on_title_screen = False
            if not self._initial_block_generation_done:
                self._generate_visible_chunks()
                self._initial_block_generation_done = True

    def _draw_game_world_elements(self):
        px.camera(self.camera_x, self.camera_y)
        px.cls(12)
        visible_blocks_list = self._get_active_blocks_in_view()
        for block in visible_blocks_list:
            if block.x + BLOCK_SIZE > self.camera_x and block.x < self.camera_x + SCREEN_WIDTH and \
               block.y + BLOCK_SIZE > self.camera_y and block.y < self.camera_y + SCREEN_HEIGHT:
                block.draw(self.show_debug_blocks, self.font)
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
        px.blt(px.mouse_x, px.mouse_y + cursor_y_offset, *SPRITE_CURSOR)

        self._calc_fps()
        if self.show_debug_info and not self.is_menu_visible and not self.on_title_screen:
            mouse_x = self.camera_x + math.floor(px.mouse_x / BLOCK_SIZE) * BLOCK_SIZE
            mouse_y = self.camera_y + math.floor(px.mouse_y / BLOCK_SIZE) * BLOCK_SIZE
            chunk_x = mouse_x // (CHUNK_SIZE_X_BLOCKS * BLOCK_SIZE)
            chunk_y = mouse_y // (CHUNK_SIZE_Y_BLOCKS * BLOCK_SIZE)

            debug_fps = self.lang_manager.get_string("main.debug.fps", fps=f"{self.current_fps:.2f}")
            debug_cam = self.lang_manager.get_string("main.debug.camera_coord", cam_x=self.camera_x, cam_y=self.camera_y)
            debug_mouse = self.lang_manager.get_string("main.debug.mouse_coord", mouse_x=mouse_x, mouse_y=mouse_y)
            debug_chunk = self.lang_manager.get_string("main.debug.chunk_coord", chunk_x=chunk_x, chunk_y=chunk_y)
            debug_blk = self.lang_manager.get_string("main.debug.block_count", blk_count=len(self.generated_chunk_coords) * CHUNK_SIZE_X_BLOCKS * CHUNK_SIZE_Y_BLOCKS)
            debug_pcl = self.lang_manager.get_string("main.debug.particle_count", pcl_count=len(self.active_particles))
            debug_hover = self.lang_manager.get_string("main.debug.hover_state", is_hovered=self._is_mouse_over_any_block)
            debug_list = [debug_fps, debug_cam, debug_mouse, debug_chunk, debug_blk, debug_pcl, debug_hover]

            i = 0
            for debug_text in debug_list:
                px.rect(1, (2 + FONT_SIZE * i)-1, estimate_text_width(debug_text)+1, FONT_SIZE, 13)
                self.font.draw(2, 2 + FONT_SIZE * i, debug_text, FONT_SIZE, 7)
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
        self.notification_manager.draw()

    def run(self):
        px.run(self.update, self.draw)

if __name__ == '__main__':
    game_instance = DiggingGame()
    game_instance.run()
