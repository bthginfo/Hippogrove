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


if __name__ == '__main__':
    atlas = Image.new('RGBA', (4 * SIZE, 4 * SIZE))
    glyphs = [wood, stone, food, prayer, mana, paw, hammer, people, hippo, magic, map_icon, scroll, book, chart, pin, road]
    for index, glyph in enumerate(glyphs):
        atlas.paste(icon(glyph), ((index % 4) * SIZE, (index // 4) * SIZE))
    out = Path(__file__).resolve().parent.parent / 'assets' / 'ui' / 'atlas.png'
    out.parent.mkdir(parents=True, exist_ok=True)
    atlas.save(out, optimize=True)
