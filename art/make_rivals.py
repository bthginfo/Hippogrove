"""Sculpt the five Hippogrove rivals. Run with blender -b -P art/make_rivals.py."""

import math
from pathlib import Path

import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[1]
MODELS = ROOT / "assets/models/rivals"


def reset():
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete(use_global=False)
    for data in list(bpy.data.materials):
        bpy.data.materials.remove(data)


def paint(name, color, roughness=.81, metal=0., glow=0.):
    def channel(c):
        c /= 255.
        return ((c + .055) / 1.055) ** 2.4 if c > .04045 else c / 12.92
    rgba = tuple(channel(int(color[n:n + 2], 16)) for n in (0, 2, 4)) + (1.,)
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = rgba
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metal
    if glow:
        bsdf.inputs["Emission Color"].default_value = rgba
        bsdf.inputs["Emission Strength"].default_value = glow
    mat.diffuse_color = rgba
    return mat


def space(p):
    return (p[0], -p[2], p[1])


def surface(name, vertices, faces, material):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([space(point) for point in vertices], [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    mesh.materials.append(material)
    return obj


def loft(name, points, material, sides=10):
    """Lay out planes around a spine. Alternating diagonals keep long bends readable."""
    vertices, faces = [], []
    for i, (center, across, depth) in enumerate(points):
        before = Vector(points[max(i - 1, 0)][0])
        after = Vector(points[min(i + 1, len(points) - 1)][0])
        direction = (after - before).normalized()
        width = Vector((1., 0., 0.))
        if abs(direction.dot(width)) > .92:
            width = Vector((0., 1., 0.))
        across_axis = (width - direction.dot(width) * direction).normalized()
        depth_axis = direction.cross(across_axis)
        for k in range(sides):
            angle = math.tau * (k + .5) / sides
            vertices.append(Vector(center)
                            + math.cos(angle) * across * across_axis
                            + math.sin(angle) * depth * depth_axis)
    for i in range(len(points) - 1):
        for k in range(sides):
            a, b = i * sides + k, i * sides + (k + 1) % sides
            if (i + k) % 2:
                faces.extend(((a, b, a + sides), (b, b + sides, a + sides)))
            else:
                faces.extend(((a, b, b + sides), (a, b + sides, a + sides)))
    faces.extend((tuple(reversed(range(sides))), tuple(range(len(vertices) - sides, len(vertices)))))
    return surface(name, vertices, faces, material)


def oval(name, at, size, material, roll=0.):
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2, radius=1., location=space(at))
    obj = bpy.context.object
    obj.name = name
    obj.scale = (size[0], size[2], size[1])
    obj.rotation_euler[1] = roll
    obj.data.materials.append(material)
    return obj


def blade(name, start, end, width, thickness, material, curve=0.):
    origin, tip = Vector(start), Vector(end)
    bend = Vector((0., curve, 0.))
    return loft(name, [
        (origin, .009, .012),
        (origin.lerp(tip, .33) + .75 * bend, width, thickness),
        (origin.lerp(tip, .68) + bend, width * .65, thickness * .57),
        (tip, .002, .003),
    ], material, 6)


def feather(name, start, end, width, surface_paint, shaft_paint):
    """Model a pointed vane with a raised shaft and folded outer planes."""
    base, tip = Vector(start), Vector(end)
    direction = (tip - base).normalized()
    side = direction.cross(Vector((0., 1., 0.))).normalized()
    normal = side.cross(direction).normalized()
    near = base.lerp(tip, .26)
    far = base.lerp(tip, .68)
    vertices = [
        base,
        near - side * width,
        far - side * width * .75,
        tip,
        far + side * width * .68,
        near + side * width * .88,
        near + normal * width * .42,
        far + normal * width * .26,
        near - normal * width * .15,
    ]
    faces = [
        (0, 6, 1), (1, 6, 7), (1, 7, 2), (2, 7, 3),
        (0, 5, 6), (5, 7, 6), (5, 4, 7), (4, 3, 7),
        (0, 1, 8), (1, 2, 8), (2, 3, 8), (3, 4, 8), (4, 5, 8), (5, 0, 8),
    ]
    obj = surface(name, vertices, faces, surface_paint)
    obj.data.materials.append(shaft_paint)
    for index in [1, 2, 6, 7]:
        obj.data.polygons[index].material_index = 1
    return obj


def thread(name, centers, radii, material, sides=7):
    return loft(name, [(center, radius, radius) for center, radius in zip(centers, radii)], material, sides)


def keen_eyes(at, spread, rim, pupil, spark):
    x, y, z = at
    for side in (-1, 1):
        eye = Vector((x + side * spread, y, z))
        depth = Vector((side * .76, .08, .65)).normalized()
        horizontal = Vector((.65, -.10 * side, -.76 * side))
        vertical = Vector((0., 1., 0.))
        axes = [(-.102, .006), (-.040, -.038), (.043, -.035), (.098, .023), (.037, .046)]
        socket = [eye + u * horizontal + v * vertical for u, v in axes]
        socket.append(eye + depth * .022)
        triangles = [(i, (i + 1) % 5, 5) for i in range(5)]
        surface("Carved angular eye socket", socket, triangles, rim)
        focus = eye + depth * .024 + horizontal * .012
        lid = [focus + u * horizontal + v * vertical for u, v in
               [(-.067, .008), (-.021, -.018), (.041, -.015), (.063, .018), (.004, .026)]]
        surface("Inset iris", lid, [(0, 1, 4), (1, 2, 4), (2, 3, 4)], pupil)
        shard = focus + depth * .004
        surface("Eye reflection", [shard + horizontal * u + vertical * v
                                  for u, v in [(-.026, .011), (-.009, -.004), (.009, .014)]],
                [(0, 1, 2)], spark)


def quadruped_leg(side, z, shoulder, fur, foot, hoof=False):
    q = side
    forward = 1 if z > 0. else -1
    x = .33 if z > 0. else .35
    end = z + .13 * forward + (.08 if side == 1 else -.04)
    loft("Shaped leg", [
        ((q * x * .83, shoulder, z), .14, .17),
        ((q * x, .67 * shoulder, z - .15 * forward), .115, .13),
        ((q * x * 1.08, .38 * shoulder, z + .14 * forward), .067 if hoof else .102, .09),
        ((q * x * 1.16, .18, end), .065 if hoof else .095, .081),
        ((q * x * 1.18, .095, end + .04), .064 if hoof else .11, .088),
    ], fur, 7)
    oval("Hoof" if hoof else "Paw", (q * x * 1.19, .102, end + .04),
         (.094 if hoof else .135, .098, .158 if hoof else .195), foot)


def deer():
    bark = paint("Chestnut coat", "86583e")
    rich = paint("Brow and hooves", "3e3027")
    cream = paint("Sand at the throat", "cba674")
    velvet = paint("Pale antler wood", "baa280")
    moss = paint("Forest moss", "556c47")
    fern = paint("Fern tips", "a2a468")
    eye = paint("Deep amber", "9a6637", .80)
    glint = paint("Pale honey", "d0ab73")
    loft("High shoulders and tucked flanks", [
        ((0., 1.26, -1.18), .05, .07), ((0., 1.29, -.86), .32, .43),
        ((0., 1.23, -.46), .39, .45), ((0., 1.31, .10), .38, .49),
        ((0., 1.46, .53), .35, .50), ((0., 1.50, .77), .23, .31),
        ((0., 1.55, .93), .09, .12)], bark, 9)
    loft("Raised neck", [((0., 1.41, .37), .29, .38), ((0., 1.78, .59), .27, .32),
        ((0., 2.06, .79), .21, .26), ((0., 2.35, .97), .17, .25),
        ((0., 2.48, 1.05), .16, .18)], bark, 8)
    blade("Throat blaze", (0., 1.56, .83), (0., 2.39, 1.24), .15, .051, cream)
    loft("Chiseled deer face", [((0., 2.48, .86), .13, .16), ((0., 2.51, 1.12), .252, .24),
        ((0., 2.41, 1.40), .178, .151), ((0., 2.29, 1.63), .116, .082),
        ((0., 2.28, 1.77), .073, .047)], bark, 8)
    blade("Nose stripe", (0., 2.65, 1.20), (0., 2.334, 1.761), .074, .032, rich)
    oval("Nose", (0., 2.294, 1.748), (.094, .050, .058), rich)
    keen_eyes((0., 2.545, 1.286), .185, rich, eye, glint)
    blade("Rump and short tail", (0., 1.34, -1.01), (0., 1.38, -1.48), .123, .104, cream, .17)
    for s in (-1, 1):
        quadruped_leg(s, -.66, 1.15, bark, rich, hoof=True)
        quadruped_leg(s, .50, 1.25, bark, rich, hoof=True)
        blade("Outer ear", (s*.19, 2.62, 1.08), (s*.58, 2.83, .91), .105, .062, bark)
        blade("Ear velvet", (s*.23, 2.65, 1.123), (s*.54, 2.79, .980), .060, .028, cream)
        beam = [(s*.15, 2.66, 1.04), (s*.25, 2.83, .95), (s*.45, 3.03, .79),
                (s*.64, 3.16, .63), (s*.77, 3.41, .47),
                (s*(.91 if s>0 else .85), 3.63 if s>0 else 3.53, .35)]
        thread("Antler beam", beam, [.077, .073, .062, .050, .029, .006], velvet)
        for j, offsets in [(1, (.16, .31)), (2, (.22, .36)), (3, (.11, .36))]:
            x, y, z = beam[j]
            wide, rise = offsets
            thread("Brow tine", [(x, y, z), (x + s * wide, y + rise*.62, z + .18),
                                (x + s * wide*.85, y + rise, z + .25)], [.05, .033, .006], velvet)
        for j in range(5):
            z = -.89 + .25*j
            blade("Layer of woodland foliage", (s*.13, 1.73 + j*.03, z),
                  (s*(.32 + (j % 2)*.12), 1.57 + j*.03, z-.21), .13, .045,
                  fern if j%3 == 0 else moss, .045)
        blade("Fern along the shoulder", (s*.29, 1.71, .32), (s*.36, 1.49, -.19), .12, .041, moss)
        for j in range(3):
            x, y, z = beam[1+j]
            blade("Antler fern", (x, y-.035, z), (x+s*.19, y+.08, z-.18), .047, .018, moss, .025)


def wolf():
    frost = paint("Slate winter coat", "596a79")
    mane = paint("Basalt guard fur", "2c414c")
    throat = paint("Ash undercoat", "98a9ad")
    mask = paint("Midnight nose and claws", "252d35")
    rune = paint("Frost inlay", "79a9a7", .84)
    spark = paint("Glacier glint", "ccd2b8")
    amber = paint("Wolf iris", "9c7a3d")
    loft("Powerful back and haunch", [
        ((0., .98, -1.36), .06, .065), ((0., 1.06, -1.10), .31, .38),
        ((0., 1.08, -.72), .43, .46), ((0., 1.14, -.25), .415, .43),
        ((0., 1.24, .34), .48, .56), ((0., 1.27, .71), .445, .58),
        ((0., 1.24, .99), .267, .35), ((0., 1.27, 1.13), .13, .13)], frost, 9)
    loft("Broad forehead and long jaw", [
        ((0., 1.44, .57), .18, .25), ((0., 1.51, .88), .43, .37),
        ((0., 1.50, 1.14), .385, .32), ((0., 1.36, 1.38), .226, .18),
        ((0., 1.32, 1.70), .158, .11), ((0., 1.35, 1.84), .10, .085)], frost, 8)
    loft("Pale cut of the lower jaw", [((0., 1.27, 1.05), .18, .11),
        ((0., 1.23, 1.32), .20, .11), ((0., 1.25, 1.61), .16, .079),
        ((0., 1.28, 1.77), .109, .045)], throat, 7)
    oval("Nose", (0., 1.355, 1.83), (.130, .064, .069), mask)
    keen_eyes((0., 1.580, 1.285), .245, mask, amber, spark)
    blade("Obsidian brow", (0., 1.735, .98), (0., 1.47, 1.60), .207, .095, mane)
    thread("Curled winter tail", [(0., 1.04, -1.10), (.13, .83, -1.49),
                                 (.24, .69, -1.91), (.42, .84, -2.25), (.49, 1.05, -2.40)],
           [.16, .22, .21, .126, .009], mane, 8)
    for side in (-1, 1):
        quadruped_leg(side, .63, 1.19, frost, mask)
        quadruped_leg(side, -.77, 1.08, frost, mask)
        blade("High pointed ear", (side*.254, 1.61, .86), (side*.42, 2.102, .735), .163, .098, mane, .07)
        blade("Inner ear", (side*.263, 1.78, .955), (side*.393, 2.034, .81), .084, .041, throat)
        for j in range(5):
            x, y, z = side*(.26 + .042*j), 1.73 - .158*j, .88 - .026*j
            blade("Faceted ruff", (x,y,z), (x+side*(.23-.025*j), y-.26, z-.68),
                  .159, .076, throat if j>1 else mane, .019)
        blade("Shoulder mantle", (side*.21, 1.69, .69), (side*.53, 1.16, -.27),
              .192, .085, mane, .13)
        for j in range(4):
            z = .12 - j*.27
            blade("Guard fur ridge", (side*.20, 1.64-j*.035, z), (side*.44, 1.32, z-.33),
                  .114, .046, mane, .055)
        p = side*.477
        thread("Rune inlay", [(p,1.29,.31),(p*1.02,1.54,.22),(p,1.36,.04),
                             (p*.97,1.20,.21)], [.027,.027,.026,.021], rune)
        thread("Rune crossbar", [(p,1.36,.36),(p,1.39,.04)], [.021,.022], rune)
    thread("Forehead frost line", [(0.,1.79,1.065),(0.,1.72,1.26),(0.,1.61,1.43)], [.042,.040,.012], rune)


def dragon():
    jade = paint("Verdigris scales", "427469")
    belly = paint("Weathered ochre belly", "c1a171")
    fin = paint("Dark blue mane", "29474c")
    horn = paint("Old gold antlers", "b28b5d", .88)
    plume = paint("Pale bronze whiskers", "bbab82")
    eye = paint("Dragon amber", "c6813e")
    cloud = paint("Pale mountain cloud", "98b5ad")
    path = []
    for i in range(19):
        t = i / 18
        x = .60 * math.sin(t * math.tau * 1.32) * (1.1 - .68*t)
        y = .32 + 1.46*t + .22*math.sin(t*math.tau + .08)
        z = -2.25 + 3.60*t
        radius = .035 + .39*t
        path.append(((x,y,z), radius*1.06, radius))
    loft("Continuous jade spine",path,jade,9)
    loft("Long temple belly",[(Vector(p)-Vector((0.,r*.64,0.)),w*.83,r*.50) for p,w,r in path[3:]],belly,8)
    for i,(p,w,r) in enumerate(path[4:-1:2]):
        x,y,z=p
        blade("Dorsal lacquer", (x,y+r*.60,z+.23), (x+.015,y+r+.21+(i%3)*.068,z-.25),
              .061+i*.005,.094,fin,.030)
        for s in [-1,1]:
            blade("Brushed scale", (x+s*w*.84,y,z+.055), (x+s*w*.95,y-.025,z-.19),
                  .040,.018,fin)
    x,y,z = path[-1][0]
    loft("Long dragon muzzle",[
        ((x,y,z-.23),.20,.19), ((x,y+.16,z+.04),.40,.38),
        ((x,y+.095,z+.37),.382,.255), ((x,y-.036,z+.66),.323,.144),
        ((x,y+.025,z+.83),.297,.145), ((x,y-.011,z+.95),.135,.079)],jade,9)
    loft("Jagged bronze jaw", [((x,y-.143,z+.30),.22,.098), ((x,y-.157,z+.57),.286,.086),
        ((x,y-.093,z+.81),.24,.073), ((x,y-.070,z+.94),.099,.030)], belly, 8)
    keen_eyes((x,y+.226,z+.477),.283,fin,eye,horn)
    for s in [-1,1]:
        oval("Nostril", (x+s*.195,y+.069,z+.86),(.075,.044,.044),fin)
        p=(x+s*.22,y+.34,z+.04)
        thread("Swept horn", [p,(x+s*.30,y+.63,z-.06),(x+s*.46,y+.67,z-.33),
                             (x+s*.54,y+.85,z-.43)], [.10,.087,.052,.009], horn)
        thread("Horn tine", [(x+s*.30,y+.62,z-.09),(x+s*.41,y+.77,z+.04),
                              (x+s*.45,y+.85,z+.09)], [.054,.034,.006], horn)
        thread("Beveled whisker",[(x+s*.20,y-.032,z+.79),(x+s*.48,y+.015,z+.97),
              (x+s*.73,y+.11,z+.74),(x+s*.80,y+.19,z+.41),(x+s*.70,y+.31,z+.10)],
              [.044,.040,.031,.020,.005],plume, 6)
        blade("Jaw spike", (x+s*.24,y-.10,z+.66), (x+s*.32,y-.26,z+.62), .052,.039,horn)
        for j in range(4):
            blade("Silk frill",(x+s*.20,y+.21-j*.135,z+.19),
                  (x+s*(.65-j*.062),y+.35-j*.16,z-.40),.105,.066,fin if j%2 else jade,.10)
        for i in [8,15]:
            (u,v,w),_,_=path[i]
            h=(u+s*.49,v-.25,w+.20)
            thread("Clawed leg",[(u+s*.19,v-.05,w), (u+s*.44,v-.13,w-.22),h],
                  [.13,.09,.076], jade)
            for j in range(3):
                blade("Dragon claw",h,(h[0]+s*(.07*j),h[1]-.09,h[2]+.21-j*.025),.035,.042,belly)
    for j in range(3):
        ox = -.46+j*.28
        loft("Mountain breeze", [((ox,.13,-1.63+j*.12),.019,.012),
            ((ox+.065,.17,-1.50+j*.11),.22,.085),
            ((ox+.066,.16,-1.13+j*.12),.14,.061),
            ((ox+.16,.12,-.96+j*.12),.009,.01)], cloud, 7)


def serpent():
    skin = paint("Papyrus clay", "9f6548")
    dark = paint("Lapis scales", "32615e")
    sand = paint("Sandstone belly plates", "d0ad76")
    outline = paint("Shadow at the crest", "303e38")
    body = []
    for i in range(31):
        t = i / 30
        a = t * math.tau * 1.31 + .86
        r = 1.08 * (1. - .76*t) + .16
        lift = max(0., (t - .61) / .39)
        radius = .033 + .33*t
        body.append(((r*math.cos(a), .16+lift**1.8*1.76, -.38+r*math.sin(a)), radius*1.05,radius))
    x,y,z=body[-1][0]
    body.extend([((x,y+.24,z+.05),.34,.27),((x,y+.43,z+.18),.29,.24)])
    loft("Coil and rising neck",body,dark,10)
    neck = body[22:]
    loft("Banded underside",[(Vector(p)+Vector((0.,0.,r*.59)),w*.69,r*.48) for p,w,r in neck],sand,8)
    for p,w,r in neck[1:-2:2]:
        px,py,pz=p
        blade("Belly scute",(px-w*.58,py,pz+r*.87),(px+w*.64,py+.009,pz+r*.86),.025,.026,skin)
    for side in [-1,1]:
        # The hood grows out of the upper neck and folds back along the spine.
        vertices, faces = [], []
        for j in range(7):
            t=j/6
            spread=.17 + .54*math.sin(math.pi*t)**.87
            for k in range(5):
                u=k/4
                vertices.append((x+side*(.14+u*spread), y-.63+t*1.07,
                                 z-.09-.23*u*u+.09*math.sin(t*math.pi)))
        for j in range(6):
            for k in range(4):
                a = j*5+k
                for order in ((a,a+1,a+5),(a+1,a+6,a+5)):
                    faces.append(order if side>0 else tuple(reversed(order)))
        hood=surface("Sculpted hood",vertices,faces,skin)
        m=hood.modifiers.new("Rounded edge","SOLIDIFY");m.thickness=.07
        for j in range(5):
            t=.18+j*.145
            spread=.17+.54*math.sin(math.pi*t)**.87
            blade("Inlaid hood stripe",(x+side*.31,y-.63+t*1.07,z+.01),
                  (x+side*(.13+spread),y-.63+t*1.07+.08,z-.25),.045,.049,
                  sand if j in (1,3) else dark)
    loft("Flat regal face", [((x,y+.37,z+.06),.22,.21), ((x,y+.40,z+.33),.37,.22),
        ((x,y+.31,z+.61),.30,.155), ((x,y+.26,z+.77),.13,.084)],dark,9)
    blade("Golden cheekline",(x,y+.202,z+.29),(x,y+.227,z+.755),.25,.058,sand)
    blade("Wedge at the crown",(x,y+.58,z+.223),(x,y+.39,z+.69),.245,.077,outline)
    keen_eyes((x,y+.443,z+.537), .237, outline, sand, skin)
    for side in [-1,1]:
        oval("Small nostril",(x+side*.104,y+.313,z+.745),(.041,.030,.038),skin)
    for j,(p,w,r) in enumerate(body[4:22:2]):
        px,py,pz=p
        blade("Coil scales",(px-.065,py+r*.87,pz-.09),(px+.095,py+r*.81,pz+.16),
              .052+j*.003,.028,skin)


def griffin():
    ochre = paint("Lion coat", "a67547")
    under = paint("Dry grass underfur", "c39764")
    ivory = paint("Ivory pinions", "dac39b")
    night = paint("Slate glazed tips", "364a50")
    clay = paint("Oxide red coverts", "a96b50")
    gold = paint("Aged brass beak", "bc8d48", .85)
    eye = paint("Obsidian eye mask", "332f29")
    loft("Lion's back", [((0.,1.12,-1.30),.06,.12),((0.,1.13,-1.02),.33,.46),
        ((0.,1.20,-.66),.44,.53),((0.,1.23,-.08),.46,.54),
        ((0.,1.42,.50),.53,.57),((0.,1.43,.85),.42,.48),((0.,1.44,1.08),.16,.22)],ochre,9)
    loft("Eagle breast and head", [((0.,.92,.68),.26,.23),((0.,1.42,.68),.48,.47),
        ((0.,1.72,.79),.42,.36),((0.,2.02,.95),.36,.33),((0.,2.18,1.13),.255,.22),
        ((0.,2.21,1.31),.15,.13)],ivory,8)
    loft("Chiseled hooked beak", [((0.,2.09,1.19),.215,.147),((0.,2.11,1.46),.23,.171),
        ((0.,2.075,1.68),.16,.129),((0.,1.977,1.785),.088,.087),
        ((0.,1.869,1.71),.016,.019)],gold,8)
    loft("Shadow under the hook", [((0.,1.976,1.25),.13,.052),
        ((0.,1.972,1.57),.149,.067), ((0.,1.95,1.67),.062,.051)],night,7)
    keen_eyes((0.,2.237,1.316),.195,eye,gold,ivory)
    blade("Brow crest",(0.,2.38,1.047),(0.,2.28,1.438), .207,.027,under)
    thread("Lion tail",[(0.,1.24,-1.17),(-.18,1.09,-1.48),(-.40,1.12,-1.88),
       (-.54,1.32,-2.04),(-.54,1.57,-2.02),(-.35,1.71,-1.86)], [.135,.105,.081,.066,.052,.025],ochre,10)
    blade("Tail tuft",(-.43,1.65,-1.97),(-.20,1.83,-1.74),.116,.122,clay,.061)
    for s in [-1,1]:
        quadruped_leg(s,-.78,1.19,ochre,under)
        quadruped_leg(s,.66,1.31,ivory,gold)
        for j in range(3):
            blade("Talon",(s*.40+(j-1)*.065,.098,.81),(s*.40+(j-1)*.072,.04,1.079),
                  .033,.036,night,.042)
        for j in range(5):
            feather("Raked eagle cheek",(s*.21,2.19-j*.15,1.03),
                    (s*(.42+j*.031),2.18-j*.148,.54-j*.031), .102,
                    ivory if j%3 else under, under if j%3 else clay)
        for j in range(3):
            blade("Crest feather",(s*.10,2.22,.98-j*.085),(s*(.18+j*.063),2.79-j*.11,.49-j*.078),
                  .086,.068,night if j==2 else ivory,.066)
        branch = [(s*.36,1.69,.48),(s*.72,2.09,.34),(s*1.12,2.32,.01),
                  (s*1.53,2.39,-.38),(s*2.01,2.31,-.70)]
        loft("Covered wing shoulder",[(p,w,w*.8) for p,w in zip(branch,[.24,.29,.25,.19,.048])],
             under,12)
        for j in range(9):
            t = j/8
            start = Vector((s*(.55+1.30*t),1.92+.45*t,.42-.99*t))
            end = Vector((s*(.92+1.69*t),1.77+1.01*t,-1.18-.62*t+.57*t*t))
            feather("Long folded pinion", start, end, .175,
                    night if j in [0,2,6,8] else ivory, under if j in [0,2,6,8] else clay)
            loftline = Vector((0.,.13,.04))
            feather("Red secondary covert", start.lerp(end,.09)+loftline,
                    start.lerp(end,.58)+loftline, .121, clay, under)
            feather("Ivory root covert", start+Vector((0.,.14,.12)),
                    start+Vector((s*.24,.10,-.54)), .125, ivory, under)


def bake(name, model, export=True):
    reset()
    model()
    # Join by palette to keep draw calls small on mobile. Apply the hood thickness before joining.
    for mat in bpy.data.materials:
        pieces = [obj for obj in bpy.context.collection.objects if obj.type == "MESH" and obj.data.materials[0] == mat]
        if not pieces:
            continue
        for obj in pieces:
            bpy.context.view_layer.objects.active = obj
            for modifier in list(obj.modifiers):
                bpy.ops.object.modifier_apply(modifier=modifier.name)
        bpy.ops.object.select_all(action="DESELECT")
        for obj in pieces:
            obj.select_set(True)
        bpy.context.view_layer.objects.active = pieces[0]
        if len(pieces) > 1:
            bpy.ops.object.join()
        bpy.context.object.name = mat.name
    if export:
        bpy.ops.export_scene.gltf(filepath=str(MODELS / (name + ".glb")), export_format="GLB",
                                  export_cameras=False, export_lights=False,
                                  export_draco_mesh_compression_enable=False)
    print(name, "faces", sum(len(o.data.polygons) for o in bpy.context.collection.objects if o.type=="MESH"))


if __name__ == "__main__":
    MODELS.mkdir(parents=True,exist_ok=True)
    for key, model in [("valley",deer),("fjord",wolf),("terraces",dragon),("lake",serpent),("aegean",griffin)]:
        bake(key,model)
