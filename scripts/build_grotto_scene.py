"""
Masterwork Procedural 3D Cavern Grotto Generator - Concept Artwork Fidelity Edition
Executes inside Blender 4.5 background mode.
Generates assets/models/cavern_kit.glb adhering strictly to the Art Reference Directives:

1. Estética e Iluminação Ambiental (Cel-shaded ink lines, soft skylight shaft, cyan bioluminescence)
2. Primeiro Plano (Largo caminho central de terra batida marrom sem grama, seixos soltos, laterais gramadas com pedras angulares)
3. Flora Bioluminescente:
   - Lado esquerdo: Cogumelos altos inclinados com chapéu azul-ciano pálido e lamelas visíveis, plantas de folhas largas e hastes com pontas brilhantes.
   - Lado direito: Três cogumelos de chapéu arredondado com emissão intensa azul-turquesa/ciano, grama alta e plantas com bulbos esféricos luminosos.
4. Plano Médio (Lagoa circular com fina borda curva de pedra nivelada com a água azul-turquesa)
5. Plano de Fundo e Teto (Paredes de blocos de rocha facetados com musgo superior e vinhas penduradas; teto em arco com estalactites pontiagudas laterais).

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
mat_rock_shadow = create_pbr_material("M_Rock_Shadow", (0.13, 0.17, 0.22), roughness=0.88)
mat_rock_lit = create_pbr_material("M_Rock_Lit", (0.24, 0.32, 0.36), roughness=0.78)
mat_cavern_moss = create_pbr_material("M_Cavern_Moss", (0.14, 0.48, 0.20), roughness=0.85)
mat_cavern_vine = create_pbr_material("M_Cavern_Vine", (0.20, 0.58, 0.26), roughness=0.72)
mat_ground_path = create_pbr_material("M_Ground_Path", (0.28, 0.16, 0.08), roughness=0.90) # Rich warm brown #472914 beaten earth
mat_ground_grass = create_pbr_material("M_Ground_Grass", (0.12, 0.46, 0.18), roughness=0.82)
mat_fine_rim = create_pbr_material("M_Fine_Stone_Rim", (0.24, 0.25, 0.27), roughness=0.70)
mat_river_pebble = create_pbr_material("M_River_Pebble", (0.35, 0.35, 0.36), roughness=0.60)
mat_angular_rock = create_pbr_material("M_Angular_Rock", (0.22, 0.24, 0.26), roughness=0.85)

# Left Side Flora: Pale Blue-Cyan Mushroom & Foliage
mat_shroom_pale_cap = create_pbr_material("M_Shroom_Pale_Cap", (0.38, 0.76, 0.88), roughness=0.35, emission=(0.30, 0.72, 0.85), emission_strength=1.4)
mat_shroom_gills = create_pbr_material("M_Shroom_Gills", (0.12, 0.65, 0.70), roughness=0.45, emission=(0.14, 0.72, 0.78), emission_strength=1.5)
mat_shroom_stem = create_pbr_material("M_Shroom_Stem", (0.78, 0.84, 0.80), roughness=0.60, emission=(0.08, 0.26, 0.24), emission_strength=0.8)
mat_broad_leaf = create_pbr_material("M_Broad_Leaf", (0.16, 0.50, 0.22), roughness=0.60)
mat_glowing_tip = create_pbr_material("M_Glowing_Tip", (0.25, 0.95, 0.92), roughness=0.20, emission=(0.20, 0.98, 0.95), emission_strength=2.5)

# Right Side Flora: Intense Cyan/Turquoise Mushrooms, Tall Grass & Glowing Bulbs
mat_shroom_vibrant_cap = create_pbr_material("M_Shroom_Vibrant_Cap", (0.04, 0.82, 0.85), roughness=0.20, emission=(0.05, 0.90, 0.92), emission_strength=1.8)
mat_tall_grass = create_pbr_material("M_Tall_Grass", (0.18, 0.54, 0.24), roughness=0.70)
mat_glow_bulb = create_pbr_material("M_Glow_Bulb", (0.15, 0.95, 0.90), roughness=0.15, emission=(0.15, 0.98, 0.92), emission_strength=2.8)

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
# 1. FULLY-ENCLOSED VAULTED CAVERN DOME & CEILING ARCH WITH OVERHEAD STALACTITES
# ==============================================================================
def build_cavern_amphitheater():
    bm = bmesh.new()
    segments_u = 48
    segments_v = 36
    
    center_y = 7.5
    h_top = 23.5  # Completely covers the upper field of view
    
    wall_grid = []
    random.seed(101)
    
    for iv in range(segments_v + 1):
        v_frac = iv / segments_v
        curr_z = v_frac * h_top
        
        dome_t = 0.0
        if curr_z > 7.5:
            dome_t = (curr_z - 7.5) / (h_top - 7.5)
            
        row = []
        for iu in range(segments_u + 1):
            u_frac = iu / segments_u
            angle = -math.pi * 0.32 + u_frac * math.pi * 1.64
            
            forward_expansion = max(0.0, -math.sin(angle)) * 4.2
            base_r = 10.4 + forward_expansion + math.sin(v_frac * math.pi * 2.5) * 0.45
            
            overhang = (dome_t ** 1.32) * (base_r - 0.45)
            curr_r = max(0.42, base_r - overhang)
            
            x = math.cos(angle) * curr_r
            y = center_y + math.sin(angle) * curr_r
            z = curr_z
            
            block_disp = (math.sin(x * 0.5 + z * 0.35) * math.cos(y * 0.45)) * 0.70
            x += math.cos(angle) * block_disp
            y += math.sin(angle) * block_disp
            
            # Central cleft opening for waterfall source drop
            if abs(x) < 2.2 and z > 14.0 and z < 20.0 and y > 6.0:
                y += 2.0
                
            vert = bm.verts.new((x, y, z))
            row.append(vert)
        wall_grid.append(row)
        
    bm.verts.ensure_lookup_table()
    for iv in range(segments_v):
        for iu in range(segments_u):
            v1 = wall_grid[iv][iu]
            v2 = wall_grid[iv][iu + 1]
            v3 = wall_grid[iv + 1][iu + 1]
            v4 = wall_grid[iv + 1][iu]
            face = bm.faces.new((v1, v2, v3, v4))
            
            center = face.calc_center_median()
            is_cleft_lit = abs(center.x) < 3.8 and center.z > 5.0 and center.z < 18.0
            is_vault_strata = center.z > 14.0 and (int(center.z) % 2 == 0)
            is_mid_strata = center.z > 5.0 and center.z < 12.0 and abs(center.x) < 7.5 and (int(center.z) % 3 != 0)
            is_moss_shelf = center.z < 3.8 and (iu % 3 != 0)
            
            if is_moss_shelf:
                face.material_index = 2 # Mossy rock
            elif is_cleft_lit or is_vault_strata or is_mid_strata:
                face.material_index = 1 # Lit rock
            else:
                face.material_index = 0 # Shadow rock

    apex_center = bm.verts.new((0.0, center_y, h_top))
    top_row = wall_grid[-1]
    for iu in range(segments_u):
        face = bm.faces.new((top_row[iu], top_row[iu + 1], apex_center))
        face.material_index = 1

    mesh = bpy.data.meshes.new("Cavern_Amphitheater_Mesh")
    obj = bpy.data.objects.new("Cavern_Amphitheater", mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for poly in mesh.polygons:
        poly.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)

build_cavern_amphitheater()

# ==============================================================================
# 2. OVERHEAD SHARP POINTED STALACTITES ANCHORED IN VAULTED ARCH (Teto e Estalactites)
# ==============================================================================
def spawn_organic_stalactite(x, y, z_top, height=3.2, radius=0.52, tilt=(0.0, 0.0), name="OrganicStalactite"):
    bm = bmesh.new()
    rings = 14
    radials = 16
    
    ring_verts = []
    for ir in range(rings + 1):
        t = ir / rings
        root_flare = ((1.0 - t) ** 4.0) * 0.42
        mineral_bumps = math.sin(t * math.pi * 5.0) * 0.05 * radius
        curr_r = max(0.02, radius * (1.0 - t * 0.94) + root_flare + mineral_bumps)
        
        curr_x = x + tilt[0] * (t ** 1.3) + math.sin(t * 7.0) * 0.03
        curr_y = y + tilt[1] * (t ** 1.3) + math.cos(t * 7.0) * 0.03
        curr_z = z_top - t * height
        
        row = []
        for ia in range(radials):
            angle = 2.0 * math.pi * ia / radials
            r_wobble = curr_r * (1.0 + math.sin(angle * 3.0 + t * 4.0) * 0.06)
            vx = curr_x + math.cos(angle) * r_wobble
            vy = curr_y + math.sin(angle) * r_wobble
            vz = curr_z
            vert = bm.verts.new((vx, vy, vz))
            row.append(vert)
        ring_verts.append(row)
        
    bm.verts.ensure_lookup_table()
    
    for ir in range(rings):
        t = (ir + 0.5) / rings
        if t < 0.22:
            mat_idx = 2  # Moss base (merges with ceiling)
        elif t < 0.72:
            mat_idx = 1  # Lit rock
        else:
            mat_idx = 0  # Wet dark shadow tip
            
        for ia in range(radials):
            ia_next = (ia + 1) % radials
            v1 = ring_verts[ir][ia]
            v2 = ring_verts[ir][ia_next]
            v3 = ring_verts[ir + 1][ia_next]
            v4 = ring_verts[ir + 1][ia]
            face = bm.faces.new((v1, v2, v3, v4))
            face.material_index = mat_idx
            
    tip_center = bm.verts.new((x + tilt[0] + 0.01, y + tilt[1], z_top - height))
    for ia in range(radials):
        ia_next = (ia + 1) % radials
        face = bm.faces.new((ring_verts[rings][ia], ring_verts[rings][ia_next], tip_center))
        face.material_index = 0
        
    top_center = bm.verts.new((x, y, z_top))
    for ia in range(radials):
        ia_next = (ia + 1) % radials
        face = bm.faces.new((ring_verts[0][ia_next], ring_verts[0][ia], top_center))
        face.material_index = 2
        
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    
    for poly in mesh.polygons:
        poly.use_smooth = True
    mesh.update()
    
    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)
    return obj

organic_stalactites = [
    # Left ceiling vault clusters flanking the waterfall
    (-6.5, 4.5, 17.5, 4.2, 0.62, (0.08, -0.05)),
    (-5.2, 6.2, 19.2, 3.8, 0.52, (-0.05, 0.06)),
    (-3.8, 7.8, 20.5, 3.5, 0.48, (0.06, -0.04)),
    (-5.8, 3.2, 16.8, 3.0, 0.42, (0.04, 0.05)),
    (-3.5, 4.8, 18.2, 2.8, 0.38, (-0.06, -0.04)),
    (-2.2, 6.5, 21.0, 3.2, 0.40, (0.05, 0.04)),
    # Right ceiling vault clusters flanking the waterfall
    (6.5, 4.5, 17.5, 4.2, 0.62, (-0.08, -0.05)),
    (5.2, 6.2, 19.2, 3.8, 0.52, (0.05, 0.06)),
    (3.8, 7.8, 20.5, 3.5, 0.48, (-0.06, -0.04)),
    (5.8, 3.2, 16.8, 3.0, 0.42, (-0.04, 0.05)),
    (3.5, 4.8, 18.2, 2.8, 0.38, (0.06, -0.04)),
    (2.2, 6.5, 21.0, 3.2, 0.40, (-0.05, 0.04)),
    # Overhead forward framing
    (-4.0, 1.8, 17.5, 2.6, 0.36, (0.05, 0.06)),
    (4.0, 1.8, 17.5, 2.6, 0.36, (-0.05, 0.06)),
]
for idx, (sx, sy, sz, sh, sr, stilt) in enumerate(organic_stalactites):
    spawn_organic_stalactite(sx, sy, sz, height=sh, radius=sr, tilt=stilt, name=f"CeilingStalactite_{idx}")

# ==============================================================================
# 3. LAYERED ROCK WALLS WITH UPPER MOSS AND HANGING VINES (Paredes Rochosas e Musgo)
# ==============================================================================
def build_layered_rock_terraces_with_vines():
    bm = bmesh.new()
    
    # Layered, stepped faceted rock shelves behind and flanking the waterfall
    slabs = [
        # Left stepped rock tiers
        (-6.2, 7.2, 0.05, 1.25, 2.8, 2.4),
        (-6.8, 8.5, 1.05, 2.35, 2.5, 2.1),
        (-7.2, 9.4, 2.10, 3.60, 2.3, 1.9),
        (-5.5, 6.2, 0.05, 0.85, 2.2, 2.0),
        # Right stepped rock tiers
        (6.2, 7.2, 0.05, 1.25, 2.8, 2.4),
        (6.8, 8.5, 1.05, 2.35, 2.5, 2.1),
        (7.2, 9.4, 2.10, 3.60, 2.3, 1.9),
        (5.5, 6.2, 0.05, 0.85, 2.2, 2.0),
    ]
    
    for (cx, cy, zb, zt, sx, sy) in slabs:
        num_corners = 7
        bot_verts = []
        top_verts = []
        for ic in range(num_corners):
            ang = 2.0 * math.pi * ic / num_corners
            rad_x = (sx * 0.5) * (0.85 + math.sin(ic * 2.3) * 0.15)
            rad_y = (sy * 0.5) * (0.85 + math.cos(ic * 1.7) * 0.15)
            vx = cx + math.cos(ang) * rad_x
            vy = cy + math.sin(ang) * rad_y
            bot_verts.append(bm.verts.new((vx, vy, zb)))
            # Chamfer top inward for natural chiseled rock bevel
            top_verts.append(bm.verts.new((vx * 0.94 + cx * 0.06, vy * 0.94 + cy * 0.06, zt)))
            
        for ic in range(num_corners):
            ic_next = (ic + 1) % num_corners
            f = bm.faces.new((bot_verts[ic], bot_verts[ic_next], top_verts[ic_next], top_verts[ic]))
            f.material_index = 1 # Chiseled lit rock face
            
        f_top = bm.faces.new(top_verts)
        f_top.material_index = 2 # Thick green moss on the upper horizontal surface!
        
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new("Layered_Rock_Terraces_Mesh")
    obj = bpy.data.objects.new("Layered_Rock_Terraces", mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for poly in mesh.polygons:
        poly.use_smooth = False # Faceted rock appearance!
    mesh.update()
    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)

build_layered_rock_terraces_with_vines()

# Hanging Vines and Cascading Foliage draping down the rock fissures
def spawn_hanging_vine(x, y, z_top, length=1.8, segments=10, name="HangingVine"):
    bm = bmesh.new()
    prev_v = None
    width = 0.07
    rows = []
    for iseg in range(segments + 1):
        t = iseg / segments
        cz = z_top - t * length
        cx = x + math.sin(t * math.pi * 3.0) * 0.08
        cy = y + (t * 0.04)
        w = width * (1.0 - t * 0.45)
        v_l = bm.verts.new((cx - w * 0.5, cy, cz))
        v_r = bm.verts.new((cx + w * 0.5, cy, cz))
        rows.append((v_l, v_r))
        
        # Leaf buds sprouting sideways
        if iseg % 2 == 1:
            side = 1.0 if (iseg % 4 == 1) else -1.0
            leaf_tip = bm.verts.new((cx + side * (w + 0.12), cy - 0.03, cz - 0.04))
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
    # Left cliff fissure vines
    (-6.5, 8.2, 2.3, 1.8),
    (-5.8, 7.5, 1.2, 1.4),
    (-7.1, 9.1, 3.5, 2.2),
    (-6.1, 6.8, 1.1, 1.3),
    # Right cliff fissure vines
    (6.5, 8.2, 2.3, 1.8),
    (5.8, 7.5, 1.2, 1.4),
    (7.1, 9.1, 3.5, 2.2),
    (6.1, 6.8, 1.1, 1.3),
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
    terrain.data.materials.append(mat_ground_path)  # Material 0: Beaten brown earth #6b4c35
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
            f.material_index = 0 # Pure beaten brown dirt #6b4c35 (TOTALMENTE LIVRE DE GRAMA)
        else:
            f.material_index = 1 # Lateral dense green grass banks

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(terrain.data)
    bm.free()
    for p in terrain.data.polygons:
        p.use_smooth = True
    terrain.data.update()

    # Fine, subtle curved stone rim leveled with the water (Fina borda curva de pedra nivelada)
    bm_rim = bmesh.new()
    rim_pts = 64
    rim_grid = []
    rim_top_z = 0.08  # Leveled with water surface at 0.05
    rim_bot_z = -0.06
    
    for ip in range(rim_pts + 1):
        ang = 2.0 * math.pi * ip / rim_pts
        cos_a = math.cos(ang)
        sin_a = math.sin(ang)
        
        # Fine stone ribbon (width ~ 0.35m, flush with terrain & water)
        v0 = bm_rim.verts.new((lake_center_x + cos_a * (lake_radius - 0.12), lake_center_y + sin_a * (lake_radius - 0.12), rim_bot_z))
        v1 = bm_rim.verts.new((lake_center_x + cos_a * (lake_radius - 0.04), lake_center_y + sin_a * (lake_radius - 0.04), rim_top_z))
        v2 = bm_rim.verts.new((lake_center_x + cos_a * (lake_radius + 0.22), lake_center_y + sin_a * (lake_radius + 0.22), rim_top_z - 0.01))
        v3 = bm_rim.verts.new((lake_center_x + cos_a * (lake_radius + 0.42), lake_center_y + sin_a * (lake_radius + 0.42), rim_bot_z))
        rim_grid.append((v0, v1, v2, v3))
        
    for ip in range(rim_pts):
        r0 = rim_grid[ip]
        r1 = rim_grid[ip + 1]
        bm_rim.faces.new((r0[0], r0[1], r1[1], r1[0]))
        bm_rim.faces.new((r0[1], r0[2], r1[2], r1[1]))
        bm_rim.faces.new((r0[2], r0[3], r1[3], r1[2]))
        
    bmesh.ops.recalc_face_normals(bm_rim, faces=bm_rim.faces)
    rim_mesh = bpy.data.meshes.new("Lake_Stone_Rim_Mesh")
    rim_obj = bpy.data.objects.new("Lake_Stone_Rim", rim_mesh)
    bpy.context.collection.objects.link(rim_obj)
    bm_rim.to_mesh(rim_mesh)
    bm_rim.free()
    for p in rim_mesh.polygons:
        p.use_smooth = True
    rim_mesh.update()
    rim_obj.data.materials.append(mat_fine_rim)

build_terrain_and_lake_rim()

# Loose River Pebbles along the dirt path borders
def spawn_river_pebble(x, y, z, size_x=0.28, size_y=0.22):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=10, ring_count=6, radius=size_x, location=(x, y, z))
    stone = bpy.context.active_object
    stone.scale = (1.0, size_y / size_x, 0.38)
    bpy.ops.object.transform_apply(scale=True)
    stone.data.materials.append(mat_river_pebble)
    for p in stone.data.polygons:
        p.use_smooth = True

pebble_coords = [
    # Left edge loose pebbles
    (-2.3, 0.8, 0.06, 0.26, 0.20),
    (-2.4, 1.8, 0.08, 0.32, 0.24),
    (-2.5, 2.9, 0.09, 0.28, 0.22),
    (-2.7, 3.8, 0.10, 0.35, 0.28),
    # Right edge loose pebbles
    (2.3, 0.8, 0.06, 0.26, 0.20),
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
# 5. LEFT SIDE FLORA: TALL LEANING MUSHROOMS WITH PALE BLUE CAPS & GILLS
# ==============================================================================
def build_pale_leaning_mushroom(root_x, root_y, scale=1.0, lean_dir=1.0, name="PaleMushroom"):
    ground_z = get_terrain_surface_z(root_x, root_y)
    root_z = ground_z - 0.20 * scale
    
    # Stem
    bm_stem = bmesh.new()
    stem_segments = 18
    radial_segments = 18
    h_stem = 2.6 * scale
    r_base = 0.30 * scale
    
    stem_rows = []
    for ih in range(stem_segments + 1):
        t = ih / stem_segments
        curr_z = root_z + t * h_stem
        curr_x = root_x + math.sin(t * math.pi * 0.65) * 0.45 * lean_dir * scale
        curr_y = root_y + (t * 0.18) * scale
        
        flare_factor = (((0.30 - t) / 0.30) ** 1.8) if t < 0.30 else 0.0
        root_flare = flare_factor * 0.45 * scale
        
        row = []
        for ir in range(radial_segments):
            angle = 2.0 * math.pi * ir / radial_segments
            effective_r = r_base * (1.0 - t * 0.45) + root_flare
            vx = curr_x + math.cos(angle) * effective_r
            vy = curr_y + math.sin(angle) * effective_r
            vz = curr_z
            row.append(bm_stem.verts.new((vx, vy, vz)))
        stem_rows.append(row)
        
    for ih in range(stem_segments):
        for ir in range(radial_segments):
            ir_next = (ir + 1) % radial_segments
            bm_stem.faces.new((stem_rows[ih][ir], stem_rows[ih][ir_next], stem_rows[ih + 1][ir_next], stem_rows[ih + 1][ir]))
            
    bottom_center = bm_stem.verts.new((root_x, root_y, root_z))
    for ir in range(radial_segments):
        ir_next = (ir + 1) % radial_segments
        bm_stem.faces.new((stem_rows[0][ir_next], stem_rows[0][ir], bottom_center))

    bmesh.ops.recalc_face_normals(bm_stem, faces=bm_stem.faces)
    cap_top_z = stem_rows[-1][0].co.z + 0.68 * scale
    cap_center_x = stem_rows[-1][0].co.x
    cap_center_y = stem_rows[-1][0].co.y
    r_cap_max = 1.70 * scale

    stem_mesh = bpy.data.meshes.new(f"{name}_Stem_Mesh")
    stem_obj = bpy.data.objects.new(f"{name}_Stem", stem_mesh)
    bpy.context.collection.objects.link(stem_obj)
    bm_stem.to_mesh(stem_mesh)
    bm_stem.free()
    for p in stem_mesh.polygons:
        p.use_smooth = True
    stem_mesh.update()
    stem_obj.data.materials.append(mat_shroom_stem)

    # Parasol Cap with Pale Blue-Cyan finish
    bm_cap = bmesh.new()
    cap_radial = 48
    cap_rings = 16
    
    cap_rows = []
    for iring in range(cap_rings + 1):
        t_ring = iring / cap_rings
        ring_r = t_ring * r_cap_max
        dome_drop = (1.0 - math.cos(t_ring * math.pi * 0.5)) * 0.70 * scale
        ring_z = cap_top_z - dome_drop
        
        row = []
        for ir in range(cap_radial):
            angle = 2.0 * math.pi * ir / cap_radial
            rim_wave = 0.0
            if t_ring > 0.70:
                rim_wave = math.sin(angle * 6.0) * 0.04 * ((t_ring - 0.70) / 0.30) * scale
                
            vx = cap_center_x + math.cos(angle) * (ring_r + rim_wave)
            vy = cap_center_y + math.sin(angle) * (ring_r + rim_wave) - (t_ring * 0.20 * scale)
            vz = ring_z + rim_wave * 0.5
            row.append(bm_cap.verts.new((vx, vy, vz)))
        cap_rows.append(row)
        
    for iring in range(cap_rings):
        for ir in range(cap_radial):
            ir_next = (ir + 1) % cap_radial
            face = bm_cap.faces.new((cap_rows[iring][ir], cap_rows[iring][ir_next], cap_rows[iring + 1][ir_next], cap_rows[iring + 1][ir]))
            face.material_index = 0 # Pale Blue-Cyan Cap
            
    # Detailed Underside Gills (Lamelas Visíveis)
    rim_row = cap_rows[-1]
    neck_center_z = cap_top_z - 0.60 * scale
    neck_r = 0.28 * scale
    inner_verts = []
    for ir in range(cap_radial):
        angle = 2.0 * math.pi * ir / cap_radial
        gill_pleat = (math.sin(angle * 24.0) * 0.03 * scale) # Radial gill ridges!
        vx = cap_center_x + math.cos(angle) * (neck_r + gill_pleat)
        vy = cap_center_y + math.sin(angle) * (neck_r + gill_pleat)
        vz = neck_center_z
        inner_verts.append(bm_cap.verts.new((vx, vy, vz)))
        
    for ir in range(cap_radial):
        ir_next = (ir + 1) % cap_radial
        f_under = bm_cap.faces.new((rim_row[ir], inner_verts[ir], inner_verts[ir_next], rim_row[ir_next]))
        f_under.material_index = 1 # Bioluminescent cyan gills
        
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

# Left Side Cluster: Tall Leaning Mushrooms with Pale Blue Caps & Gills
build_pale_leaning_mushroom(-5.0, 3.2, scale=1.28, lean_dir=0.9, name="Left_Shroom_Large")
build_pale_leaning_mushroom(-4.3, 2.3, scale=0.74, lean_dir=0.7, name="Left_Shroom_Medium")
build_pale_leaning_mushroom(-5.6, 3.8, scale=0.48, lean_dir=0.5, name="Left_Shroom_Small")

# Broad-Leafed Plants around left mushrooms (Plantas de Folhas Largas)
def spawn_broad_leaf_plant(x, y, z, scale=1.0, name="BroadLeafPlant"):
    bm = bmesh.new()
    num_leaves = 6
    for il in range(num_leaves):
        ang = (il / num_leaves) * math.pi * 2.0
        length = 0.95 * scale
        width = 0.35 * scale
        
        # Curved broad leaf mesh
        segs = 6
        left_v = []
        right_v = []
        for iseg in range(segs + 1):
            t = iseg / segs
            dist = t * length
            w = math.sin(t * math.pi) * width * 0.5
            drop = (t ** 2.0) * 0.32 * scale
            lx = x + math.cos(ang) * dist
            ly = y + math.sin(ang) * dist
            lz = z + (t * 0.20 * scale) - drop
            
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

spawn_broad_leaf_plant(-4.6, 1.8, 0.10, scale=1.1, name="Left_Broad_Leaf_1")
spawn_broad_leaf_plant(-5.3, 2.8, 0.12, scale=0.9, name="Left_Broad_Leaf_2")

# Slender Stalks with Glowing Tips (Hastes com Pontas Brilhantes)
def spawn_glowing_stalk(x, y, z, h=0.75, name="GlowingStalk"):
    bm = bmesh.new()
    segs = 6
    stem_v = []
    for iseg in range(segs + 1):
        t = iseg / segs
        sz = z + t * h
        sx = x + math.sin(t * 3.0) * 0.05
        sy = y + math.cos(t * 3.0) * 0.04
        stem_v.append((sx, sy, sz))
    
    # Stem cylinder
    bpy.ops.mesh.primitive_cylinder_add(vertices=8, radius=0.018, depth=h, location=(x, y, z + h * 0.5))
    stem = bpy.context.active_object
    stem.name = f"{name}_Stem"
    stem.data.materials.append(mat_cavern_vine)
    
    # Glowing spherical tip
    bpy.ops.mesh.primitive_uv_sphere_add(segments=10, ring_count=8, radius=0.055, location=(x + 0.04, y, z + h))
    tip = bpy.context.active_object
    tip.name = f"{name}_Tip"
    tip.data.materials.append(mat_glowing_tip)

for idx, (gx, gy) in enumerate([(-4.2, 2.0), (-4.7, 2.2), (-5.1, 1.9), (-4.4, 3.6)]):
    gz = get_terrain_surface_z(gx, gy)
    spawn_glowing_stalk(gx, gy, gz, h=0.65 + idx * 0.1, name=f"Left_GlowingStalk_{idx}")

# ==============================================================================
# 6. RIGHT SIDE FLORA (ATRAS DO BANCO): 3 COGUMELOS ARREDONDADOS COM EMISSAO INTENSA,
#    GRAMA ALTA E BULBOS ESFÉRICOS LUMINOSOS
# ==============================================================================
def build_vibrant_dome_mushroom(root_x, root_y, scale=1.0, name="VibrantMushroom"):
    ground_z = get_terrain_surface_z(root_x, root_y)
    root_z = ground_z - 0.20 * scale
    
    # Stem
    bpy.ops.mesh.primitive_cylinder_add(
        vertices=16, radius=0.22 * scale, depth=2.1 * scale,
        location=(root_x, root_y, root_z + 1.05 * scale)
    )
    stem = bpy.context.active_object
    stem.name = f"{name}_Stem"
    stem.data.materials.append(mat_shroom_stem)
    for p in stem.data.polygons:
        p.use_smooth = True
        
    # High-emission rounded dome cap (Chapéu Arredondado com Emissão Intensa Ciano/Turquesa)
    cap_z = root_z + 2.1 * scale
    bpy.ops.mesh.primitive_uv_sphere_add(
        segments=24, ring_count=16, radius=1.15 * scale,
        location=(root_x, root_y, cap_z)
    )
    cap = bpy.context.active_object
    cap.name = f"{name}_Cap"
    cap.scale = (1.0, 1.0, 0.72) # Smooth spherical dome cap
    bpy.ops.object.transform_apply(scale=True)
    cap.data.materials.append(mat_shroom_vibrant_cap)
    for p in cap.data.polygons:
        p.use_smooth = True

# EXACTLY THREE rounded dome mushrooms with intense cyan emission behind the bench!
build_vibrant_dome_mushroom(4.6, 3.1, scale=1.20, name="Right_Dome_Mushroom_1")
build_vibrant_dome_mushroom(4.0, 2.3, scale=0.76, name="Right_Dome_Mushroom_2")
build_vibrant_dome_mushroom(5.3, 3.7, scale=0.52, name="Right_Dome_Mushroom_3")

# Tall Grass Clumps behind bench and around right mushrooms (Grama Alta)
def spawn_tall_grass_clump(x, y, z, scale=1.0, num_blades=12, name="TallGrass"):
    bm = bmesh.new()
    for ib in range(num_blades):
        ang = (ib / num_blades) * math.pi * 2.0 + (ib * 0.2)
        h = (0.75 + (ib % 3) * 0.15) * scale
        w = 0.05 * scale
        tilt = 0.18 * scale
        
        # 3-segment curving blade
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

for idx, (tx, ty) in enumerate([(4.1, 2.0), (4.7, 2.5), (3.7, 2.7), (5.1, 3.2)]):
    tz = get_terrain_surface_z(tx, ty)
    spawn_tall_grass_clump(tx, ty, tz, scale=1.1, name=f"Right_TallGrass_{idx}")

# Plants with Small Glowing Spherical Bulbs (Plantas com Pequenos Bulbos Esféricos Luminosos)
def spawn_bulb_plant(x, y, z, name="BulbPlant"):
    bm = bmesh.new()
    num_bulbs = 4
    for ib in range(num_bulbs):
        ang = (ib / num_bulbs) * math.pi * 2.0
        bx = x + math.cos(ang) * 0.18
        by = y + math.sin(ang) * 0.18
        bz = z + 0.28 + (ib % 2) * 0.10
        
        # Arching stem
        bpy.ops.mesh.primitive_cylinder_add(vertices=6, radius=0.012, depth=0.35, location=(x + math.cos(ang) * 0.09, y + math.sin(ang) * 0.09, z + 0.16))
        stem = bpy.context.active_object
        stem.name = f"{name}_Stem_{ib}"
        stem.rotation_euler = (0.2 * math.sin(ang), 0.2 * math.cos(ang), 0.0)
        stem.data.materials.append(mat_cavern_vine)
        
        # Glowing spherical bulb
        bpy.ops.mesh.primitive_uv_sphere_add(segments=10, ring_count=8, radius=0.065, location=(bx, by, bz))
        bulb = bpy.context.active_object
        bulb.name = f"{name}_Bulb_{ib}"
        bulb.data.materials.append(mat_glow_bulb)

for idx, (bx, by) in enumerate([(4.3, 1.8), (4.9, 2.2), (3.6, 2.9)]):
    bz = get_terrain_surface_z(bx, by)
    spawn_bulb_plant(bx, by, bz, name=f"Right_BulbPlant_{idx}")

# ==============================================================================
# 7. EXPORT COMPLETE GLB MODEL
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
