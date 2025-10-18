import pyxel as px
import random
import math
import time
import os
import json
import traceback
import PyxelUniversalFont as puf

import gen_translation
from constants import *
from utils import *

from managers import LanguageManager, NotificationManager, ParticleManager, WorldManager, PersistenceManager, InputHandler
from ui import SelectBlock, ButtonBox, GameMenu, Notification

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

        # Manager instances
        self.world_manager = WorldManager(self)
        self.particle_manager = ParticleManager()
        self.persistence_manager = PersistenceManager(self)
        self.select_block_highlighter = SelectBlock()
        self.lang_manager = LanguageManager()
        self.button_handler = ButtonBox(self.font, self.lang_manager)
        self.game_menu = GameMenu(self, self.lang_manager, self.font)
        self.notification_manager = NotificationManager(self.font)
        self.input_handler = InputHandler(self)

        self.current_language_code = self.lang_manager.current_lang_code
        self.update_window_title()

        # Game State
        self.on_title_screen = True
        self.is_menu_visible = False
        self.is_loading = False
        self.show_debug_info = False
        self.show_debug_blocks = False
        self._initial_block_generation_done = False
        self._is_mouse_over_any_block = False

        # Player/Camera related
        self.camera_x = 0
        self.camera_y = 0
        self.current_hp = 10
        self.max_hp = 10

        # System/Debug
        self.combination_keys = [px.KEY_B]
        self.combination_key_pressed_during_f3 = {key: False for key in self.combination_keys}
        self.frame_count = 0
        self.start_time = time.time()
        self.current_fps = 0
        self.last_calc_time = time.time()
        self.last_calc_frame = px.frame_count

        # Settings
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

    def _update_game_logic(self):
        world_mouse_x = px.mouse_x + self.camera_x
        world_mouse_y = px.mouse_y + self.camera_y

        self._is_mouse_over_any_block = False
        hovered_block = self.world_manager.get_block_at_world_coords(world_mouse_x, world_mouse_y)
        if hovered_block and not hovered_block.is_broken:
            self._is_mouse_over_any_block = True
        self.select_block_highlighter.update_selection_status(self._is_mouse_over_any_block)

        collidable_blocks = self.world_manager.get_active_blocks_in_view()
        self.particle_manager.update(collidable_blocks)

    def _handle_menu_action(self, action):
        if action == "Save Game":
            self.persistence_manager.save_game_state()
        elif action == "Load Game":
            self.persistence_manager.load_game_state()
        elif action == "Quit Game":
            self.persistence_manager.save_game_state()
            px.quit()

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

    def set_loading_state(self, is_loading):
        self.is_loading = is_loading

    def update(self):
        if self.is_loading:
            if not self.persistence_manager.is_loading:
                self.persistence_manager.process_load_completion()
            return # Do not process other updates while loading

        if not self.persistence_manager.is_saving:
            self.persistence_manager.process_save_completion()

        if self.on_title_screen:
            pass
        else:
            self.input_handler.process_inputs()
            if not self.is_menu_visible:
                self._update_game_logic()

        self.notification_manager.update()

    def _draw_loading_screen(self):
        px.cls(0)
        loading_text = self.lang_manager.get_string("main.loading")
        text_x, text_y = calculate_text_center_position(SCREEN_WIDTH, SCREEN_HEIGHT, loading_text)
        self.font.draw(text_x, text_y, loading_text, FONT_SIZE, 7)

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
            self.persistence_manager.load_game_state(True)
            if not self._initial_block_generation_done:
                self.world_manager.generate_visible_chunks()
                self._initial_block_generation_done = True

    def _draw_game_world_elements(self):
        px.camera(self.camera_x, self.camera_y)
        px.cls(12)
        visible_blocks_list = self.world_manager.get_active_blocks_in_view()
        for block in visible_blocks_list:
            block.draw(self.show_debug_blocks, self.font)
        self.particle_manager.draw()

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
            mouse_x = self.camera_x + math.floor(px.mouse_x / BLOCK_SIZE)
            mouse_y = self.camera_y + math.floor(px.mouse_y / BLOCK_SIZE)
            chunk_x = mouse_x // (CHUNK_SIZE_X_BLOCKS * BLOCK_SIZE)
            chunk_y = mouse_y // (CHUNK_SIZE_Y_BLOCKS * BLOCK_SIZE)

            debug_fps = self.lang_manager.get_string("main.debug.fps", fps=f"{self.current_fps:.2f}")
            debug_cam = self.lang_manager.get_string("main.debug.camera_coord", cam_x=self.camera_x/BLOCK_SIZE, cam_y=self.camera_y/BLOCK_SIZE)
            debug_mouse = self.lang_manager.get_string("main.debug.mouse_coord", mouse_x=mouse_x, mouse_y=mouse_y)
            debug_chunk = self.lang_manager.get_string("main.debug.chunk_coord", chunk_x=chunk_x, chunk_y=chunk_y)
            debug_blk = self.lang_manager.get_string("main.debug.block_count", blk_count=len(self.world_manager.generated_chunk_coords) * CHUNK_SIZE_X_BLOCKS * CHUNK_SIZE_Y_BLOCKS)
            debug_pcl = self.lang_manager.get_string("main.debug.particle_count", pcl_count=len(self.particle_manager.active_particles))
            debug_list = [debug_fps, debug_cam, debug_mouse, debug_chunk, debug_blk, debug_pcl]

            i = 0
            for debug_text in debug_list:
                px.rect(1, (2 + FONT_SIZE * i)-1, estimate_text_width(debug_text)+1, FONT_SIZE, 13)
                self.font.draw(2, 2 + FONT_SIZE * i, debug_text, FONT_SIZE, 7)
                i += 1

    def draw(self):
        if self.is_loading:
            self._draw_loading_screen()
            return

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