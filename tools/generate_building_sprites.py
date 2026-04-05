"""Generate 32x32 pixel art building sprites for WorldSim."""
from PIL import Image
import os

os.makedirs('assets/sprites/buildings', exist_ok=True)

# ========== CAMPFIRE (32x32) ==========
cf = Image.new('RGBA', (32, 32), (0,0,0,0))
stone = (110, 105, 95, 255)
for x, y in [(12,16),(20,16),(16,12),(16,20),(12,13),(20,13),(13,19),(19,19),
             (11,15),(21,15),(11,17),(21,17),(14,11),(18,11),(14,21),(18,21)]:
    cf.putpixel((x, y), stone)
    cf.putpixel((x+1, y), stone)
log_dark = (80, 50, 25, 255)
log_mid = (100, 65, 30, 255)
for x in range(13, 20):
    cf.putpixel((x, 16), log_dark)
    cf.putpixel((x, 17), log_mid)
for y in range(13, 20):
    cf.putpixel((16, y), log_dark)
    cf.putpixel((17, y), log_mid)
fire_colors = [
    ((255, 200, 50, 255), [(15,14),(16,14),(17,14),(15,15),(16,15),(17,15),(16,13)]),
    ((255, 120, 30, 255), [(14,14),(18,14),(14,15),(18,15),(15,13),(17,13),(16,12),(15,16),(17,16)]),
    ((220, 60, 20, 255), [(14,13),(18,13),(15,12),(17,12),(16,11),(14,16),(18,16)]),
]
for color, pixels in fire_colors:
    for x, y in pixels:
        cf.putpixel((x, y), color)
glow = (255, 150, 50, 30)
for x in range(10, 23):
    for y in range(10, 23):
        dx, dy = x - 16, y - 16
        if dx*dx + dy*dy < 36 and cf.getpixel((x,y))[3] == 0:
            cf.putpixel((x, y), glow)
cf.save('assets/sprites/buildings/campfire.png')

# ========== SHELTER (32x32) ==========
sh = Image.new('RGBA', (32, 32), (0,0,0,0))
floor_color = (140, 115, 70, 200)
for x in range(4, 28):
    for y in range(14, 28):
        sh.putpixel((x, y), floor_color)
wall_dark = (90, 60, 30, 255)
wall_mid = (120, 80, 40, 255)
for y in range(8, 28):
    sh.putpixel((4, y), wall_dark)
    sh.putpixel((5, y), wall_mid)
    sh.putpixel((27, y), wall_dark)
    sh.putpixel((26, y), wall_mid)
for x in range(4, 28):
    sh.putpixel((x, 14), wall_dark)
    sh.putpixel((x, 15), wall_mid)
roof_dark = (130, 100, 40, 255)
roof_mid = (160, 125, 55, 255)
roof_light = (180, 145, 70, 255)
for row_off in range(0, 7):
    y = 7 + row_off
    left = 4 + row_off
    right = 27 - row_off
    for x in range(left, right + 1):
        c = roof_dark if (x+row_off)%3==0 else (roof_mid if (x+row_off)%3==1 else roof_light)
        sh.putpixel((x, y), c)
for x in range(13, 19):
    sh.putpixel((x, 6), roof_dark)
    sh.putpixel((x, 5), roof_mid)
door = (60, 40, 20, 180)
for x in range(13, 19):
    for y in range(20, 28):
        sh.putpixel((x, y), door)
for y in range(19, 28):
    sh.putpixel((12, y), wall_dark)
    sh.putpixel((19, y), wall_dark)
for x in [13,14,17,18]:
    sh.putpixel((x, 19), wall_dark)
bed = (100, 75, 45, 200)
for x in range(7, 12):
    for y in range(18, 24):
        sh.putpixel((x, y), bed)
sh.save('assets/sprites/buildings/shelter.png')

# ========== STOCKPILE (32x32) ==========
sp = Image.new('RGBA', (32, 32), (0,0,0,0))
ground = (130, 110, 75, 180)
for x in range(3, 29):
    for y in range(5, 29):
        sp.putpixel((x, y), ground)
post = (85, 60, 30, 255)
for px_x in [3, 15, 28]:
    for y in range(5, 29):
        sp.putpixel((px_x, y), post)
for py_y in [5, 28]:
    for x in range(3, 29):
        sp.putpixel((x, py_y), post)
wood_colors = [(100,65,30,255),(80,50,20,255),(115,75,35,255)]
for x in range(5, 14):
    for y in range(7, 17):
        sp.putpixel((x, y), wood_colors[(x+y)%3])
log_end = (70, 45, 20, 255)
log_ring = (90, 60, 25, 255)
for cx, cy in [(7,9),(11,9),(7,13),(11,13),(9,11)]:
    sp.putpixel((cx, cy), log_end)
    for dx, dy in [(-1,0),(1,0),(0,-1),(0,1)]:
        nx, ny = cx+dx, cy+dy
        if 5<=nx<14 and 7<=ny<17:
            sp.putpixel((nx, ny), log_ring)
stone_colors = [(140,135,125,255),(120,115,105,255),(160,155,145,255)]
for x in range(17, 27):
    for y in range(7, 17):
        sp.putpixel((x, y), stone_colors[(x*3+y*7)%3])
basket = (150, 120, 60, 255)
basket_dark = (120, 95, 45, 255)
for x in range(6, 14):
    for y in range(19, 26):
        sp.putpixel((x, y), basket if (x+y)%2==0 else basket_dark)
food = (180, 60, 40, 255)
food2 = (120, 160, 50, 255)
for cx, cy in [(8,21),(10,20),(12,22),(9,23),(11,21)]:
    sp.putpixel((cx, cy), food)
for cx, cy in [(7,22),(11,23),(10,22)]:
    sp.putpixel((cx, cy), food2)
tool = (100, 80, 50, 255)
tool_head = (150, 140, 130, 255)
for y in range(19, 27):
    sp.putpixel((20, y), tool)
sp.putpixel((20, 19), tool_head)
sp.putpixel((20, 18), tool_head)
for y in range(20, 27):
    sp.putpixel((24, y), tool)
for x in [23, 25]:
    sp.putpixel((x, 20), tool_head)
    sp.putpixel((x, 21), tool_head)
sp.save('assets/sprites/buildings/stockpile.png')

print("Generated: campfire.png, shelter.png, stockpile.png (all 32x32)")
