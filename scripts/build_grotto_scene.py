"""
Masterwork Procedural 3D Cavern Grotto Generator - Concept Artwork Fidelity Edition
Executes inside Blender 4.5 background mode.
Generates assets/models/cavern_kit.glb adhering strictly to the Art Reference Directives:

1. Estética e Iluminação Ambiental (Cel-shaded ink lines, soft skylight shaft, cyan bioluminescence)
2. Teto e Arco da Caverna (Teto rebaixado e estalactites pontiagudas visíveis emoldurando a cena)
3. Primeiro Plano (Largo caminho central de terra batida marrom sem grama, seixos soltos, laterais gramadas com pedras angulares)
4. Flora Bioluminescente:
   - Lado esquerdo: Cogumelos altos inclinados com chapéu azul-ciano e lamelas visíveis, plantas de folhas largas e hastes com pontas brilhantes.
   - Lado direito: Três cogumelos de chapéu arredondado com emissão intensa azul-turquesa/ciano, grama alta e plantas com bulbos esféricos luminosos.
5. Plano Médio (Lagoa circular com fina borda curva de pedra nivelada com a água azul-turquesa, nuvens volumétricas de espuma de impacto e anéis de ondulação)
6. Plano de Fundo (Paredes de blocos de rocha facetados em anfiteatro com musgo superior e vinhas penduradas).

COORDINATE MAPPING (Blender to Bevy glTF):
Bevy camera is at (0, 1.75, 0) looking towards -Z.
Blender +Y exports to glTF -Z (Forward in front of camera!).
Blender -Y exports to glTF +Z (Behind camera).
Blender Z exports to Bevy Y (Up).
"""
import bpy
import bmesh
import math
import random
import os
from mathutils import Vector, Matrix, Euler

# Reset entire Blender scene
for obj in list(bpy.data.objects):
    bpy.data.objects.remove(obj, do_unlink=True)
for mesh in list(bpy.data.meshes):
    bpy.data.meshes.remove(mesh)
for mat in list(bpy.data.materials):
    bpy.data.materials.remove(mat)

def create_pbr_material(name, base_color, roughness=0.8, metallic=0.0, emission=None, emission_strength=1.0, alpha=1.0):
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    bsdf = next((n for n in nodes if n.type == 'BSDF_PRINCIPLED'), None)
    if bsdf:
        if "Base Color" in bsdf.inputs:
            bsdf.inputs["Base Color"].default_value = (*base_color[:3], 1.0)
        if "Roughness" in bsdf.inputs:
            bsdf.inputs["Roughness"].default_value = roughness
        if "Metallic" in bsdf.inputs:
            bsdf.inputs["Metallic"].default_value = metallic
        if "Alpha" in bsdf.inputs:
            bsdf.inputs["Alpha"].default_value = alpha
        if alpha < 1.0:
            mat.blend_method = 'BLEND'
        if emission:
            if "Emission Color" in bsdf.inputs:
                bsdf.inputs["Emission Color"].default_value = (*emission[:3], 1.0)
            elif "Emission" in bsdf.inputs:
                bsdf.inputs["Emission"].default_value = (*emission[:3], 1.0)
            if "Emission Strength" in bsdf.inputs:
                bsdf.inputs["Emission Strength"].default_value = emission_strength
    return mat

# --- Color Palette strictly matching Concept Art ---
mat_rock_shadow = create_pbr_material("M_Rock_Shadow", (0.11, 0.15, 0.19), roughness=0.90)
mat_rock_lit = create_pbr_material("M_Rock_Lit", (0.22, 0.30, 0.35), roughness=0.80)
mat_cavern_moss = create_pbr_material("M_Cavern_Moss", (0.10, 0.46, 0.16), roughness=0.86) # Velvet emerald moss #246e34
mat_cavern_vine = create_pbr_material("M_Cavern_Vine", (0.16, 0.50, 0.22), roughness=0.75)
mat_ground_path = create_pbr_material("M_Ground_Path", (0.15, 0.08, 0.03), roughness=0.95) # Deep rich chocolate earth #3d1f0c
mat_ground_grass = create_pbr_material("M_Ground_Grass", (0.12, 0.42, 0.16), roughness=0.85)
mat_fine_rim = create_pbr_material("M_Fine_Stone_Rim", (0.22, 0.24, 0.26), roughness=0.75)
mat_river_pebble = create_pbr_material("M_River_Pebble", (0.32, 0.34, 0.36), roughness=0.65)
mat_angular_rock = create_pbr_material("M_Angular_Rock", (0.20, 0.23, 0.26), roughness=0.88)

# Flora: Left Side (Vibrant Cyan Parasol Mushrooms, Gills, Broad Leaves, Glowing Stalks)
mat_shroom_pale_cap = create_pbr_material("M_Shroom_Pale_Cap", (0.05, 0.88, 0.86), roughness=0.20, emission=(0.06, 0.95, 0.92), emission_strength=2.6)
mat_shroom_dots = create_pbr_material("M_Shroom_Dots", (0.85, 0.98, 1.0), roughness=0.20, emission=(0.80, 0.98, 1.0), emission_strength=2.2)
mat_shroom_gills = create_pbr_material("M_Shroom_Gills", (0.05, 0.72, 0.78), roughness=0.30, emission=(0.08, 0.84, 0.88), emission_strength=2.2)
mat_shroom_stem = create_pbr_material("M_Shroom_Stem", (0.72, 0.84, 0.80), roughness=0.65, emission=(0.06, 0.22, 0.22), emission_strength=0.6)
mat_broad_leaf = create_pbr_material("M_Broad_Leaf", (0.14, 0.48, 0.20), roughness=0.65)
mat_glowing_tip = create_pbr_material("M_Glowing_Tip", (0.20, 0.95, 0.92), roughness=0.20, emission=(0.20, 0.98, 0.95), emission_strength=3.0)

# Flora: Right Side (Intense Cyan/Turquoise Mushrooms, Tall Grass, Glowing Bulbs)
mat_shroom_vibrant_cap = create_pbr_material("M_Shroom_Vibrant_Cap", (0.04, 0.88, 0.90), roughness=0.18, emission=(0.06, 0.96, 0.94), emission_strength=2.6)
mat_tall_grass = create_pbr_material("M_Tall_Grass", (0.16, 0.50, 0.22), roughness=0.72)
mat_glow_bulb = create_pbr_material("M_Glow_Bulb", (0.15, 0.95, 0.90), roughness=0.15, emission=(0.15, 0.98, 0.92), emission_strength=3.0)

# Waterfall Foam Billows & Water Ripples
mat_water_foam_billow = create_pbr_material("M_Water_Foam_Billow", (0.92, 0.98, 1.0), roughness=0.20, emission=(0.60, 0.90, 1.0), emission_strength=1.5)
mat_water_ripple_ring = create_pbr_material("M_Water_Ripple_Ring", (0.40, 0.92, 0.98), roughness=0.15, emission=(0.35, 0.90, 0.98), emission_strength=1.8, alpha=0.85)

# Ceiling Lichen
mat_ceiling_lichen = create_pbr_material("M_Ceiling_Lichen", (0.20, 0.92, 0.88), roughness=0.15, emission=(0.18, 0.95, 0.90), emission_strength=2.8)

# ==============================================================================
# EXACT MATHEMATICAL TERRAIN SURFACE HEIGHT FUNCTION
# ==============================================================================
def get_terrain_surface_z(x, y):
    lake_center_x = 0.0
    lake_center_y = 8.5
    lake_radius = 5.2
    dist_lake = math.sqrt((x - lake_center_x)**2 + (y - lake_center_y)**2)
    if dist_lake < lake_radius:
        depth_factor = 1.0 - (dist_lake / lake_radius)
        return -0.75 * depth_factor
    elif dist_lake < lake_radius + 2.0:
        return 0.05
    elif y < 4.2 and abs(x) < 2.6:
        return 0.04 # Flat beaten brown dirt path
    else:
        dist_side = max(0.0, abs(x) - 3.8)
        bank_h = min(2.0, dist_side * 0.38)
        return 0.05 + bank_h + math.sin(x * 0.3) * 0.08

# ==============================================================================
# 1. VISIBLE VAULTED CAVERN ARCH & CEILING (EMOLDURANDO A CENA NO TOPO DA TELA)
# ==============================================================================
def build_visible_cavern_roof_and_arch():
    """
    Creates the dark natural cavern arch framing the upper camera view,
    dipping down to Z=4.2..6.5 so it is prominently visible in the upper frame,
    leaving the central waterfall shaft open to the sky/fissure.
    """
    bm = bmesh.new()
    segments_u = 40
    segments_v = 24
    
    # Outer radius around the grotto
    center_y = 6.0
    
    grid = []
    for iv in range(segments_v + 1):
        v_frac = iv / segments_v
        # Y spans from behind the camera (Y=-2.0) to behind the waterfall (Y=12.0)
        curr_y = -2.0 + v_frac * 14.0
        
        row = []
        for iu in range(segments_u + 1):
            u_frac = iu / segments_u
            # Angle from left wall to right wall
            angle = -math.pi * 0.45 + u_frac * math.pi * 0.90
            
            # Distance from center line
            wall_dist = 6.2 + abs(curr_y - 6.0) * 0.32
            
            # Base arch height profile:
            # Over the waterfall (curr_y > 7.0 and abs(angle) < 0.25), arch rises for waterfall opening
            is_waterfall_cleft = (curr_y > 7.0 and curr_y < 10.5 and abs(math.sin(angle)) < 0.28)
            
            if is_waterfall_cleft:
                base_z = 7.5 + (1.0 - abs(math.sin(angle)) / 0.28) * 4.5
            else:
                # Arched cave roof dipping down to frame the scene
                arch_dip = math.cos(angle * 1.3) * 2.2
                base_z = 3.6 + arch_dip + (curr_y * 0.18)
                
            # Rocky noise
            noise = math.sin(angle * 6.0 + curr_y * 0.8) * 0.35 + math.cos(angle * 12.0) * 0.15
            curr_z = base_z + noise
            curr_x = math.sin(angle) * wall_dist
            
            v = bm.verts.new((curr_x, curr_y, curr_z))
            row.append(v)
        grid.append(row)
        
    bm.verts.ensure_lookup_table()
    for iv in range(segments_v):
        for iu in range(segments_u):
            v1 = grid[iv][iu]
            v2 = grid[iv][iu + 1]
            v3 = grid[iv + 1][iu + 1]
            v4 = grid[iv + 1][iu]
            f = bm.faces.new((v1, v2, v3, v4))
            
            center = f.calc_center_median()
            # Upper faces get rock shadow, fissure gets rock lit
            if center.z > 7.0:
                f.material_index = 1 # Lit rock
            else:
                f.material_index = 0 # Dark shadow rock
                
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new("Cavern_Visible_Arch_Mesh")
    obj = bpy.data.objects.new("Cavern_Visible_Arch", mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    
    for p in mesh.polygons:
        p.use_smooth = False # Faceted rocky appearance
    mesh.update()
    
    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)
    return obj

build_visible_cavern_roof_and_arch()

# ==============================================================================
# 2. OVERHEAD SHARP POINTED STALACTITES VISIBLE IN CAMERA FRAME (Teto e Estalactites)
# ==============================================================================
def spawn_organic_stalactite(x, y, z_top, height=2.4, radius=0.45, tilt=(0.0, 0.0), name="OrganicStalactite"):
    """
    Spawns sharp, organic mineral stalactites hanging into the camera field of view.
    """
    bm = bmesh.new()
    rings = 10
    radials = 12
    
    ring_verts = []
    for ir in range(rings + 1):
        t = ir / rings
        # Root flaring into ceiling rock
        root_flare = ((1.0 - t) ** 3.0) * 0.35
        # Mineral rings/ridges
        mineral_bumps = math.sin(t * math.pi * 4.0) * 0.04 * radius
        curr_r = max(0.015, radius * (1.0 - t * 0.95) + root_flare + mineral_bumps)
        
        curr_x = x + tilt[0] * (t ** 1.2) + math.sin(t * 5.0) * 0.02
        curr_y = y + tilt[1] * (t ** 1.2) + math.cos(t * 5.0) * 0.02
        curr_z = z_top - t * height
        
        row = []
        for ia in range(radials):
            angle = 2.0 * math.pi * ia / radials
            r_wobble = curr_r * (1.0 + math.sin(angle * 3.0 + t * 4.0) * 0.08)
            vx = curr_x + math.cos(angle) * r_wobble
            vy = curr_y + math.sin(angle) * r_wobble
            vz = curr_z
            vert = bm.verts.new((vx, vy, vz))
            row.append(vert)
        ring_verts.append(row)
        
    bm.verts.ensure_lookup_table()
    
    for ir in range(rings):
        t = (ir + 0.5) / rings
        if t < 0.30:
            mat_idx = 0  # Dark shadow rock merging seamlessly with cavern arch
        elif t < 0.75:
            mat_idx = 1  # Faceted mineral rock
        else:
            mat_idx = 0  # Wet dark stone tip
            
        for ia in range(radials):
            ia_next = (ia + 1) % radials
            v1 = ring_verts[ir][ia]
            v2 = ring_verts[ir][ia_next]
            v3 = ring_verts[ir + 1][ia_next]
            v4 = ring_verts[ir + 1][ia]
            face = bm.faces.new((v1, v2, v3, v4))
            face.material_index = mat_idx
            
    tip_center = bm.verts.new((x + tilt[0] + 0.005, y + tilt[1], z_top - height))
    for ia in range(radials):
        ia_next = (ia + 1) % radials
        face = bm.faces.new((ring_verts[rings][ia], ring_verts[rings][ia_next], tip_center))
        face.material_index = 0
        
    top_center = bm.verts.new((x, y, z_top))
    for ia in range(radials):
        ia_next = (ia + 1) % radials
        face = bm.faces.new((ring_verts[0][ia_next], ring_verts[0][ia], top_center))
        face.material_index = 0
        
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    
    for poly in mesh.polygons:
        poly.use_smooth = False # Crisp faceted minerals!
    mesh.update()
    
    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)
    return obj

# PROMINENT LATERAL STALACTITES (EIXO CENTRAL DA CACHOEIRA 100% LIVRE)
# All stalactites kept at |X| >= 3.4m, leaving waterfall axis open to the sky/fissure
visible_stalactites = [
    # Top-Left framing stalactites (descending naturally from the rocky archway)
    (-5.2, 3.8, 4.9, 1.75, 0.38, (0.05, -0.04)),
    (-4.2, 3.4, 4.6, 1.45, 0.34, (0.04, 0.03)),
    (-4.8, 5.2, 5.3, 1.95, 0.42, (0.06, -0.05)),
    (-3.6, 4.8, 5.1, 1.55, 0.35, (-0.03, 0.04)),
    (-3.4, 6.2, 5.8, 1.80, 0.38, (-0.04, 0.03)),

    # Top-Right framing stalactites (descending naturally from the rocky archway)
    (5.2, 3.8, 4.9, 1.75, 0.38, (-0.05, -0.04)),
    (4.2, 3.4, 4.6, 1.45, 0.34, (-0.04, 0.03)),
    (4.8, 5.2, 5.3, 1.95, 0.42, (-0.06, -0.05)),
    (3.6, 4.8, 5.1, 1.55, 0.35, (0.03, 0.04)),
    (3.4, 6.2, 5.8, 1.80, 0.38, (0.04, 0.03)),
]

for idx, (sx, sy, sz, sh, sr, stilt) in enumerate(visible_stalactites):
    spawn_organic_stalactite(sx, sy, sz, height=sh, radius=sr, tilt=stilt, name=f"VisibleStalactite_{idx}")

# ==============================================================================
# 3. TIERED FACETED ROCK AMPHITHEATER FLANKING WATERFALL (Paredes de Rocha e Musgo)
# ==============================================================================
def build_faceted_rock_slab(name, center_x, center_y, z_base, z_top, width_x, length_y, rot_z=0.0):
    """
    Creates an angular faceted rock slab with flat shading and top moss.
    """
    bm = bmesh.new()
    num_corners = 7
    bot_verts = []
    top_verts = []
    
    rng = random.Random(int(abs(center_x * 100 + center_y * 10)))
    
    for ic in range(num_corners):
        ang = (ic / num_corners) * math.pi * 2.0 + rot_z
        jitter_r = 0.82 + rng.uniform(-0.12, 0.12)
        vx = center_x + math.cos(ang) * (width_x * 0.5) * jitter_r
        vy = center_y + math.sin(ang) * (length_y * 0.5) * jitter_r
        
        bot_verts.append(bm.verts.new((vx, vy, z_base)))
        # Top bevel
        top_verts.append(bm.verts.new((vx * 0.94 + center_x * 0.06, vy * 0.94 + center_y * 0.06, z_top)))
        
    bm.verts.ensure_lookup_table()
    
    # Side faces (Faceted dark rock)
    for ic in range(num_corners):
        ic_next = (ic + 1) % num_corners
        face = bm.faces.new((bot_verts[ic], bot_verts[ic_next], top_verts[ic_next], top_verts[ic]))
        face.material_index = 0 # M_Rock_Shadow
        
    # Top face (Moss covered ledge!)
    top_face = bm.faces.new(top_verts)
    top_face.material_index = 2 # M_Cavern_Moss
    
    # Bottom face
    bm.faces.new(list(reversed(bot_verts)))
    
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    
    for p in mesh.polygons:
        p.use_smooth = False # Crisp angular facets!
    mesh.update()
    
    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)
    return obj

# Stacked amphitheater tiers hugging the waterfall
rock_tiers = [
    # Left tiered shelves stepping up towards the waterfall
    ("Left_Tier_1", -4.2, 6.8, 0.05, 1.10, 2.8, 2.4, 0.2),
    ("Left_Tier_2", -4.8, 7.8, 0.95, 2.25, 2.6, 2.2, -0.3),
    ("Left_Tier_3", -5.2, 8.8, 2.10, 3.45, 2.4, 2.0, 0.4),
    ("Left_Tier_4", -5.5, 9.8, 3.30, 4.65, 2.2, 1.8, -0.2),
    ("Left_Tier_Low", -3.6, 5.8, 0.05, 0.75, 2.0, 1.8, 0.5),
    
    # Right tiered shelves stepping up towards the waterfall
    ("Right_Tier_1", 4.2, 6.8, 0.05, 1.10, 2.8, 2.4, -0.2),
    ("Right_Tier_2", 4.8, 7.8, 0.95, 2.25, 2.6, 2.2, 0.3),
    ("Right_Tier_3", 5.2, 8.8, 2.10, 3.45, 2.4, 2.0, -0.4),
    ("Right_Tier_4", 5.5, 9.8, 3.30, 4.65, 2.2, 1.8, 0.2),
    ("Right_Tier_Low", 3.6, 5.8, 0.05, 0.75, 2.0, 1.8, -0.5),
    
    # Back rock cliff behind the waterfall
    ("Back_Cliff_Left", -2.8, 10.2, 0.05, 6.5, 3.2, 2.4, 0.1),
    ("Back_Cliff_Right", 2.8, 10.2, 0.05, 6.5, 3.2, 2.4, -0.1),
    ("Back_Cliff_Center", 0.0, 11.2, 0.05, 8.0, 4.5, 2.2, 0.0),
]

for (tname, tx, ty, tzb, tzt, tw, tl, trot) in rock_tiers:
    build_faceted_rock_slab(tname, tx, ty, tzb, tzt, tw, tl, trot)

# Hanging Vines draping down the rock fissure ledges
def spawn_hanging_vine(x, y, z_top, length=1.8, segments=8, name="HangingVine"):
    bm = bmesh.new()
    width = 0.06
    rows = []
    for iseg in range(segments + 1):
        t = iseg / segments
        cz = z_top - t * length
        cx = x + math.sin(t * math.pi * 3.0) * 0.07
        cy = y + (t * 0.03)
        w = width * (1.0 - t * 0.40)
        v_l = bm.verts.new((cx - w * 0.5, cy, cz))
        v_r = bm.verts.new((cx + w * 0.5, cy, cz))
        rows.append((v_l, v_r))
        
        # Leaf buds sprouting sideways
        if iseg % 2 == 1:
            side = 1.0 if (iseg % 4 == 1) else -1.0
            leaf_tip = bm.verts.new((cx + side * (w + 0.14), cy - 0.02, cz - 0.03))
            bm.faces.new((v_l, v_r, leaf_tip))
            
    for iseg in range(segments):
        bm.faces.new((rows[iseg][0], rows[iseg][1], rows[iseg + 1][1], rows[iseg + 1][0]))
        
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_cavern_vine)

vine_locations = [
    (-3.8, 6.9, 1.1, 0.9),
    (-4.5, 7.9, 2.2, 1.2),
    (-4.9, 8.9, 3.4, 1.4),
    (3.8, 6.9, 1.1, 0.9),
    (4.5, 7.9, 2.2, 1.2),
    (4.9, 8.9, 3.4, 1.4),
]
for idx, (vx, vy, vz, vl) in enumerate(vine_locations):
    spawn_hanging_vine(vx, vy, vz, length=vl, name=f"Rock_Vine_{idx}")

# ==============================================================================
# 4. TERRAIN (WIDE GRASSLESS BEATEN BROWN PATH, GRASS BANKS, AND FINE STONE RIM)
# ==============================================================================
def build_terrain_and_lake_rim():
    bpy.ops.mesh.primitive_grid_add(x_subdivisions=52, y_subdivisions=52, size=34.0, location=(0.0, 0.0, 0.0))
    terrain = bpy.context.active_object
    terrain.name = "Cavern_Terrain_Grid"
    terrain.data.materials.append(mat_ground_path)  # Material 0: Beaten brown earth #472914
    terrain.data.materials.append(mat_ground_grass) # Material 1: Dense green grass

    bm = bmesh.new()
    bm.from_mesh(terrain.data)
    
    lake_center_x = 0.0
    lake_center_y = 8.5
    lake_radius = 5.2
    
    for v in bm.verts:
        v.co.z = get_terrain_surface_z(v.co.x, v.co.y)

    for f in bm.faces:
        center = f.calc_center_median()
        dist_l = math.sqrt((center.x - lake_center_x)**2 + (center.y - lake_center_y)**2)
        # Wide grassless path in lower center (X between -2.6 and +2.6, Y from 0 to 4.5)
        is_path_strip = (center.y < 4.8 and abs(center.x) < 2.55)
        is_lake_shore = dist_l < (lake_radius + 1.2)
        
        if is_path_strip or is_lake_shore:
            f.material_index = 0 # Pure beaten brown dirt #472914 (TOTALMENTE LIVRE DE GRAMA)
        else:
            f.material_index = 1 # Lateral dense green grass banks

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(terrain.data)
    bm.free()
    for p in terrain.data.polygons:
        p.use_smooth = True
    terrain.data.update()

    # Fine curved stone rim leveled with the water
    bm_rim = bmesh.new()
    rim_pts = 64
    rim_grid = []
    rim_top_z = 0.08
    rim_bot_z = -0.06
    
    for ip in range(rim_pts + 1):
        frac = ip / rim_pts
        ang = math.pi * 0.85 + frac * math.pi * 1.30
        
        r_inner = lake_radius - 0.08
        r_outer = lake_radius + 0.16
        
        xi = lake_center_x + math.cos(ang) * r_inner
        yi = lake_center_y + math.sin(ang) * r_inner
        
        xo = lake_center_x + math.cos(ang) * r_outer
        yo = lake_center_y + math.sin(ang) * r_outer
        
        v_in_top = bm_rim.verts.new((xi, yi, rim_top_z))
        v_out_top = bm_rim.verts.new((xo, yo, rim_top_z))
        v_out_bot = bm_rim.verts.new((xo, yo, rim_bot_z))
        v_in_bot = bm_rim.verts.new((xi, yi, rim_bot_z))
        rim_grid.append((v_in_top, v_out_top, v_out_bot, v_in_bot))
        
    for ip in range(rim_pts):
        r1 = rim_grid[ip]
        r2 = rim_grid[ip + 1]
        bm_rim.faces.new((r1[0], r2[0], r2[1], r1[1]))
        bm_rim.faces.new((r1[1], r2[1], r2[2], r1[2]))
        
    bmesh.ops.recalc_face_normals(bm_rim, faces=bm_rim.faces)
    rim_mesh = bpy.data.meshes.new("Lake_Fine_Rim_Mesh")
    rim_obj = bpy.data.objects.new("Lake_Fine_Rim", rim_mesh)
    bpy.context.collection.objects.link(rim_obj)
    bm_rim.to_mesh(rim_mesh)
    bm_rim.free()
    for p in rim_mesh.polygons:
        p.use_smooth = False
    rim_mesh.update()
    rim_obj.data.materials.append(mat_fine_rim)

build_terrain_and_lake_rim()

# Loose river pebbles on dirt path edges
def spawn_river_pebble(x, y, z, sx, sy):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=8, ring_count=6, radius=0.10, location=(x, y, z))
    peb = bpy.context.active_object
    peb.scale = (sx, sy, 0.45)
    bpy.ops.object.transform_apply(scale=True)
    peb.data.materials.append(mat_river_pebble)
    for p in peb.data.polygons:
        p.use_smooth = True

pebble_coords = [
    (-2.3, 0.8, 0.06, 0.28, 0.22),
    (-2.4, 1.8, 0.07, 0.35, 0.26),
    (-2.2, 2.7, 0.08, 0.26, 0.20),
    (-2.5, 3.6, 0.09, 0.32, 0.25),
    (2.3, 0.8, 0.06, 0.25, 0.20),
    (2.4, 1.8, 0.08, 0.32, 0.24),
    (2.5, 2.9, 0.09, 0.28, 0.22),
    (2.7, 3.8, 0.10, 0.35, 0.28),
]
for (px, py, pz, psx, psy) in pebble_coords:
    spawn_river_pebble(px, py, pz, psx, psy)

# Angular Transition Rocks on the boundary between Dirt and Grass
def spawn_angular_rock(x, y, z, sx=0.55, sy=0.45, sz=0.35, rot_z=0.0):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(x, y, z))
    rock = bpy.context.active_object
    rock.scale = (sx, sy, sz)
    rock.rotation_euler = (0.2, 0.15, rot_z)
    bpy.ops.object.transform_apply(scale=True, rotation=True)
    rock.data.materials.append(mat_angular_rock)
    for p in rock.data.polygons:
        p.use_smooth = False # Crisp angular facets!

angular_rock_coords = [
    (-2.6, 1.2, 0.12, 0.58, 0.44, 0.32, 0.4),
    (-2.7, 2.4, 0.14, 0.62, 0.48, 0.36, -0.6),
    (-2.9, 3.5, 0.16, 0.70, 0.52, 0.42, 0.8),
    (2.6, 1.2, 0.12, 0.58, 0.44, 0.32, -0.4),
    (2.7, 2.4, 0.14, 0.62, 0.48, 0.36, 0.6),
    (2.9, 3.5, 0.16, 0.70, 0.52, 0.42, -0.8),
]
for (ax, ay, az, asx, asy, asz, arot) in angular_rock_coords:
    spawn_angular_rock(ax, ay, az, asx, asy, asz, arot)

# ==============================================================================
# 5. LEFT SIDE FLORA: MASTER ARTWORK LEANING MUSHROOMS WITH PALE BLUE-CYAN CAPS & GILLS
# ==============================================================================
def build_stylized_parasol_mushroom(root_x, root_y, scale=1.0, lean_angle=-0.22, name="ParasolMushroom"):
    """
    Creates an arched umbrella parasol mushroom with curved downwards rim,
    luminous cyan gills, glowing spore spots, and slender leaning stem.
    """
    ground_z = get_terrain_surface_z(root_x, root_y)
    root_z = ground_z - 0.15 * scale
    
    # 1. Stem
    bm_stem = bmesh.new()
    stem_segments = 16
    radial_segments = 16
    h_stem = 2.4 * scale
    r_base = 0.22 * scale
    
    stem_rows = []
    for ih in range(stem_segments + 1):
        t = ih / stem_segments
        curr_z = root_z + t * h_stem
        # Lean towards center pool
        curr_x = root_x + (t ** 1.3) * math.sin(lean_angle) * h_stem
        curr_y = root_y + (t ** 1.3) * 0.25 * scale
        
        root_flare = (((0.25 - t) / 0.25) ** 2.0) * 0.30 * scale if t < 0.25 else 0.0
        r_curr = r_base * (1.0 - t * 0.42) + root_flare
        
        row = []
        for ir in range(radial_segments):
            angle = 2.0 * math.pi * ir / radial_segments
            vx = curr_x + math.cos(angle) * r_curr
            vy = curr_y + math.sin(angle) * r_curr
            vz = curr_z
            row.append(bm_stem.verts.new((vx, vy, vz)))
        stem_rows.append(row)
        
    for ih in range(stem_segments):
        for ir in range(radial_segments):
            ir_next = (ir + 1) % radial_segments
            bm_stem.faces.new((stem_rows[ih][ir], stem_rows[ih][ir_next], stem_rows[ih + 1][ir_next], stem_rows[ih + 1][ir]))
            
    bmesh.ops.recalc_face_normals(bm_stem, faces=bm_stem.faces)
    cap_cx = stem_rows[-1][0].co.x
    cap_cy = stem_rows[-1][0].co.y
    cap_cz = stem_rows[-1][0].co.z + 0.55 * scale

    stem_mesh = bpy.data.meshes.new(f"{name}_Stem_Mesh")
    stem_obj = bpy.data.objects.new(f"{name}_Stem", stem_mesh)
    bpy.context.collection.objects.link(stem_obj)
    bm_stem.to_mesh(stem_mesh)
    bm_stem.free()
    for p in stem_mesh.polygons:
        p.use_smooth = True
    stem_mesh.update()
    stem_obj.data.materials.append(mat_shroom_stem)
    
    # 2. Arched Parasol Cap (Downward curving rim with prominent gills)
    r_cap = 1.15 * scale
    bm_cap = bmesh.new()
    cap_radials = 36
    cap_rings = 14
    
    cap_grid = []
    for iring in range(cap_rings + 1):
        t_r = iring / cap_rings
        ring_r = t_r * r_cap
        # Downward dome curvature: gentle at center, dropping smoothly at the rim
        drop_z = (t_r ** 2.2) * 0.62 * scale
        curr_z = cap_cz - drop_z
        
        row = []
        for ia in range(cap_radials):
            angle = 2.0 * math.pi * ia / cap_radials
            rim_wave = 0.0
            if t_r > 0.80:
                rim_wave = math.sin(angle * 6.0) * 0.03 * scale
            vx = cap_cx + math.cos(angle) * (ring_r + rim_wave)
            vy = cap_cy + math.sin(angle) * (ring_r + rim_wave)
            vz = curr_z + rim_wave * 0.3
            row.append(bm_cap.verts.new((vx, vy, vz)))
        cap_grid.append(row)
        
    for iring in range(cap_rings):
        for ia in range(cap_radials):
            ia_next = (ia + 1) % cap_radials
            f = bm_cap.faces.new((cap_grid[iring][ia], cap_grid[iring][ia_next], cap_grid[iring + 1][ia_next], cap_grid[iring + 1][ia]))
            f.material_index = 0 # Pale Blue-Cyan Cap
            
    # Luminous underside gills
    rim_verts = cap_grid[-1]
    inner_neck_z = cap_cz - 0.40 * scale
    neck_verts = []
    for ia in range(cap_radials):
        angle = 2.0 * math.pi * ia / cap_radials
        pleat = math.sin(angle * 18.0) * 0.02 * scale
        vx = cap_cx + math.cos(angle) * (0.22 * scale + pleat)
        vy = cap_cy + math.sin(angle) * (0.22 * scale + pleat)
        vz = inner_neck_z
        neck_verts.append(bm_cap.verts.new((vx, vy, vz)))
        
    for ia in range(cap_radials):
        ia_next = (ia + 1) % cap_radials
        f_gill = bm_cap.faces.new((rim_verts[ia], neck_verts[ia], neck_verts[ia_next], rim_verts[ia_next]))
        f_gill.material_index = 1 # Glowing cyan gills
        
    bmesh.ops.recalc_face_normals(bm_cap, faces=bm_cap.faces)
    cap_mesh = bpy.data.meshes.new(f"{name}_Cap_Mesh")
    cap_obj = bpy.data.objects.new(f"{name}_Cap", cap_mesh)
    bpy.context.collection.objects.link(cap_obj)
    bm_cap.to_mesh(cap_mesh)
    bm_cap.free()
    for p in cap_mesh.polygons:
        p.use_smooth = True
    cap_mesh.update()
    cap_obj.data.materials.append(mat_shroom_pale_cap)
    cap_obj.data.materials.append(mat_shroom_gills)

# Left Side Cluster: Harmonious Tiered Parasol Mushrooms
build_stylized_parasol_mushroom(-4.6, 3.6, scale=1.10, lean_angle=0.12, name="Left_Shroom_Large")
build_stylized_parasol_mushroom(-3.8, 2.8, scale=0.74, lean_angle=0.10, name="Left_Shroom_Medium")
build_stylized_parasol_mushroom(-4.9, 2.6, scale=0.48, lean_angle=0.08, name="Left_Shroom_Small")

# Broad Arching Fern/Palmate Leaves surrounding Left Mushrooms
def spawn_broad_leaf_spray(x, y, z, scale=1.0, num_leaves=7, name="BroadLeafSpray"):
    bm = bmesh.new()
    for il in range(num_leaves):
        ang = (il / float(num_leaves)) * math.pi * 1.8 - 0.4
        length = 1.20 * scale
        width = 0.40 * scale
        
        segs = 6
        left_v = []
        right_v = []
        for iseg in range(segs + 1):
            t = iseg / segs
            dist = t * length
            w = math.sin(t * math.pi) * width * 0.5
            drop = (t ** 1.8) * 0.38 * scale
            lx = x + math.cos(ang) * dist
            ly = y + math.sin(ang) * dist
            lz = z + (t * 0.28 * scale) - drop
            
            px = -math.sin(ang) * w
            py = math.cos(ang) * w
            left_v.append(bm.verts.new((lx - px, ly - py, lz)))
            right_v.append(bm.verts.new((lx + px, ly + py, lz)))
            
        for iseg in range(segs):
            bm.faces.new((left_v[iseg], right_v[iseg], right_v[iseg + 1], left_v[iseg + 1]))
            
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_broad_leaf)

spawn_broad_leaf_spray(-4.0, 2.2, 0.10, scale=1.15, name="Left_Fern_Spray_1")
spawn_broad_leaf_spray(-4.6, 3.4, 0.12, scale=0.95, name="Left_Fern_Spray_2")

# Slender Stalks with Glowing Tips
def spawn_glowing_stalk(x, y, z, h=0.75, name="GlowingStalk"):
    bpy.ops.mesh.primitive_cylinder_add(vertices=8, radius=0.016, depth=h, location=(x, y, z + h * 0.5))
    stem = bpy.context.active_object
    stem.name = f"{name}_Stem"
    stem.data.materials.append(mat_cavern_vine)
    
    bpy.ops.mesh.primitive_uv_sphere_add(segments=10, ring_count=8, radius=0.055, location=(x + 0.03, y, z + h))
    tip = bpy.context.active_object
    tip.name = f"{name}_Tip"
    tip.data.materials.append(mat_glowing_tip)

for idx, (gx, gy) in enumerate([(-3.8, 2.0), (-4.5, 2.1), (-3.2, 3.1), (-4.9, 3.6)]):
    gz = get_terrain_surface_z(gx, gy)
    spawn_glowing_stalk(gx, gy, gz, h=0.65 + idx * 0.12, name=f"Left_GlowingStalk_{idx}")

# ==============================================================================
# 6. RIGHT SIDE FLORA (BEHIND AND AROUND BENCH): 3 VIBRANT ROUNDED CYAN MUSHROOMS
# ==============================================================================
def build_vibrant_dome_mushroom(root_x, root_y, scale=1.0, name="VibrantMushroom"):
    ground_z = get_terrain_surface_z(root_x, root_y)
    root_z = ground_z - 0.15 * scale
    
    # Slender curved stem
    bpy.ops.mesh.primitive_cylinder_add(
        vertices=16, radius=0.20 * scale, depth=2.0 * scale,
        location=(root_x, root_y, root_z + 1.0 * scale)
    )
    stem = bpy.context.active_object
    stem.name = f"{name}_Stem"
    stem.data.materials.append(mat_shroom_stem)
    for p in stem.data.polygons:
        p.use_smooth = True
        
    # High-emission rounded dome cap
    cap_z = root_z + 2.0 * scale
    bpy.ops.mesh.primitive_uv_sphere_add(
        segments=24, ring_count=16, radius=1.05 * scale,
        location=(root_x, root_y, cap_z)
    )
    cap = bpy.context.active_object
    cap.name = f"{name}_Cap"
    cap.scale = (1.0, 1.0, 0.70)
    bpy.ops.object.transform_apply(scale=True)
    cap.data.materials.append(mat_shroom_vibrant_cap)
    for p in cap.data.polygons:
        p.use_smooth = True

# Exactly three rounded dome mushrooms behind the bench
build_vibrant_dome_mushroom(4.2, 3.2, scale=1.22, name="Right_Dome_Mushroom_1")
build_vibrant_dome_mushroom(3.6, 2.5, scale=0.80, name="Right_Dome_Mushroom_2")
build_vibrant_dome_mushroom(5.0, 3.8, scale=0.55, name="Right_Dome_Mushroom_3")

# Tall Grass Clumps behind bench
def spawn_tall_grass_clump(x, y, z, scale=1.0, num_blades=12, name="TallGrass"):
    bm = bmesh.new()
    for ib in range(num_blades):
        ang = (ib / num_blades) * math.pi * 2.0 + (ib * 0.2)
        h = (0.75 + (ib % 3) * 0.15) * scale
        w = 0.05 * scale
        tilt = 0.18 * scale
        
        v0_l = bm.verts.new((x + math.cos(ang) * 0.04 - math.sin(ang) * w * 0.5, y + math.sin(ang) * 0.04 + math.cos(ang) * w * 0.5, z))
        v0_r = bm.verts.new((x + math.cos(ang) * 0.04 + math.sin(ang) * w * 0.5, y + math.sin(ang) * 0.04 - math.cos(ang) * w * 0.5, z))
        
        v1_l = bm.verts.new((x + math.cos(ang) * (0.04 + tilt * 0.5) - math.sin(ang) * w * 0.35, y + math.sin(ang) * (0.04 + tilt * 0.5) + math.cos(ang) * w * 0.35, z + h * 0.55))
        v1_r = bm.verts.new((x + math.cos(ang) * (0.04 + tilt * 0.5) + math.sin(ang) * w * 0.35, y + math.sin(ang) * (0.04 + tilt * 0.5) - math.cos(ang) * w * 0.35, z + h * 0.55))
        
        tip = bm.verts.new((x + math.cos(ang) * (0.04 + tilt), y + math.sin(ang) * (0.04 + tilt), z + h))
        
        bm.faces.new((v0_l, v0_r, v1_r, v1_l))
        bm.faces.new((v1_l, v1_r, tip))
        
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_tall_grass)

for idx, (tx, ty) in enumerate([(3.8, 2.2), (4.5, 2.7), (3.4, 2.9), (4.8, 3.4)]):
    tz = get_terrain_surface_z(tx, ty)
    spawn_tall_grass_clump(tx, ty, tz, scale=1.1, name=f"Right_TallGrass_{idx}")

# Glowing Bulb Plants
def spawn_bulb_plant(x, y, z, name="BulbPlant"):
    for ib in range(4):
        ang = (ib / 4.0) * math.pi * 2.0
        bx = x + math.cos(ang) * 0.18
        by = y + math.sin(ang) * 0.18
        bz = z + 0.28 + (ib % 2) * 0.10
        
        bpy.ops.mesh.primitive_cylinder_add(vertices=6, radius=0.012, depth=0.35, location=(x + math.cos(ang) * 0.09, y + math.sin(ang) * 0.09, z + 0.16))
        stem = bpy.context.active_object
        stem.name = f"{name}_Stem_{ib}"
        stem.rotation_euler = (0.2 * math.sin(ang), 0.2 * math.cos(ang), 0.0)
        stem.data.materials.append(mat_cavern_vine)
        
        bpy.ops.mesh.primitive_uv_sphere_add(segments=10, ring_count=8, radius=0.065, location=(bx, by, bz))
        bulb = bpy.context.active_object
        bulb.name = f"{name}_Bulb_{ib}"
        bulb.data.materials.append(mat_glow_bulb)

for idx, (bx, by) in enumerate([(4.1, 1.9), (4.6, 2.3), (3.5, 2.8)]):
    bz = get_terrain_surface_z(bx, by)
    spawn_bulb_plant(bx, by, bz, name=f"Right_BulbPlant_{idx}")

# ==============================================================================
# 7. WATERFALL IMPACT FROTH BILLOWS & STYLIZED SURFACE RIPPLE RINGS
# ==============================================================================
def spawn_waterfall_impact_froth_billows():
    """
    Creates the puffy billowing cloud of impact froth at the base of the waterfall,
    matching the artwork's white/cyan splash foam cushions.
    """
    foam_puffs = [
        (0.0, 8.5, 0.12, 0.55, 0.45, 0.28),
        (-0.6, 8.4, 0.10, 0.48, 0.40, 0.24),
        (0.6, 8.4, 0.10, 0.48, 0.40, 0.24),
        (-1.1, 8.3, 0.08, 0.42, 0.36, 0.20),
        (1.1, 8.3, 0.08, 0.42, 0.36, 0.20),
        (-0.3, 8.2, 0.09, 0.45, 0.38, 0.22),
        (0.3, 8.2, 0.09, 0.45, 0.38, 0.22),
    ]
    for idx, (fx, fy, fz, fsx, fsy, fsz) in enumerate(foam_puffs):
        bpy.ops.mesh.primitive_uv_sphere_add(segments=12, ring_count=8, radius=1.0, location=(fx, fy, fz))
        puff = bpy.context.active_object
        puff.name = f"Waterfall_Foam_Billow_{idx}"
        puff.scale = (fsx, fsy, fsz)
        bpy.ops.object.transform_apply(scale=True)
        puff.data.materials.append(mat_water_foam_billow)
        for p in puff.data.polygons:
            p.use_smooth = True

spawn_waterfall_impact_froth_billows()

def spawn_water_ripple_rings():
    """
    Creates concentric stylized turquoise ripple rings radiating across the pool.
    """
    ring_radii = [1.8, 3.2, 4.4]
    lake_cx = 0.0
    lake_cy = 8.5
    
    for idx, rad in enumerate(ring_radii):
        bm = bmesh.new()
        segs = 48
        w = 0.06
        z_ring = 0.06
        for iseg in range(segs + 1):
            ang = (iseg / float(segs)) * math.pi * 2.0
            r_in = rad - w * 0.5
            r_out = rad + w * 0.5
            v_in = bm.verts.new((lake_cx + math.cos(ang) * r_in, lake_cy + math.sin(ang) * r_in, z_ring))
            v_out = bm.verts.new((lake_cx + math.cos(ang) * r_out, lake_cy + math.sin(ang) * r_out, z_ring))
            
        bm.verts.ensure_lookup_table()
        for iseg in range(segs):
            i0_in = iseg * 2
            i0_out = iseg * 2 + 1
            i1_in = (iseg + 1) * 2
            i1_out = (iseg + 1) * 2 + 1
            bm.faces.new((bm.verts[i0_in], bm.verts[i1_in], bm.verts[i1_out], bm.verts[i0_out]))
            
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
        mesh = bpy.data.meshes.new(f"Water_Ripple_Ring_{idx}_Mesh")
        obj = bpy.data.objects.new(f"Water_Ripple_Ring_{idx}", mesh)
        bpy.context.collection.objects.link(obj)
        bm.to_mesh(mesh)
        bm.free()
        for p in mesh.polygons:
            p.use_smooth = True
        mesh.update()
        obj.data.materials.append(mat_water_ripple_ring)

spawn_water_ripple_rings()

# ==============================================================================
# 8. EXPORT COMPLETE GLB MODEL
# ==============================================================================
output_path = os.path.abspath("assets/models/cavern_kit.glb")
os.makedirs(os.path.dirname(output_path), exist_ok=True)

bpy.ops.export_scene.gltf(
    filepath=output_path,
    export_format='GLB',
    use_selection=False,
    export_apply=True,
    export_materials='EXPORT',
)

print(f">>> [BLENDER 4.5] Successfully exported Concept-Faithful Cavern Kit to: {output_path}")
