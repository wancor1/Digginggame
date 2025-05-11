import pyxel as px
import random
import math
import time

# todo: 鉱石の追加 [鉄,コバルト,銅,金,銀,ダイヤ]
# todo: パーティクルの最大数を設定する [0~inf]

# wip: menuつくる [設定,終了]

# bug: 描画重すぎ [パーティクル,カメラ動かすときのブロックの描画]

px.init(160, 120, title="Break", fps=60)
px.load('sprite_sheet.pyxres')

class Selectblock: #ff0000
	def __init__(self):
		self.start_time = time.time()
	def update(self):
		global cur_block_check
		if not cur_block_check:
			elapsed_time = time.time() - self.start_time
			if elapsed_time >= 0.1:
				self.start_time = time.time()
	def select(self):
		mx=px.mouse_x
		my=px.mouse_y
		bmx=math.floor(mx/8)*8
		bmy=math.floor(my/8)*8
		elapsed_time = time.time() - self.start_time
		if elapsed_time <= 1:
			px.blt(bmx,bmy,0,0,0,8,8,0)
		else:
			px.blt(bmx-1,bmy-1,0,0,8,10,10,0)
		if elapsed_time >= 2:
			self.start_time = time.time()
	def draw(self):
		global cur_block_check
		self.update()
		if not is_menu and cur_block_check:
			self.select()

class Buttonbox():#ff7f00
	def C_boxtextpos(self, w, h, text):
		#textをbox中央に配置するための計算
		#tc(TextCount) = 文字の数
		#tuXp(TextUseXPixel) = tc*3+(tc-1)
		tc = len(text)
		tuXp = tc*3+(tc-1)
		th = h/2-2
		tw = (w-tuXp)/2
		return tw, th

	def C_cursolstat(self, x, y, w, h):
		if px.mouse_x>=x and px.mouse_x<=x+w-1 and px.mouse_y>=y and px.mouse_y<=y+h-1:
			hover = True
		else:
			hover = False
		if px.btn(px.MOUSE_BUTTON_LEFT):
			press = True
		else:
			press = False
		if px.btnr(px.MOUSE_BUTTON_LEFT) and hover:
			r_press = True
		else:
			r_press = False
		if hover and press:
			hovpress = True
		else:
			hovpress = False
		return hovpress, hover, press, r_press

	def D_box(self, x, y, w, h, text='text', button:bool=False, pressedtext='press', bpr:bool=False):
		if button==True:
			hovpress,hover,press,r_press=self.C_cursolstat(x, y, w, h)
			if hovpress == True:
				px.rect(x,y, w, h, 13)
				px.rectb(x,y, w, h, 7)
				tw,th = self.C_boxtextpos(w, h, pressedtext)
				px.text(x+tw, y+th, pressedtext, 7)
			else:
				px.rect(x,y, w, h, 13)
				px.rectb(x,y, w, h, 7)
				px.line(x+w-1,y+1, x+w-1, y+h-1, 0)
				px.line(x+1,y+h-1, x+w-1, y+h-1, 0)
				tw,th = self.C_boxtextpos(w, h, text)
				px.text(x+tw, y+th, text, 7)
			if bpr:
				return hovpress
			else:
				return r_press
		else:
			px.rect(x,y, w, h, 13)
			px.rectb(x,y, w, h, 7)
			tw,th = self.C_boxtextpos(w, h, text)
			px.text(x+tw, y+th, text, 7)

class Particle: #ffff00
	def __init__(self, x, y, click_max):
		self.x = x + 5
		self.y = y + 5
		angle = random.uniform(0, 2 * math.pi)
		speed = random.uniform(20, 60) / 60
		self.vx = math.cos(angle) * speed
		self.vy = math.sin(angle) * speed - 1.5
		self.alive = True
		self.timer = 0
		self.landed_time = None
		if click_max <= 5:
			if random.random() < 0.9:
				self.color = 9
			else:
				self.color = 13
		elif click_max <= 10:
			if random.random() < 0.9:
				self.color = 13
			else:
				self.color = 6
		else:
			self.color = 0

	def update(self, blocks):
		if not self.alive:
			return
		self.vy += 0.19  # 重力
		self.x += self.vx
		for b in blocks:
			if b.broken:
				continue
			if b.x <= self.x < b.x + 8 and b.y <= self.y < b.y + 8:
				if self.vx > 0:  # 右に移動中
					self.x = b.x - 0.1
				else:  # 左に移動中
					self.x = b.x + 8.1
				self.vx *= -0.4
		self.y += self.vy
		on_ground = False
		for b in blocks:
			if b.broken:
				continue
			if b.x <= self.x < b.x + 8 and b.y <= self.y < b.y + 8:
				if self.vy > 0:  # 地面
					self.y = b.y - 0.1
					self.vy = 0
					self.vx *= 0.85
					on_ground = True
				elif self.vy < 0:  # 天井
					self.y = b.y + 8.1
					self.vy = 0
		if on_ground:
			if self.landed_time is None:
				self.landed_time = time.time()
			elif time.time() - self.landed_time > 5:
				self.alive = False
		else:
			self.landed_time = None


	def draw(self):
		if self.alive:
			px.pset(int(self.x), int(self.y), self.color)

class Block: #7fff00
	def __init__(self, x, y):
		self.x = x
		self.y = y
		self.block_state = 0
		self.scale = 0.01
		self.B_HARD_MIN= 3
		self.B_HARD_MAX= 10
		self.click_max = int(math.floor((self.B_HARD_MAX-self.B_HARD_MIN)*abs(px.noise(x*self.scale ,y*self.scale))+self.B_HARD_MIN))
		self.click_count = self.click_max
		self.broken = False
		if self.click_max <= 5:
			self.block_state = 1
		elif self.click_max <= 10:
			self.block_state = 3
		else:
			self.block_state = 0
		if self.y == 56 :
			self.block_state = 2
		px.nseed(seed2)
		if self.block_state == 3 and px.noise(self.x*(self.scale*4),self.y*(self.scale*4)) >= 0.4:
			self.block_state = 4
		px.nseed(seed1)

	def C_boxtextpos(self, w, h, text):
		tc = len(text)
		tuXp = tc * 3 + (tc - 1)
		th = h / 2 - 2
		tw = (w - tuXp) / 2
		return tw, th

	def breakdur(self):
		if self.click_count == self.click_max:
			return 0
		diff = self.click_max - self.click_count
		ans = math.ceil(diff / (self.click_max / 5))-1
		if ans == 0:
			ans = 1
		return ans

	def draw(self):
		global seed1,seed2
		if self.broken:
			return
		if (camera_x <= self.x < camera_x + px.width) and (camera_y <= self.y < camera_y + px.height):
			if self.block_state == 1:
				px.blt(self.x, self.y, 1, 8, 0, 8, 8, 0)  # dirt
			elif self.block_state == 2:
				px.blt(self.x, self.y, 1, 16, 0, 8, 8, 0) # grass
			elif self.block_state == 3:
				px.blt(self.x, self.y, 1, 8, 8, 8, 8, 0)  # stone
			elif self.block_state == 4:
				px.blt(self.x, self.y, 1, 16, 8, 8, 8, 1) # coal
			else:
				px.blt(self.x, self.y, 0, 8, 0, 8, 8, 1) # error
			now_break = self.breakdur()
			if now_break != 0:
				now_break_p = (now_break) * 8
				px.blt(self.x, self.y, 1, 0, now_break_p, 8, 8, 11)
			if is_debug:
				tx, ty = self.C_boxtextpos(8, 8, f'{self.click_count}')
				if self.click_count != self.click_max:
					self.hp_noww = self.click_count / self.click_max * 6
					px.line(self.x + 1, self.y + 6, self.x + 6, self.y + 6, 13)
					px.line(self.x + 1, self.y + 6, self.x + self.hp_noww, self.y + 6, 3)
				if self.click_count == self.click_max:
					px.text(int(tx) + self.x, int(ty) + self.y, f'{self.click_count}', 7)
				else:
					px.text(int(tx) + self.x, int(ty) + self.y - 1, f'{self.click_count}', 7)
					
	def check_click(self, mx, my):
		mx += camera_x
		my += camera_y
		if self.broken:
			return []
		if self.x <= mx < self.x + 8 and self.y <= my < self.y + 8:
			self.click_count -= 1
			if self.click_count <= 0:
				px.play(0, 1)
				self.broken = True
				num_particles = int(min(15, max(5, random.gauss(10, 2))))
				return [Particle(self.x, self.y, self.click_max) for _ in range(num_particles)]
			else:
				px.play(0, 0)
		return []
	
	def check_hover(self, mx, my):
		global cur_block_check
		mx += camera_x
		my += camera_y
		if not cur_block_check and not self.broken:
			if self.x <= mx < self.x + 8 and self.y <= my < self.y + 8:
				cur_block_check = True

class Menu: #00ff00
	def __init__(self):
		#self.is_menu:bool = False
		self.is_mainmenu:bool = False 
		self.is_menudic:bool = False
		self.menu_main_w = 60
		self.menu_main_h = px.height-(px.height/10*2)
		self.menu_main_x = (px.width-self.menu_main_w)/2
		self.menu_main_y = (px.height-self.menu_main_h)/2
	def update(self):
		pass
	def draw(self):
		if is_menu:
			px.rect(self.menu_main_x,self.menu_main_y,self.menu_main_w,self.menu_main_h,9)
			px.rectb(self.menu_main_x,self.menu_main_y,self.menu_main_w,self.menu_main_h,0)
			px.pset(0,0,5)

#ff0000
h8:int = px.height
w8:int = px.width
blocks = [] #Block(i * 8, j * 8) for j in range(int(h8)) for i in range(int(w8))
particles = []

select_block = Selectblock()
buttonbox = Buttonbox()
menu = Menu()
block_coords = set()

cur_block_check:bool = False
title_button:bool = False
is_title:bool = True
is_menu:bool = False
is_debug:bool = False
moved:bool = False
first_generate:bool = False

camera_x, camera_y = 0, 0
press_W, press_A, press_S, press_D = 0, 0, 0, 0
press_W_start_time = time.time()
press_A_start_time = time.time()
press_S_start_time = time.time()
press_D_start_time = time.time()

seed1 = px.rndi(1,2147483647)
seed2 = px.rndi(1,2147483647)
px.nseed(seed1)
px.rseed(seed1)
TITLE = 'Digging Game'

def generate_blocks_if_needed():
	global blocks
	min_x = math.floor(camera_x / 8)
	max_x = math.ceil((camera_x + px.width) / 8)
	#min_y = math.floor(camera_y / 8)
	min_y = 7
	max_y = math.ceil((camera_y + px.height) / 8)
	for x in range(min_x, max_x):
		for y in range(min_y, max_y):
			if not any(b.x == x * 8 and b.y == y * 8 for b in blocks):
				blocks.append(Block(x * 8, y * 8))

def C_boxtextpos(w, h, text):
	tc = len(text)
	tuXp = tc * 3 + (tc - 1)
	th = h / 2 - 2
	tw = (w - tuXp) / 2
	return tw, th

def D_is_title():
	global title_button
	px.cls(1)
	for i in range(1, -1, -1):
		color = 11 if i == 0 else 3
		t_tw, t_th = C_boxtextpos(px.width, px.height, TITLE)
		px.text(int(t_tw + i), int(t_th * 0.5), TITLE, color)
	title_button = buttonbox.D_box((px.width-35)/2,(px.height-10)/2*1.25,35,10,'Start',True,'Start',False)

def draw():
	global is_title, camera_x, camera_y
	if is_title:
		px.camera(0, 0)
		D_is_title()
	else:
		px.camera(camera_x, camera_y)
		px.cls(1)
		for b in blocks:
			if (camera_x <= b.x < camera_x + px.width) and (camera_y <= b.y < camera_y + px.height):
				b.draw()
		for p in particles:
			p.draw()
		menu.draw()
		px.camera(0, 0)
		select_block.draw()
	px.camera(0, 0)
	D_cursol()

def D_cursol():
	mouce_push = 1 if px.btn(px.MOUSE_BUTTON_LEFT) else 0
	px.blt(px.mouse_x, px.mouse_y + mouce_push, 0, 16, 0, 8, 8, 0)

def update():
	global is_menu,is_debug,is_title,cur_block_check,title_button,camera_x,camera_y,moved,first_generate,press_W,press_A,press_S,press_D,press_W_start_time,press_A_start_time,press_S_start_time,press_D_start_time
	if is_title:
		if title_button:
			is_title = False
		return
	if not first_generate:
		generate_blocks_if_needed()
		first_generate = True
	if px.btn(px.KEY_A):
		press_A = time.time() - press_A_start_time
		if press_A >= 0.5:
			press_A_start_time += 0.1
			if px.btn(px.KEY_SHIFT):
				camera_x -= 16
			else:
				camera_x -= 8
		moved = True
	if px.btn(px.KEY_D):
		press_D = time.time() - press_D_start_time
		if press_D >= 0.5:
			press_D_start_time += 0.1
			if px.btn(px.KEY_SHIFT):
				camera_x += 16
			else:
				camera_x += 8
		moved = True
	if px.btn(px.KEY_W):
		press_W = time.time() - press_W_start_time
		if press_W >= 0.5:
			press_W_start_time += 0.1
			if px.btn(px.KEY_SHIFT):
				camera_y -= 16
			else:
				camera_y -= 8
		moved = True
	if px.btn(px.KEY_S):
		press_S = time.time() - press_S_start_time
		if press_S >= 0.5:
			press_S_start_time += 0.1
			if px.btn(px.KEY_SHIFT):
				camera_y += 16
			else:
				camera_y += 8
		moved = True
	if px.btnp(px.KEY_W):
		if px.btn(px.KEY_SHIFT):
			camera_y -= 16
		else:
			camera_y -= 8
		press_W_start_time = time.time()
	if px.btnp(px.KEY_A):
		if px.btn(px.KEY_SHIFT):
			camera_x -= 16
		else:
			camera_x -= 8
		press_A_start_time = time.time()
	if px.btnp(px.KEY_S):
		if px.btn(px.KEY_SHIFT):
			camera_y += 16
		else:
			camera_y += 8
		press_S_start_time = time.time()
	if px.btnp(px.KEY_D):
		if px.btn(px.KEY_SHIFT):
			camera_x += 16
		else:
			camera_x += 8
		press_D_start_time = time.time()
	if px.btnr(px.KEY_W):
		press_W_start_time = time.time()
	if px.btnr(px.KEY_A):
		press_A_start_time = time.time()
	if px.btnr(px.KEY_S):
		press_S_start_time = time.time()
	if px.btnr(px.KEY_D):
		press_D_start_time = time.time()
	if px.btnp(px.KEY_F3):
		is_debug = not is_debug
	if px.btnp(px.KEY_M):
		is_menu = not is_menu

	if not is_menu:
		if moved:
			generate_blocks_if_needed()
			moved = False
		mx, my = px.mouse_x, px.mouse_y
		cur_block_check = False
		for b in blocks:
			b.check_hover(mx,my)
		if px.btnp(px.MOUSE_BUTTON_LEFT):
			for b in blocks:
				new_particles = b.check_click(mx, my)
				particles.extend(new_particles)

	for p in particles:
		p.update(blocks)

	particles[:] = [p for p in particles if p.alive]

px.run(update, draw)
