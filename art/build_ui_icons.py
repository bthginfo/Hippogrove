"""Draw the Hippogrove HUD atlas. Run with Python 3 and Pillow."""
from pathlib import Path
from PIL import Image, ImageDraw

SIZE = 64
SCALE = 4
GOLD = '#f4ca74'
LIGHT = '#ffe7a8'
SHADE = '#b9803d'
EDGE = '#63462c'
TEAL = '#9ee2cb'


def icon(draw_icon):
    img = Image.new('RGBA', (SIZE * SCALE, SIZE * SCALE))
    d = ImageDraw.Draw(img)

    def coordinates(box):
        return tuple(round(x * SCALE) for x in box)

    def poly(points, fill):
        d.polygon([coordinates(point) for point in points], fill=fill)

    def line(points, fill, width=2):
        d.line([coordinates(point) for point in points], fill=fill, width=width * SCALE, joint='curve')

    def oval(box, fill, outline=None):
        d.ellipse(coordinates(box), fill, outline, width=SCALE * 2)

    def rect(box, fill, radius=4, outline=None):
        d.rounded_rectangle(coordinates(box), radius * SCALE, fill, outline, width=SCALE * 2)

    draw_icon(poly, line, oval, rect)
    return img.resize((SIZE, SIZE), Image.Resampling.LANCZOS)


def wood(p, l, o, r):
    for x, y in [(18, 21), (24, 32), (36, 30)]:
        p([(x-8,y+6),(x+17,y-7),(x+24,y+4),(x+3,y+20)], '#b57746')
        l([(x-3,y+6),(x+20,y-5)], LIGHT, 2)
        o((x-8,y+4,x+6,y+21), '#d3a36a', EDGE)
        o((x-4,y+8,x+2,y+17), '#ecd29b')


def stone(p, l, o, r):
    p([(8,39),(17,21),(29,16),(43,20),(56,40),(46,53),(21,54)], '#887c74')
    p([(17,21),(29,16),(43,20),(35,33),(15,33)], '#d9c9b7')
    p([(43,20),(56,40),(46,53),(35,33)], '#a2978c')
    p([(8,46),(15,37),(30,38),(39,50),(33,56),(14,57)], '#c1b1a0')
    p([(15,37),(30,38),(39,50),(29,49),(11,49)], '#e3d5c5')


def food(p, l, o, r):
    o((32,8,43,24), '#90b767')
    o((40,10,55,23), '#bccf75')
    o((42,21,56,30), '#7ea36c')
    p([(34,21),(46,28),(41,39),(21,53),(13,54),(16,43)], '#d88036')
    p([(34,21),(43,26),(32,40),(16,50),(16,43)], '#ffa451')
    l([(25,31),(31,36)], '#c17129')
    l([(20,41),(25,44)], '#c17129')


def prayer(p, l, o, r):
    o((22,24,42,44), GOLD, EDGE)
    p([(18,44),(45,44),(50,54),(13,54)], GOLD)
    l([(16,51),(47,51)], SHADE)
    for segment in [[(32,9),(32,17)],[(17,13),(21,19)],[(47,13),(43,19)],[(9,25),(15,27)],[(55,25),(49,27)]]:
        l(segment, LIGHT, 3)


def mana(p, l, o, r):
    p([(32,8),(50,32),(51,43),(44,52),(32,56),(20,52),(13,43),(14,32)], EDGE)
    p([(32,11),(47,33),(48,43),(41,50),(31,53),(21,49),(16,40),(19,32)], TEAL)
    p([(32,13),(47,36),(40,47),(29,48),(22,42),(35,34)], '#60bca2')
    l([(31,20),(24,32),(23,38)], '#efffed', 3)


def paw(p, l, o, r):
    for x, y, w in [(11,24,10),(22,13,12),(35,13,12),(46,24,10)]:
        o((x,y,x+w,y+15), GOLD, EDGE)
        o((x+2,y+2,x+7,y+8), LIGHT)
    o((16,32,49,53), GOLD, EDGE)
    o((24,35,41,46), LIGHT)


def hammer(p, l, o, r):
    p([(12,48),(35,23),(42,29),(22,55),(16,56)], EDGE)
    p([(14,48),(36,24),(39,29),(21,53),(16,54)], GOLD)
    p([(17,20),(30,8),(54,27),(43,40)], EDGE)
    p([(20,20),(30,12),(51,28),(42,37)], GOLD)
    p([(20,20),(30,12),(50,27),(36,27)], LIGHT)


def people(p, l, o, r):
    for x, y in [(9,25),(42,25),(25,14)]:
        o((x,y,x+14,y+15), GOLD, EDGE)
    for x, y, w in [(4,40,20),(40,40,20),(18,32,28)]:
        r((x,y,x+w,y+21 if w>20 else y+15), GOLD, 8, EDGE)
    l([(24,42),(24,47)], LIGHT, 3)


def hippo(p, l, o, r):
    for x in [17,39]:
        o((x,10,x+12,25), '#a994c0', '#554563')
        o((x+3,13,x+9,20), '#e8a6ae')
    r((12,17,53,50), '#a994c0', 15, '#554563')
    r((9,30,56,54), '#e9a7af', 13, '#794e68')
    for x in [23,39]:
        o((x,24,x+4,29), '#302338')
    for x in [19,42]:
        o((x,36,x+4,41), '#9f6170')
    l([(28,46),(35,46)], '#b67386')


def magic(p, l, o, r):
    p([(32,6),(37,24),(53,28),(38,35),(32,55),(26,36),(11,28),(27,23)], EDGE)
    p([(32,11),(35,25),(48,28),(36,33),(32,49),(28,34),(16,28),(29,24)], LIGHT)
    l([(49,13),(49,23)], GOLD, 2)
    l([(44,18),(54,18)], GOLD, 2)
    l([(16,38),(16,50)], GOLD, 2)
    l([(10,44),(22,44)], GOLD, 2)


def map_icon(p, l, o, r):
    p([(8,17),(24,10),(41,16),(55,11),(57,47),(41,54),(24,47),(8,53)], EDGE)
    p([(10,20),(24,13),(41,19),(53,14),(54,45),(41,51),(24,44),(11,49)], LIGHT)
    p([(24,13),(41,19),(41,51),(24,44)], GOLD)
    l([(15,34),(23,29),(31,34),(40,29),(49,32)], '#577a60', 3)


def scroll(p, l, o, r):
    r((15,11,49,52), LIGHT, 5, EDGE)
    r((11,7,53,17), GOLD, 5, EDGE)
    r((11,47,53,57), GOLD, 5, EDGE)
    l([(22,29),(28,35),(42,22)], '#52744d', 4)
    l([(25,42),(40,42)], SHADE, 2)


def book(p, l, o, r):
    for x in [9,31]:
        r((x,14,x+24,50), GOLD if x==9 else LIGHT, 5, EDGE)
    l([(32,19),(32,47)], SHADE)
    for y in [26,32,38]:
        l([(15,y),(25,y)], SHADE)
        l([(38,y),(48,y)], SHADE)


def chart(p, l, o, r):
    for x, y in [(10,36),(25,25),(40,12)]:
        r((x,y,x+13,53), GOLD, 3, EDGE)
        l([(x+4,y+4),(x+4,48)], LIGHT, 2)


def pin(p, l, o, r):
    p([(32,55),(15,33),(14,22),(20,13),(31,9),(43,13),(50,22),(49,33)], EDGE)
    p([(32,51),(18,32),(18,23),(24,15),(32,12),(41,16),(46,23),(46,32)], GOLD)
    o((24,19,40,35), '#304438')


def road(p, l, o, r):
    path = [(12,52),(17,42),(24,39),(37,36),(43,30),(44,21),(52,11)]
    l(path, EDGE, 15)
    l(path, LIGHT, 11)
    for box in [(6,46,18,57),(46,6,58,17)]:
        r(box, TEAL, 3, '#466550')
    for x,y in [(18,42),(31,38),(43,29),(45,19)]:
        l([(x-3,y-3),(x+3,y+3)], SHADE, 2)


def boards(p, l, o, r):
    for y in [35, 24]:
        p([(8,y+5),(34,y-12),(57,y-4),(57,y+5),(30,y+22),(8,y+14)], EDGE)
        p([(10,y+5),(34,y-10),(55,y-4),(29,y+12)], '#edbc72')
        p([(29,y+12),(55,y-4),(55,y+4),(29,y+19)], '#a56c42')
        p([(10,y+6),(29,y+12),(29,y+19),(10,y+13)], '#c89050')
        l([(17,y+8),(41,y-7)], '#f8d8a0', 2)


def blocks(p, l, o, r):
    for x, y in [(9, 39), (29, 38), (20, 23)]:
        p([(x,y),(x+16,y-9),(x+29,y-5),(x+29,y+9),(x+15,y+17),(x,y+11)], '#b2a295')
        p([(x,y),(x+16,y-9),(x+29,y-5),(x+15,y+4)], '#eadbba')
        p([(x+15,y+4),(x+29,y-5),(x+29,y+9),(x+15,y+17)], '#a5968c')
        l([(x+4,y+2),(x+13,y+6)], '#cdc0a4')


def mallet(p, l, o, r):
    l([(16,53),(39,29)], EDGE, 12)
    l([(16,51),(40,27)], '#c39259', 8)
    p([(20,24),(34,10),(58,19),(58,34),(44,45),(20,38)], EDGE)
    p([(22,23),(34,13),(55,20),(43,30)], '#eab77b')
    p([(43,30),(55,20),(55,33),(43,41)], '#9c663d')
    p([(22,25),(43,31),(43,42),(22,36)], '#c88c55')


def fruit_basket(p, l, o, r, fruit, outline, shine, long=False):
    o((10,23,55,58), '#ba8a56', EDGE)
    o((10,22,55,42), '#e3b579', EDGE)
    for x,y in [(18,23),(26,16),(35,20),(43,24),(28,29)]:
        o((x-5,y-4,x+6,y+(10 if long else 7)), fruit, outline)
        o((x-2,y-2,x+2,y+2), shine)
    o((39,13,54,22), '#79a560')
    l([(13,43),(26,46),(40,45),(52,42)], '#ebc08a')
    l([(16,50),(29,52),(46,48)], '#dfa875')


def berry_basket(p, l, o, r):
    fruit_basket(p, l, o, r, '#bb5147', '#773e36', '#e89479')


def frostberries(p, l, o, r):
    fruit_basket(p, l, o, r, '#626ca8', '#3e466b', '#9da7e1')


def dates(p, l, o, r):
    fruit_basket(p, l, o, r, '#b6723f', '#7b4a2f', '#dda76b', True)


def olives(p, l, o, r):
    fruit_basket(p, l, o, r, '#7a994b', '#455f35', '#c0c96f', True)


def rice(p, l, o, r):
    o((8,24,56,57), '#c0714f', EDGE)
    o((8,22,56,40), '#ecd5ac', EDGE)
    for x, y in [(15,29),(22,24),(25,32),(33,29),(38,24),(42,32),(49,29)]:
        o((x-3,y-2,x+4,y+3), '#fff0cf')
    l([(13,44),(24,48),(41,47),(51,44)], '#f2a26e', 2)


def bamboo(p, l, o, r):
    for x,y in [(10,39),(21,42),(26,32)]:
        p([(x,y),(x+31,y-24),(x+37,y-16),(x+7,y+8)], '#4f7850')
        l([(x+4,y),(x+34,y-20)], '#abc276', 2)
        o((x,y-1,x+8,y+8), '#c8ce81', '#657849')
        l([(x+12,y-8),(x+16,y-3)], '#8eaa5e', 2)
        l([(x+23,y-16),(x+28,y-10)], '#8eaa5e', 2)


def jade(p, l, o, r):
    p([(9,43),(17,22),(35,16),(52,24),(55,43),(44,53),(20,53)], '#4d7f67')
    p([(17,22),(35,16),(52,24),(43,31),(27,32)], '#ace0a2')
    p([(9,43),(27,32),(43,31),(44,53),(20,53)], '#71b58b')
    l([(17,40),(28,37),(39,38)], '#8ac99a', 2)


def pouch(p, l, o, r):
    p([(19,13),(43,13),(39,24),(49,33),(55,49),(48,56),(15,56),(9,49),(14,33),(25,24)], EDGE)
    p([(22,15),(40,15),(36,26),(47,36),(51,48),(45,53),(18,53),(13,48),(17,35),(28,26)], '#c69460')
    p([(22,36),(31,28),(42,33),(45,48),(27,51),(16,47)], GOLD)
    l([(24,24),(40,24)], SHADE, 5)
    l([(30,27),(25,36)], '#876440', 2)


if __name__ == '__main__':
    atlas = Image.new('RGBA', (4 * SIZE, 7 * SIZE))
    glyphs = [wood, stone, food, prayer, mana, paw, hammer, people, hippo, magic, map_icon, scroll, book, chart, pin, road,
              boards, blocks, mallet, berry_basket, pouch, frostberries, rice, dates, olives, bamboo, jade]
    for index, glyph in enumerate(glyphs):
        atlas.paste(icon(glyph), ((index % 4) * SIZE, (index // 4) * SIZE))
    out = Path(__file__).resolve().parent.parent / 'assets' / 'ui' / 'atlas.png'
    out.parent.mkdir(parents=True, exist_ok=True)
    atlas.save(out, optimize=True)
