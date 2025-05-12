import pyxel as px
import random
import math
import time
import os
import json
import traceback

# TODO: 鉱石の追加 [鉄、コバルト、銅、金、銀、ダイヤ]
# TODO: インベントリの追加
# TODO: マップの追加
# TODO: バイオームの追加
# TODO: 鉱石の高さによる出現頻度の変化
# TODO: 石の高さによる硬度の変化
# TODO: アップグレード等の追加

SAVE_FILE_NAME = "savegame.json"

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

DEFAULT_VOLUME = 7

def calculate_text_center_position(box_width, box_height, text_content, char_width=3, char_spacing=1, char_height=5):
    if not text_content:
        return box_width / 2, box_height / 2 - char_height / 2
    text_char_count = len(text_content)

    text_width_pixels = text_char_count * px.FONT_WIDTH -1 if text_char_count > 0 else 0

    text_y = (box_height - px.FONT_HEIGHT) / 2
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

class ButtonBox:
    def _get_mouse_status_on_button(self, x, y, w, h):
        is_hover = (x <= px.mouse_x <= x + w - 1 and
                    y <= px.mouse_y <= y + h - 1)

        is_pressed_on_button = px.btn(px.MOUSE_BUTTON_LEFT) and is_hover
        is_released_on_button = px.btnr(px.MOUSE_BUTTON_LEFT) and is_hover

        return is_pressed_on_button, is_hover, is_released_on_button

    def draw_button(self, x, y, w, h, text='text', pressed_text='press'):
        is_being_pressed, _, is_released_on = self._get_mouse_status_on_button(x, y, w, h)

        current_text = pressed_text if is_being_pressed else text
        bg_color = COLOR_BUTTON_PRESSED_BG if is_being_pressed else COLOR_BUTTON_BG

        px.rect(x, y, w, h, bg_color)
        px.rectb(x, y, w, h, COLOR_BUTTON_BORDER)

        if not is_being_pressed:
            px.line(x + w -1 , y + 1, x + w -1, y + h - 2, 0)
            px.line(x + 1, y + h - 1, x + w - 2, y + h - 1, 0)

        text_x_offset, text_y_offset = calculate_text_center_position(w, h, current_text)
        px.text(x + text_x_offset, y + text_y_offset, current_text, COLOR_BUTTON_TEXT)

        return is_released_on

    def draw_static_box(self, x, y, w, h, text='text'):
        px.rect(x,y, w, h, COLOR_BUTTON_BG)
        px.rectb(x,y, w, h, COLOR_BUTTON_BORDER)
        text_x_offset, text_y_offset = calculate_text_center_position(w, h, text)
        px.text(x + text_x_offset, y + text_y_offset, text, COLOR_BUTTON_TEXT)

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

    def draw(self, show_debug_info):
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
                px.text(self.x + text_x_offset, self.y + text_y_offset -1, hp_text, 7)
            else:
                px.text(self.x + text_x_offset, self.y + text_y_offset, hp_text, 7)

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

class GameMenu:
    def __init__(self, button_handler_ref, game_instance_ref):
        self.width = 90
        self.height = SCREEN_HEIGHT - (SCREEN_HEIGHT // 10 * 2)
        self.x = (SCREEN_WIDTH - self.width) // 2
        self.y = (SCREEN_HEIGHT - self.height) // 2
        self.button_handler = button_handler_ref
        self.game = game_instance_ref

        self.menu_items = [
            {"label": "Sound Effects", "type": "checkbox", "setting_attr": "se_on"},
            {"label": "Music", "type": "checkbox", "setting_attr": "bgm_on"},
            {"label": "Save Game", "type": "button"},
            {"label": "Load Game", "type": "button"},
            {"label": "Quit Game", "type": "button"}
        ]
        self.selected_action = None

    def _draw_checkbox(self, x, y, label_text, is_checked):
        px.rectb(x, y + (MENU_ITEM_HEIGHT - CHECKBOX_SIZE) // 2, CHECKBOX_SIZE, CHECKBOX_SIZE, 0)
        if is_checked:
            px.rect(x + 2, y + (MENU_ITEM_HEIGHT - CHECKBOX_SIZE) // 2 + 2, CHECKBOX_SIZE - 4, CHECKBOX_SIZE - 4, 0)

        px.text(x + CHECKBOX_SIZE + CHECKBOX_TEXT_GAP, y + (MENU_ITEM_HEIGHT - px.FONT_HEIGHT) // 2, label_text, 0)

    def draw(self, is_active):
        if not is_active:
            return None

        self.selected_action = None

        px.rect(self.x, self.y, self.width, self.height, 9)
        px.rectb(self.x, self.y, self.width, self.height, 0)

        menu_title = "MENU"
        title_x_offset, title_y_offset = calculate_text_center_position(self.width, MENU_ITEM_HEIGHT, menu_title)
        px.text(self.x + title_x_offset, self.y + MENU_PADDING + title_y_offset, menu_title, 0)

        current_y = self.y + MENU_ITEM_HEIGHT + MENU_PADDING * 2

        for item_data in self.menu_items:
            item_x = self.x + MENU_PADDING
            item_w = self.width - MENU_PADDING * 2
            label = item_data["label"]

            if item_data["type"] == "checkbox":
                is_checked_value = getattr(self.game, item_data["setting_attr"])
                self._draw_checkbox(item_x, current_y, label, is_checked_value)

                mouse_on_item = (item_x <= px.mouse_x < item_x + item_w and
                                 current_y <= px.mouse_y < current_y + MENU_ITEM_HEIGHT)

                if mouse_on_item and px.btnp(px.MOUSE_BUTTON_LEFT):
                    setattr(self.game, item_data["setting_attr"], not is_checked_value)
                    # self.game._apply_sound_settings()
                    print(f"{label} toggled to: {not is_checked_value}")

            elif item_data["type"] == "button":
                if self.button_handler.draw_button(item_x, current_y, item_w, MENU_ITEM_HEIGHT, label, f"{label}!"):
                    self.selected_action = label

            current_y += MENU_ITEM_HEIGHT + MENU_PADDING

        return self.selected_action

class DiggingGame:
    GAME_TITLE = 'Digging Game'
    GROUND_SURFACE_Y_BLOCK_INDEX = 7

    CAMERA_SPEED_NORMAL = 8
    CAMERA_SPEED_FAST = 16

    CAMERA_KEY_REPEAT_DELAY_INITIAL = 0.4
    CAMERA_KEY_REPEAT_INTERVAL = 0.05

    def __init__(self):
        px.init(SCREEN_WIDTH, SCREEN_HEIGHT, title=self.GAME_TITLE, fps=60, quit_key=0)

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

        self.all_blocks = []
        self.active_particles = []
        self.generated_block_coordinates = set()

        self.select_block_highlighter = SelectBlock()
        self.button_handler = ButtonBox()
        self.game_menu = GameMenu(self.button_handler, game_instance_ref=self)

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
        self.bgm_on = False

    def play_se(self, ch, snd, loop=False):
        if self.se_on:
            px.play(ch, snd, loop=loop)

    def play_bgm(self, ch, snd, loop=True):
        if self.bgm_on:
            px.play(ch, snd, loop=loop)
        else:
            px.stop(ch)

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
            "camera_x": self.camera_x,
            "camera_y": self.camera_y,
            "se_on": self.se_on,
            "bgm_on": self.bgm_on,
            "world_seed_main": self.world_seed_main,
            "world_seed_ore": self.world_seed_ore,
            "generated_coords": list(self.generated_block_coordinates),
            "active_blocks_states": active_blocks_data
        }

        try:
            with open(SAVE_FILE_NAME, "w") as f:
                json.dump(save_data, f, indent=2)
            print(f"Game saved to {SAVE_FILE_NAME}")
            # TODO: ユーザーに「Saved!」というフィードバックを画面に表示する
        except IOError as e:
            print(f"Error saving game: Could not write to file {SAVE_FILE_NAME}. {e}")
        except Exception as e:
            print(f"An unexpected error occurred during saving: {e}")
            traceback.print_exc()

    def load_game_state(self):
        try:
            with open(SAVE_FILE_NAME, "r") as f:
                load_data = json.load(f)

            self.camera_x = load_data["camera_x"]
            self.camera_y = load_data["camera_y"]
            self.se_on = load_data["se_on"]
            self.bgm_on = load_data["bgm_on"]

            self.world_seed_main = load_data.get("world_seed_main", px.rndi(1, 2**31 - 1))
            self.world_seed_ore = load_data.get("world_seed_ore", px.rndi(1, 2**31 - 1))
            px.nseed(self.world_seed_main)
            px.rseed(self.world_seed_main)

            self.all_blocks = []
            self.generated_block_coordinates = set()

            loaded_generated_coords = load_data.get("generated_coords", [])
            for coord_list in loaded_generated_coords:
                self.generated_block_coordinates.add(tuple(coord_list))

            block_hp_map = {}
            loaded_active_block_states = load_data.get("active_blocks_states", [])
            for block_data in loaded_active_block_states:
                block_hp_map[(block_data["x"], block_data["y"])] = block_data["current_hp"]

            for world_x, world_y in list(self.generated_block_coordinates):
                if world_y // BLOCK_SIZE < self.GROUND_SURFACE_Y_BLOCK_INDEX:
                    continue

                block = Block(world_x, world_y, self.world_seed_main, self.world_seed_ore)

                if (world_x, world_y) in block_hp_map:
                    block.current_hp = block_hp_map[(world_x, world_y)]

                if block.current_hp <= 0:
                    block.is_broken = True
                    block.current_hp = 0
                else:
                    block.is_broken = False

                if not block.is_broken:
                    self.all_blocks.append(block)

            self.active_particles = []

            self._initial_block_generation_done = True

            print(f"Game loaded from {SAVE_FILE_NAME}")
            self.on_title_screen = False
            self.is_menu_visible = False

            # TODO: BGMの再生状態を復元 (例: self.play_bgm(...) を呼ぶ)
            # if self.bgm_on:
            #     self.play_bgm(BGM_CHANNEL, BGM_SOUND_ID, loop=True)
            # else:
            #     px.stop(BGM_CHANNEL) # チャンネルを指定して停止

        except FileNotFoundError:
            print(f"Save file not found: {SAVE_FILE_NAME}")
            # TODO: ユーザーに「セーブファイルが見つかりません」というフィードバックを表示
        except json.JSONDecodeError as e:
            print(f"Error decoding save file ({SAVE_FILE_NAME}): Invalid JSON format. {e}")
            # TODO: ユーザーにエラーフィードバックを表示
        except Exception as e:
            print(f"An unexpected error occurred during loading: {e}")
            traceback.print_exc() # 開発用に詳細なトレースバックを表示
            # TODO: ユーザーに一般的なエラーメッセージを表示

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
            if self.is_menu_visible:
                pass
            else:
                if px.btnp(px.KEY_F3):
                    self.show_debug_overlay = not self.show_debug_overlay
                self._handle_camera_movement()
                self._update_game_logic()

    def _draw_title_screen(self):
        px.cls(1)
        px.camera(0, 0)

        title_x, title_y = calculate_text_center_position(SCREEN_WIDTH, SCREEN_HEIGHT, self.GAME_TITLE)
        px.text(title_x + 1, title_y * 0.5 + 1, self.GAME_TITLE, 3)
        px.text(title_x, title_y * 0.5, self.GAME_TITLE, 11)

        button_w, button_h = 35, 10
        button_x = (SCREEN_WIDTH - button_w) / 2
        button_y = (SCREEN_HEIGHT - button_h) / 2 * 1.25

        if self.button_handler.draw_button(button_x, button_y, button_w, button_h, 'Start', 'Click!'):
            self.load_game_state()
            self.on_title_screen = False
            if not self._initial_block_generation_done:
                self._generate_visible_blocks()
                self._initial_block_generation_done = True

    def _draw_game_world_elements(self):
        px.camera(self.camera_x, self.camera_y)
        px.cls(1)

        for block in self.all_blocks:
            if block.x + BLOCK_SIZE > self.camera_x and block.x < self.camera_x + SCREEN_WIDTH and \
               block.y + BLOCK_SIZE > self.camera_y and block.y < self.camera_y + SCREEN_HEIGHT:
                block.draw(self.show_debug_overlay)

        for particle in self.active_particles:
            particle.draw()

    def _draw_ui_and_overlays(self):
        px.camera(0, 0)

        self.select_block_highlighter.draw(px.mouse_x, px.mouse_y,
                                           not self.is_menu_visible and not self.on_title_screen)

        selected_button_action = self.game_menu.draw(self.is_menu_visible)

        if selected_button_action:
            self._handle_menu_action(selected_button_action)

        cursor_y_offset = 1 if px.btn(px.MOUSE_BUTTON_LEFT) else 0
        px.blt(px.mouse_x, px.mouse_y + cursor_y_offset, *SPRITE_CURSOR)

        self._calc_fps()

        if self.show_debug_overlay and not self.is_menu_visible and not self.on_title_screen:
            px.rect(1,2-1,len(f"FPS: {self.current_fps:.2f}")*4+1,7,13)
            px.rect(1,9-1,len(f"Cam:({self.camera_x},{self.camera_y})")*4+1,7,13)
            px.rect(1,16-1,len(f"Blk:{len(self.all_blocks)} Pcl:{len(self.active_particles)}")*4+1,7,13)
            px.rect(1,23-1,len(f"Hover:{self._is_mouse_over_any_block}")*4+1,7,13)
            px.text(2, 2,  f"FPS: {self.current_fps:.2f}", 7)
            px.text(2, 9,  f"Cam:({self.camera_x},{self.camera_y})", 7)
            px.text(2, 16, f"Blk:{len(self.all_blocks)} Pcl:{len(self.active_particles)}", 7)
            px.text(2, 23, f"Hover:{self._is_mouse_over_any_block}", 7)

    def draw(self):
        if self.on_title_screen:
            self._draw_title_screen()
            self._draw_ui_and_overlays()
        else:
            self._draw_game_world_elements()
            self._draw_ui_and_overlays()

    def run(self):
        px.run(self.update, self.draw)

if __name__ == '__main__':
    game_instance = DiggingGame()
    game_instance.run()
