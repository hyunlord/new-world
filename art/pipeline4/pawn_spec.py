# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow", "numpy"]
# ///
# How to run: python3 -m art.pipeline4.pawn_gen OUTPUT (tests: python3 -m pytest art/pipeline4)
"""Declarative pawn shape specification; all coordinates are game pixels."""
CANVAS = (64, 96)
SCALE = 4
DIRECTIONS = ('S', 'E', 'N')
STATES = ('idle', 'walk_a', 'walk_b', 'work_a', 'work_b', 'carry', 'sleep')
HAIRS = ('none', 'short', 'long_tied', 'bun', 'braid', 'grey_thin')
TOPS = ('none', 'hide_wrap', 'fur_cloak', 'woven_tunic')
ITEMS = ('none', 'stone_axe', 'log_bundle', 'basket')
GRIPS = {'S': (46,64), 'E': (44,62), 'N': (18,64)}
MOTION = {'idle':(0,0,0), 'walk_a':(0,-1,-4), 'walk_b':(0,1,4), 'work_a':(2,0,-30), 'work_b':(2,0,40), 'carry':(0,0,0), 'sleep':(0,0,0)}
PALETTE_INDICES = {'skin': (21,14,4,6), 'hair': (20,3,13,16), 'cloth': (12,4), 'secondary': (17,11), 'wood': (6,), 'stone': (23,)}
BODY_WIDTH = {'adult': {'M':26,'F':24}, 'child': {'M':18,'F':18}, 'elder': {'M':24,'F':24}}
BODY = [('capsule',(19,54,45,88),'skin'), ('polygon',((22,84),(27,84),(27,88),(19,88)),'skin'), ('polygon',((37,84),(42,84),(45,88),(37,88)),'skin'), ('polygon',((28,48),(36,48),(36,58),(28,58)),'skin'), ('ellipse',(21,32,43,52),'skin')]
HEAD_CHILD = (23,34,41,50)
FACE = {'S':[('ellipse',(26,41,28,43),'ink'),('ellipse',(36,41,38,43),'ink'),('polyline',((30,47),(34,47)),'ink')], 'E':[('ellipse',(37,41,39,43),'ink')], 'N':[]}
ELDER_FACE = [('polyline',((25,45),(28,45)),'ink'),('polyline',((36,45),(39,45)),'ink')]
HAIR = {
'none': [],
'short': [('polygon',((23,39),(24,35),(29,33),(36,33),(40,35),(41,40),(36,37),(31,39),(27,37)),'hair')],
'long_tied': [('polygon',((22,41),(24,35),(29,33),(36,33),(40,36),(42,46),(39,49),(37,40),(32,36),(27,39),(26,49),(23,46)),'hair')],
'bun': [('ellipse',(27,33,37,40),'hair'),('polygon',((23,39),(25,36),(39,36),(41,41),(37,40),(32,38),(27,41)),'hair')],
'braid': [('polygon',((23,39),(24,35),(30,33),(36,33),(40,36),(40,41),(36,38),(29,38),(26,41)),'hair'),('ellipse',(23,39,28,45),'hair'),('ellipse',(24,43,29,49),'hair')],
'grey_thin': [('polygon',((23,40),(24,36),(27,35),(26,41),(25,45),(23,43)),'hair'),('polygon',((37,35),(40,37),(41,43),(39,45),(38,40)),'hair'),('polyline',((28,34),(32,34),(35,35)),'hair')]
}
HAIR_STROKES = {'none':[], 'short':[((27,35),(29,37)),((33,35),(35,37))], 'long_tied':[((24,39),(25,45)),((39,38),(40,45))], 'bun':[((30,35),(33,37))], 'braid':[((25,41),(27,43)),((26,45),(28,47))], 'grey_thin':[]}
TOP = {
'none':[],
'hide_wrap':[('polygon',((29,56),(35,56),(41,62),(41,74),(24,77),(23,64)),'cloth'),('polygon',((27,59),(29,57),(39,72),(37,75)),'secondary')],
'fur_cloak':[('polygon',((28,56),(36,56),(40,60),(42,67),(40,74),(37,72),(34,76),(31,73),(27,76),(24,72),(22,67),(24,60)),'cloth'),('polygon',((27,58),(31,61),(35,61),(38,58),(36,65),(29,65)),'secondary')],
'woven_tunic':[('polygon',((28,56),(36,56),(40,61),(41,75),(23,75),(24,61)),'cloth'),('polygon',((24,69),(40,69),(40,72),(24,72)),'secondary')]
}
TOP_STROKES = {'none':[], 'hide_wrap':[((26,66),(28,71)),((34,61),(37,66))], 'fur_cloak':[((25,63),(27,67)),((39,63),(37,67)),((30,69),(32,72))], 'woven_tunic':[((28,62),(28,67)),((36,62),(36,67))]}
# Item coordinates are local to the grip. Negative x reaches into the body.
ITEM = {'none':[],
'stone_axe':[('capsule',(-5,-9,0,11),'wood'),('polygon',((-6,-9),(3,-14),(10,-11),(10,-4),(3,-3),(-5,-6)),'stone')],
'log_bundle':[('capsule',(-24,-8,8,0),'wood'),('capsule',(-22,-2,10,6),'wood'),('ellipse',(4,-8,11,0),'secondary'),('ellipse',(6,-2,13,6),'secondary'),('polyline',((-8,-7),(-8,5)),'ink'),('polyline',((-2,-7),(-2,5)),'ink')],
'basket':[('ellipse',(-8,-14,6,1),'wood'),('polygon',((-12,-6),(11,-6),(8,12),(-9,12)),'secondary'),('polyline',((-10,0),(9,0)),'wood'),('polyline',((-9,5),(8,5)),'wood'),('polyline',((-4,-5),(-4,10)),'wood'),('polyline',((3,-5),(3,10)),'wood')]}
AGE_OVERLAY = {'adult':(1.0,1.0), 'child':(18/22,16/20), 'elder':(1.0,1.0)}

TOP_ENVELOPE = [('capsule',(19,54,45,88),'cloth'),('polygon',((28,48),(36,48),(36,58),(28,58)),'cloth')]
TOP_START = 48
TOP_CROP = (0,0,64,81)

HAIR_BACK = [('ellipse',(23,34,41,49),'hair')]

CARRY_GRIPS = {'S':(42,56),'E':(42,56),'N':(24,56)}
