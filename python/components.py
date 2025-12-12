import pyxel as px
import random
import math
import time

from constants import *
from utils import *

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
            if block.x <= self.x < block.x + BLOCK_SIZE and block.y <= self.y < block.y + BLOCK_SIZE:
                self.x = block.x - 0.1 if self.vx > 0 else block.x + BLOCK_SIZE + 0.1
                self.vx *= self.BOUNCE_DAMPENING_X
                break

        self.y += self.vy
        is_on_ground_this_frame = False
        for block in collidable_blocks:
            if block.x <= self.x < block.x + BLOCK_SIZE and block.y <= self.y < block.y + BLOCK_SIZE:
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

    def _initialize_blocks(self):
        if self.is_generated:
            return

        world_origin_x, world_origin_y = chunk_coords_to_world_origin(self.chunk_x, self.chunk_y)
        for rel_bx in range(CHUNK_SIZE_X_BLOCKS):
            for rel_by in range(CHUNK_SIZE_Y_BLOCKS):
                block_world_x = world_origin_x + rel_bx * BLOCK_SIZE
                block_world_y = world_origin_y + rel_by * BLOCK_SIZE
                self.blocks[rel_bx][rel_by] = Block(
                    block_world_x, block_world_y,
                    self.game.world_manager.world_seed_main, self.game.world_manager.world_seed_ore
                )
        self.is_generated = True

    def get_block(self, rel_bx, rel_by):
        if not self.is_generated:
            self._initialize_blocks()
        return self.blocks[rel_bx][rel_by] if 0 <= rel_bx < CHUNK_SIZE_X_BLOCKS and 0 <= rel_by < CHUNK_SIZE_Y_BLOCKS else None

    def get_block_by_world_coords(self, world_x, world_y):
        rel_bx, rel_by = world_to_relative_in_chunk_coords(world_x, world_y)
        return self.get_block(rel_bx, rel_by)

    def mark_as_modified_in_session(self):
        self.is_modified_in_session = True

    def to_save_data(self):
        if not self.is_generated:
            return None
        modified_blocks_in_chunk = [block.to_save_data() for row in self.blocks for block in row if block and block.is_modified]
        return {"cx": self.chunk_x, "cy": self.chunk_y, "modified_blocks": modified_blocks_in_chunk} if modified_blocks_in_chunk else None

    def apply_loaded_block_data(self, block_data_list):
        if not self.is_generated:
            self._initialize_blocks()
        for mod_block_save_data in block_data_list:
            rel_bx, rel_by = world_to_relative_in_chunk_coords(mod_block_save_data["x"], mod_block_save_data["y"])
            block_to_update = self.get_block(rel_bx, rel_by)
            if block_to_update:
                block_to_update.current_hp = mod_block_save_data["current_hp"]
                block_to_update.is_modified = True
                block_to_update.is_broken = block_to_update.current_hp <= 0
                if "sprite_id" in mod_block_save_data and mod_block_save_data["sprite_id"] in ID_SPRITE_MAP:
                    block_to_update.sprite_info = ID_SPRITE_MAP[mod_block_save_data["sprite_id"]]
        self.is_modified_in_session = True

    def get_all_active_blocks_in_chunk(self):
        if not self.is_generated:
            return []
        return [block for row in self.blocks for block in row if block and not block.is_broken]

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
        else:
            depth_below_surface_solid = y_block - (self.SURFACE_Y_LEVEL_IN_BLOCKS + 1)
            base_hardness_y = self.HARDNESS_MIN + depth_below_surface_solid * self.HARDNESS_INCREASE_PER_BLOCK_BELOW_SURFACE
            px.nseed(world_seed_noise)
            noise_val_hardness = px.noise(self.x * self.NOISE_SCALE_HARDNESS, self.y * self.NOISE_SCALE_HARDNESS, 0)
            depth_scale_for_noise = min(1.0, depth_below_surface_solid / self.NOISE_VARIATION_TRANSITION_DEPTH_BLOCKS) if self.NOISE_VARIATION_TRANSITION_DEPTH_BLOCKS > 0 else 1.0
            effective_noise_range = self.NOISE_HARDNESS_VARIATION_RANGE * depth_scale_for_noise
            noise_contribution = noise_val_hardness * effective_noise_range * (1.0 if noise_val_hardness >= 0 else self.NEGATIVE_NOISE_IMPACT_FACTOR)
            self.max_hp = math.floor(max(self.HARDNESS_MIN, base_hardness_y + noise_contribution))

            if self.max_hp <= 10:
                self.sprite_info = SPRITE_BLOCK_DIRT
            else:
                px.nseed(world_seed_ore)
                noise_val_ore = px.noise(self.x * self.NOISE_SCALE_ORE, self.y * self.NOISE_SCALE_ORE, 256)
                self.sprite_info = SPRITE_BLOCK_COAL if noise_val_ore >= self.ORE_THRESHOLD else SPRITE_BLOCK_STONE
        self.current_hp = self.max_hp

    def _get_break_animation_frame_index(self):
        if self.current_hp == self.max_hp or self.is_broken:
            return 0
        damage_ratio = (self.max_hp - self.current_hp) / self.max_hp
        return max(1, min(math.ceil(damage_ratio * 5), 5))

    def draw(self, show_debug_info, font_writer):
        if self.is_broken or self.sprite_info is None:
            return
        px.blt(self.x, self.y, *self.sprite_info)
        break_anim_idx = self._get_break_animation_frame_index()
        if break_anim_idx > 0:
            anim_v = SPRITE_BREAK_ANIM_V_START + (break_anim_idx - 1) * BLOCK_SIZE
            px.blt(self.x, self.y, SPRITE_BREAK_ANIM_BANK, SPRITE_BREAK_ANIM_U, anim_v, BLOCK_SIZE, BLOCK_SIZE, SPRITE_BREAK_ANIM_COLKEY)
        if show_debug_info:
            hp_text = f'{self.current_hp}'
            text_x_offset, text_y_offset = calculate_text_center_position(BLOCK_SIZE, BLOCK_SIZE, hp_text)
            if self.current_hp != self.max_hp:
                bar_width_pixels = (self.current_hp / self.max_hp) * (BLOCK_SIZE - 2)
                px.rect(self.x + 1, self.y + BLOCK_SIZE - 2, BLOCK_SIZE - 2, 1, 13)
                px.rect(self.x + 1, self.y + BLOCK_SIZE - 2, bar_width_pixels, 1, 3)
                font_writer.draw(self.x + text_x_offset, self.y + text_y_offset - 1, hp_text, 8, 7)
            else:
                font_writer.draw(self.x + text_x_offset, self.y + text_y_offset, hp_text, 8, 7)

    def handle_click(self):
        if self.is_broken:
            return []
        if self.current_hp == self.max_hp:
            self.is_modified = True
        self.current_hp -= 1
        if self.current_hp <= 0:
            self.is_broken = True
            self.current_hp = 0
            self.is_modified = True
            # game_instance.play_se(0, 1) # This needs to be handled by the game instance
            num_particles = int(min(self.PARTICLES_MAX_ON_BREAK, max(self.PARTICLES_MIN_ON_BREAK, random.gauss(self.PARTICLES_MEAN_ON_BREAK, self.PARTICLES_STDDEV_ON_BREAK))))
            return [Particle(self.x, self.y, self.max_hp) for _ in range(num_particles)]
        else:
            # game_instance.play_se(0, 0) # This needs to be handled by the game instance
            return []

    def is_mouse_over(self, world_mouse_x, world_mouse_y):
        return not self.is_broken and self.x <= world_mouse_x < self.x + BLOCK_SIZE and self.y <= world_mouse_y < self.y + BLOCK_SIZE

    def to_save_data(self):
        sprite_id = next((id_name for id_name, sprite_tuple in ID_SPRITE_MAP.items() if self.sprite_info == sprite_tuple), "unknown")
        return {"x": self.x, "y": self.y, "current_hp": self.current_hp, "sprite_id": sprite_id}
