import sys
import os
import pytest
import math
import importlib
import json
import io

# Add project root to Python path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

# Import all necessary modules from the game
from constants import *
from utils import estimate_text_width, calculate_text_center_position
from components import Particle, Block
from managers import LanguageManager, NotificationManager, PersistenceManager, WorldManager, InputHandler

@pytest.fixture
def input_handler(mock_px_setup):
    mock_px, diggame, _, _, _, _ = mock_px_setup
    # A mock game object is needed for the input handler
    game = MockGameObject()
    game.game_menu = MockGameMenu()
    return InputHandler(game)

class MockGameMenu:
    def __init__(self):
        self.is_lang_dropdown_open = False
    def handle_input(self):
        pass

def test_input_handler_toggle_menu(input_handler, mock_px_setup, monkeypatch):
    mock_px, _, _, _, _, _ = mock_px_setup
    game = input_handler.game
    game.is_menu_visible = False
    mock_px.KEY_ESCAPE = 1 # Mock ESC key code
    monkeypatch.setattr(mock_px, 'btnr', lambda key: key == mock_px.KEY_ESCAPE)

    input_handler.process_inputs()

    assert game.is_menu_visible
from ui import ButtonBox

# --- Mocks and Fixtures ---

class MockPyxel:
    def __init__(self):
        self.noise_val = 0.5
        self.mouse_x = 0
        self.mouse_y = 0
        self._btn_status = {}
        self.MOUSE_BUTTON_LEFT = 0

    def nseed(self, seed):
        pass

    def rseed(self, seed):
        pass

    def noise(self, x, y, z):
        return self.noise_val

    def btn(self, key):
        return self._btn_status.get(key, False)

    def btnp(self, key, hold=0, period=0):
        return self._btn_status.get(key, False)

    def btnr(self, key):
        return False

    def rect(self, x, y, w, h, col):
        pass

    def rectb(self, x, y, w, h, col):
        pass

    def line(self, x1, y1, x2, y2, col):
        pass

    def rndi(self, a, b):
        return a

class MockGameObject:
    def __init__(self):
        self.camera_x = 0
        self.camera_y = 0
        self.se_on = True
        self.bgm_on = True
        self.show_debug_info = False
        self.world_manager = MockWorldManager()
        self.lang_manager = MockLangManager()
        self.notification_manager = MockNotificationManager()
        self.particle_manager = MockParticleManager()
        self.on_title_screen = False
        self.is_menu_visible = False
        self._initial_block_generation_done = False
        self.current_language_code = "en_us"

    def update_window_title(self):
        pass

class MockWorldManager:
    def __init__(self):
        self.world_seed_main = 123
        self.world_seed_ore = 456
        self.generated_chunk_coords = set()
        self.chunks = {}

    def regenerate_world_from_chunks_and_apply_mods(self, coords, mods):
        pass

    def generate_visible_chunks(self):
        pass

class MockLangManager:
    def __init__(self):
        self.current_lang_code = "en_us"

    def get_string(self, key, **kwargs):
        return key

    def set_language(self, lang_code):
        self.current_lang_code = lang_code
        return True

class MockNotificationManager:
    def add_notification(self, msg, msg_type):
        pass

class MockParticleManager:
    def __init__(self):
        self.active_particles = []

class MockFontWriter:
    def draw(self, x, y, text, size, color):
        pass

class MockNotification:
    def __init__(self, message, duration, msg_type, wrap_text_func, max_wrap_width):
        self.is_alive = True
        self.update_called = False
        self.state = "visible"

    def update(self):
        self.update_called = True

    def get_box_dimensions(self):
        return (10, 10)

    def set_target_position(self, x, y):
        pass

class MockFileContextManager:
    def __init__(self, mock_file):
        self.mock_file = mock_file
    def __enter__(self):
        return self.mock_file
    def __exit__(self, exc_type, exc_val, exc_tb):
        pass # Do not close the file

@pytest.fixture(scope="function")
def mock_px_setup():
    original_pyxel = sys.modules.get('pyxel')
    mock_instance = MockPyxel()
    sys.modules['pyxel'] = mock_instance

    modules_to_reload = ['diggame', 'utils', 'components', 'managers', 'ui']
    for mod_name in modules_to_reload:
        if mod_name in sys.modules:
            importlib.reload(sys.modules[mod_name])

    import diggame, utils, components, managers, ui
    yield mock_instance, diggame, utils, components, managers, ui

    if original_pyxel:
        sys.modules['pyxel'] = original_pyxel
    elif 'pyxel' in sys.modules:
        del sys.modules['pyxel']

@pytest.fixture
def particle():
    return Particle(0, 0, 5)

@pytest.fixture
def block(mock_px_setup):
    return Block(0, 56, 1, 2)

@pytest.fixture
def lang_manager(tmp_path):
    lang_dir = tmp_path / "lang"
    lang_dir.mkdir()
    (lang_dir / "en_us.json").write_text(json.dumps({"_metadata": {"display_name": "English (US)"}, "test_key": "Hello"}))
    (lang_dir / "de_de.json").write_text(json.dumps({"_metadata": {"display_name": "Deutsch"}, "test_key": "Hallo"}))
    return LanguageManager(lang_folder=str(lang_dir), default_lang="en_us")

@pytest.fixture
def notification_manager():
    return NotificationManager(MockFontWriter())

@pytest.fixture
def persistence_manager():
    return PersistenceManager(MockGameObject())

@pytest.fixture
def button_box(mock_px_setup):
    _, _, _, _, _, ui = mock_px_setup
    return ui.ButtonBox(MockFontWriter(), MockLangManager())

# --- Test Cases ---

# Utils
def test_estimate_text_width():
    assert estimate_text_width("hello") == (FONT_SIZE / 2) * 5
    assert estimate_text_width("こんにちは") == FONT_SIZE * 5

def test_calculate_text_center_position():
    x, y = calculate_text_center_position(100, 50, "test")
    assert x == (100 - (FONT_SIZE / 2) * 4) / 2

# Game Logic (Coordinates)
def test_world_to_chunk_coords(mock_px_setup):
    _, _, utils, _, _, _ = mock_px_setup
    assert utils.world_to_chunk_coords(0, 0) == (0, 0)

# Components
def test_particle_gravity(particle):
    initial_vy = particle.vy
    particle.update([])
    assert particle.vy == initial_vy + particle.GRAVITY

def test_block_handle_click_reduces_hp(block):
    initial_hp = block.current_hp
    block.handle_click()
    assert block.current_hp == initial_hp - 1

# Managers
def test_language_manager_discovery(lang_manager):
    assert "en_us" in lang_manager.get_available_languages()

def test_language_manager_get_string(lang_manager):
    assert lang_manager.get_string("test_key") == "Hello"

def test_notification_manager_add_notification(notification_manager, monkeypatch):
    monkeypatch.setattr("ui.Notification", MockNotification)
    notification_manager.add_notification("test message")
    assert len(notification_manager.notifications) == 1

def test_persistence_manager_save(persistence_manager, monkeypatch):
    mock_file = io.StringIO()
    monkeypatch.setattr("builtins.open", lambda name, mode: MockFileContextManager(mock_file))
    monkeypatch.setattr("json.dump", lambda data, file, indent: file.write(json.dumps(data)))
    persistence_manager.save_game_state()
    saved_data = json.loads(mock_file.getvalue())
    assert saved_data["camera_x"] == 0

def test_persistence_manager_load(persistence_manager, monkeypatch):
    save_data = {"camera_x": 100, "camera_y": 200, "se_on": False, "bgm_on": False, "world_seed_main": 789, "world_seed_ore": 101, "generated_chunk_coords": [], "modified_chunks": [], "current_language": "de_de"}
    mock_file = io.StringIO(json.dumps(save_data))
    monkeypatch.setattr("builtins.open", lambda name, mode: MockFileContextManager(mock_file))
    persistence_manager.load_game_state()
    game = persistence_manager.game
    assert game.camera_x == 100
    assert game.world_manager.world_seed_main == 789
    assert game.current_language_code == "de_de"

# UI
def test_button_box_draw_button_not_pressed(button_box, mock_px_setup):
    mock_px, _, _, _, _, _ = mock_px_setup
    mock_px.mouse_x = 0
    is_released = button_box.draw_button(5, 5, 20, 20)
    assert not is_released

def test_button_box_draw_button_pressed(button_box, mock_px_setup):
    mock_px, _, _, _, _, _ = mock_px_setup
    mock_px.mouse_x = 10
    mock_px._btn_status[mock_px.MOUSE_BUTTON_LEFT] = True
    is_released = button_box.draw_button(5, 5, 20, 20)
    assert not is_released

def test_button_box_draw_button_released(button_box, mock_px_setup, monkeypatch):
    mock_px, _, _, _, _, _ = mock_px_setup
    mock_px.mouse_x = 10
    mock_px.mouse_y = 10 # Set y-coordinate for hover
    monkeypatch.setattr(mock_px, 'btnr', lambda key: key == mock_px.MOUSE_BUTTON_LEFT)
    is_released = button_box.draw_button(5, 5, 20, 20)
    assert is_released