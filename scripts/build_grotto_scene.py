"""
Masterwork Procedural 3D Cavern Grotto Generator for Deep Alchemy - Phase 3
Executes inside Blender 4.5 background mode.

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

# --- Color Palette (Matching Exact Directives & Reference Artwork) ---
mat_rock_shadow = create_pbr_material("M_Rock_Shadow", (0.17, 0.23, 0.29), roughness=0.88)
mat_rock_lit = create_pbr_material("M_Rock_Lit", (0.26, 0.36, 0.40), roughness=0.80)
mat_cavern_moss = create_pbr_material("M_Cavern_Moss", (0.18, 0.43, 0.27), roughness=0.82)
mat_cavern_vine = create_pbr_material("M_Cavern_Vine", (0.30, 0.65, 0.42), roughness=0.75)
mat_ground_path = create_pbr_material("M_Ground_Path", (0.42, 0.30, 0.21), roughness=0.88) # Rich brown #6b4c35
mat_ground_grass = create_pbr_material("M_Ground_Grass", (0.12, 0.48, 0.22), roughness=0.82)
mat_travertine = create_pbr_material("M_Travertine_Rim", (0.78, 0.72, 0.62), roughness=0.55)
mat_river_pebble = create_pbr_material("M_River_Pebble", (0.38, 0.36, 0.34), roughness=0.55)

# Bioluminescent Mushroom & Ceiling Lichen Materials (HDR Emission > 2.0 for Bloom)
mat_shroom_cap = create_pbr_material("M_Mushroom_Cap_Turquoise", (0.17, 0.91, 0.85), roughness=0.20, emission=(0.17, 0.95, 0.88), emission_strength=3.8) # #2be8d8
mat_shroom_stem = create_pbr_material("M_Mushroom_Stem", (0.80, 0.90, 0.86), roughness=0.55, emission=(0.10, 0.35, 0.30), emission_strength=1.0)
mat_shroom_gills = create_pbr_material("M_Mushroom_Gills", (0.12, 0.70, 0.65), roughness=0.45, emission=(0.15, 0.80, 0.75), emission_strength=2.4)
mat_ceiling_lichen = create_pbr_material("M_Ceiling_Lichen", (0.22, 0.95, 0.90), roughness=0.15, emission=(0.20, 0.98, 0.92), emission_strength=4.2)
mat_foliage_frond = create_pbr_material("M_Foliage_Frond", (0.18, 0.54, 0.24), roughness=0.60)

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
    elif dist_lake < lake_radius + 2.2:
        return 0.05
    elif y < 4.2 and abs(x) < 2.5:
        return 0.04
    else:
        dist_side = max(0.0, abs(x) - 4.2)
        bank_h = min(2.2, dist_side * 0.40)
        return 0.05 + bank_h + math.sin(x * 0.3) * 0.10

# ==============================================================================
# 1. FULLY-ENCLOSED VAULTED CAVERN DOME & CEILING (FIM TOTAL DO VAZIO PRETO)
# ==============================================================================
def build_cavern_amphitheater():
    bm = bmesh.new()
    segments_u = 48
    segments_v = 36
    
    # Cavern amphitheater extending all the way into a vaulted ceiling dome
    center_y = 7.5
    h_top = 23.5  # Completely covers the upper field of view
    
    wall_grid = []
    random.seed(101)
    
    for iv in range(segments_v + 1):
        v_frac = iv / segments_v
        curr_z = v_frac * h_top
        
        # Smooth inward vaulted doming starting at Z = 7.5m, converging to 0.4m at apex
        dome_t = 0.0
        if curr_z > 7.5:
            dome_t = (curr_z - 7.5) / (h_top - 7.5)
            
        row = []
        for iu in range(segments_u + 1):
            u_frac = iu / segments_u
            # Wide wrap-around from -58 deg to 238 deg enclosing entire field of view
            angle = -math.pi * 0.32 + u_frac * math.pi * 1.64
            
            # Flanking radius expands forward to form a grand cavern hall
            forward_expansion = max(0.0, -math.sin(angle)) * 4.2
            base_r = 10.4 + forward_expansion + math.sin(v_frac * math.pi * 2.5) * 0.45
            
            # Dome overhang pulls radius in to 0.45m at apex, completely closing the ceiling!
            overhang = (dome_t ** 1.32) * (base_r - 0.45)
            curr_r = max(0.42, base_r - overhang)
            
            x = math.cos(angle) * curr_r
            y = center_y + math.sin(angle) * curr_r
            z = curr_z
            
            # Organic rock facets / block displacement
            block_disp = (math.sin(x * 0.5 + z * 0.35) * math.cos(y * 0.45)) * 0.70
            x += math.cos(angle) * block_disp
            y += math.sin(angle) * block_disp
            
            # Central cleft opening for waterfall source drop (Z in [14.0, 19.5])
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

    # Seal the top apex cap of the ceiling dome
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
# 2. OVERHEAD ORGANIC STALACTITES & CEILING BIOLUMINESCENT NODES
# ==============================================================================
def spawn_organic_stalactite(x, y, z_top, height=3.2, radius=0.55, tilt=(0.0, 0.0), name="OrganicStalactite"):
    bm = bmesh.new()
    rings = 14
    radials = 16
    
    ring_verts = []
    for ir in range(rings + 1):
        t = ir / rings  # 0.0 at top (root), 1.0 at tip
        root_flare = ((1.0 - t) ** 4.0) * 0.45
        mineral_bumps = math.sin(t * math.pi * 5.0) * 0.06 * radius
        curr_r = max(0.02, radius * (1.0 - t * 0.92) + root_flare + mineral_bumps)
        
        curr_x = x + tilt[0] * (t ** 1.3) + math.sin(t * 7.0) * 0.03
        curr_y = y + tilt[1] * (t ** 1.3) + math.cos(t * 7.0) * 0.03
        curr_z = z_top - t * height
        
        row = []
        for ia in range(radials):
            angle = 2.0 * math.pi * ia / radials
            r_wobble = curr_r * (1.0 + math.sin(angle * 3.0 + t * 4.0) * 0.07)
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
            
    # Closed tip cap
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

def spawn_ceiling_glow_cluster(x, y, z, size=0.45, name="CeilingLichen"):
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=2, radius=size)
    random.seed(int(abs(x * 100 + z * 10)))
    for v in bm.verts:
        v.co.x += (random.random() - 0.5) * size * 0.4
        v.co.y += (random.random() - 0.5) * size * 0.4
        v.co.z += (random.random() - 0.5) * size * 0.3
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for poly in mesh.polygons:
        poly.use_smooth = True
    mesh.update()
    obj.location = (x, y, z)
    obj.data.materials.append(mat_ceiling_lichen)
    return obj

# Stalactites anchored directly in the vaulted cavern ceiling (Z >= 16.5m)
organic_stalactites = [
    # Left ceiling vault clusters
    (-6.5, 4.5, 17.5, 4.2, 0.65, (0.08, -0.05)),
    (-5.2, 6.2, 19.2, 3.8, 0.55, (-0.05, 0.06)),
    (-3.8, 7.8, 20.5, 3.5, 0.50, (0.06, -0.04)),
    (-5.8, 3.2, 16.8, 3.0, 0.45, (0.04, 0.05)),
    (-3.5, 4.8, 18.2, 2.8, 0.40, (-0.06, -0.04)),
    (-2.2, 6.5, 21.0, 3.2, 0.42, (0.05, 0.04)),
    # Right ceiling vault clusters
    (6.5, 4.5, 17.5, 4.2, 0.65, (-0.08, -0.05)),
    (5.2, 6.2, 19.2, 3.8, 0.55, (0.05, 0.06)),
    (3.8, 7.8, 20.5, 3.5, 0.50, (-0.06, -0.04)),
    (5.8, 3.2, 16.8, 3.0, 0.45, (-0.04, 0.05)),
    (3.5, 4.8, 18.2, 2.8, 0.40, (0.06, -0.04)),
    (2.2, 6.5, 21.0, 3.2, 0.42, (-0.05, 0.04)),
    # Overhead forward arch framing
    (-4.0, 1.8, 17.5, 2.6, 0.38, (0.05, 0.06)),
    (4.0, 1.8, 17.5, 2.6, 0.38, (-0.05, 0.06)),
]
for idx, (sx, sy, sz, sh, sr, stilt) in enumerate(organic_stalactites):
    spawn_organic_stalactite(sx, sy, sz, height=sh, radius=sr, tilt=stilt, name=f"CeilingStalactite_{idx}")

# Bioluminescent turquoise nodes dotted across the high ceiling vault
ceiling_glow_nodes = [
    (-5.5, 5.0, 18.5, 0.42),
    (-3.6, 7.2, 20.8, 0.38),
    (-2.0, 5.8, 21.5, 0.45),
    (5.5, 5.0, 18.5, 0.42),
    (3.6, 7.2, 20.8, 0.38),
    (2.0, 5.8, 21.5, 0.45),
    (-4.2, 2.5, 17.8, 0.35),
    (4.2, 2.5, 17.8, 0.35),
]
for idx, (gx, gy, gz, gs) in enumerate(ceiling_glow_nodes):
    spawn_ceiling_glow_cluster(gx, gy, gz, size=gs, name=f"CeilingLichen_{idx}")

# ==============================================================================
# 3. ORGANIC ROCK TERRACE FORMATIONS FLANKING THE WATERFALL (NO MARIO PIPES)
# ==============================================================================
def build_organic_rock_terraces():
    # Left and right tiered rock shelf formations behind the lake
    bm = bmesh.new()
    
    slabs = [
        # Left stepped rock slabs
        (-6.2, 7.2, 0.05, 1.10, 2.6, 2.2),
        (-6.8, 8.4, 0.85, 2.15, 2.4, 2.0),
        (-7.2, 9.2, 1.80, 3.40, 2.2, 1.8),
        # Right stepped rock slabs
        (6.2, 7.2, 0.05, 1.10, 2.6, 2.2),
        (6.8, 8.4, 0.85, 2.15, 2.4, 2.0),
        (7.2, 9.2, 1.80, 3.40, 2.2, 1.8),
    ]
    
    for (cx, cy, zb, zt, sx, sy) in slabs:
        num_corners = 6
        bot_verts = []
        top_verts = []
        for ic in range(num_corners):
            ang = 2.0 * math.pi * ic / num_corners
            rad_x = (sx * 0.5) * (0.85 + math.sin(ic * 2.3) * 0.15)
            rad_y = (sy * 0.5) * (0.85 + math.cos(ic * 1.7) * 0.15)
            vx = cx + math.cos(ang) * rad_x
            vy = cy + math.sin(ang) * rad_y
            bot_verts.append(bm.verts.new((vx, vy, zb)))
            # Chamfer top inward for natural chiseled bevel
            top_verts.append(bm.verts.new((vx * 0.93 + cx * 0.07, vy * 0.93 + cy * 0.07, zt)))
            
        for ic in range(num_corners):
            ic_next = (ic + 1) % num_corners
            f = bm.faces.new((bot_verts[ic], bot_verts[ic_next], top_verts[ic_next], top_verts[ic]))
            f.material_index = 1 # Lit rock
            
        f_top = bm.faces.new(top_verts)
        f_top.material_index = 2 # Cavern moss
        
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new("Organic_Rock_Terraces_Mesh")
    obj = bpy.data.objects.new("Organic_Rock_Terraces", mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for poly in mesh.polygons:
        poly.use_smooth = False # Crisp chiseled rock facets!
    mesh.update()
    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)

build_organic_rock_terraces()

# ==============================================================================
# 4. LATERAL BIOLUMINESCENT MUSHROOMS WITH FLARED ROOTS SINKING INTO GROUND
# ==============================================================================
def build_smooth_parasol_mushroom(root_x, root_y, scale=1.0, lean_dir=1.0, name="SmoothMushroom"):
    # Precise analytical terrain height at root position
    ground_z = get_terrain_surface_z(root_x, root_y)
    # Sink base into soil so it is 100% solidly rooted with zero gap
    root_z = ground_z - 0.22 * scale
    
    # --- A. Smooth Curved Stem with Flared Root Buttress ---
    bm_stem = bmesh.new()
    stem_segments = 18
    radial_segments = 20
    h_stem = 2.4 * scale
    r_base = 0.32 * scale
    
    stem_rows = []
    for ih in range(stem_segments + 1):
        t = ih / stem_segments
        curr_z = root_z + t * h_stem
        # Gentle sinusoidal bend leaning inward into scene
        curr_x = root_x + math.sin(t * math.pi * 0.65) * 0.40 * lean_dir * scale
        curr_y = root_y + (t * 0.15) * scale
        
        # Flared organic root buttress spreading into soil in the bottom 30% of the stem
        root_flare = 0.0
        flare_factor = 0.0
        if t < 0.30:
            flare_factor = ((0.30 - t) / 0.30) ** 1.8
            root_flare = flare_factor * 0.50 * scale
            
        row = []
        for ir in range(radial_segments):
            angle = 2.0 * math.pi * ir / radial_segments
            root_toes = math.cos(angle * 5.0) * 0.16 * flare_factor * scale if flare_factor > 0.0 else 0.0
            effective_r = r_base * (1.0 - t * 0.48) + root_flare + root_toes
            
            vx = curr_x + math.cos(angle) * effective_r
            vy = curr_y + math.sin(angle) * effective_r
            vz = curr_z
            vert = bm_stem.verts.new((vx, vy, vz))
            row.append(vert)
        stem_rows.append(row)
        
    for ih in range(stem_segments):
        for ir in range(radial_segments):
            ir_next = (ir + 1) % radial_segments
            v1 = stem_rows[ih][ir]
            v2 = stem_rows[ih][ir_next]
            v3 = stem_rows[ih + 1][ir_next]
            v4 = stem_rows[ih + 1][ir]
            bm_stem.faces.new((v1, v2, v3, v4))
            
    # Cap bottom ring
    bottom_center = bm_stem.verts.new((root_x, root_y, root_z))
    for ir in range(radial_segments):
        ir_next = (ir + 1) % radial_segments
        bm_stem.faces.new((stem_rows[0][ir_next], stem_rows[0][ir], bottom_center))

    bmesh.ops.recalc_face_normals(bm_stem, faces=bm_stem.faces)
    cap_top_z = stem_rows[-1][0].co.z + 0.65 * scale
    cap_center_x = stem_rows[-1][0].co.x
    cap_center_y = stem_rows[-1][0].co.y
    r_cap_max = 1.65 * scale

    stem_mesh = bpy.data.meshes.new(f"{name}_Stem_Mesh")
    stem_obj = bpy.data.objects.new(f"{name}_Stem", stem_mesh)
    bpy.context.collection.objects.link(stem_obj)
    bm_stem.to_mesh(stem_mesh)
    bm_stem.free()
    for p in stem_mesh.polygons:
        p.use_smooth = True
    stem_mesh.update()
    stem_obj.data.materials.append(mat_shroom_stem)

    # --- B. Smooth Parasol Cap (Cúpula Parabólica Achatada y = cos(r * pi/2)) ---
    bm_cap = bmesh.new()
    cap_radial = 48
    cap_rings = 16
    
    cap_rows = []
    for iring in range(cap_rings + 1):
        t_ring = iring / cap_rings
        ring_r = t_ring * r_cap_max
        dome_drop = (1.0 - math.cos(t_ring * math.pi * 0.5)) * 0.68 * scale
        ring_z = cap_top_z - dome_drop
        
        row = []
        for ir in range(cap_radial):
            angle = 2.0 * math.pi * ir / cap_radial
            rim_wave = 0.0
            if t_ring > 0.70:
                rim_wave = math.sin(angle * 6.0) * 0.04 * ((t_ring - 0.70) / 0.30) * scale
                
            vx = cap_center_x + math.cos(angle) * (ring_r + rim_wave)
            vy = cap_center_y + math.sin(angle) * (ring_r + rim_wave) - (t_ring * 0.22 * scale)
            vz = ring_z + rim_wave * 0.5
            vert = bm_cap.verts.new((vx, vy, vz))
            row.append(vert)
        cap_rows.append(row)
        
    for iring in range(cap_rings):
        for ir in range(cap_radial):
            ir_next = (ir + 1) % cap_radial
            v1 = cap_rows[iring][ir]
            v2 = cap_rows[iring][ir_next]
            v3 = cap_rows[iring + 1][ir_next]
            v4 = cap_rows[iring + 1][ir]
            face = bm_cap.faces.new((v1, v4, v3, v2))
            face.material_index = 0
            
    # --- C. Underside Smooth Membrane ---
    rim_row = cap_rows[-1]
    neck_center_z = cap_top_z - 0.58 * scale
    neck_r = 0.28 * scale
    inner_verts = []
    for ir in range(cap_radial):
        angle = 2.0 * math.pi * ir / cap_radial
        vx = cap_center_x + math.cos(angle) * neck_r
        vy = cap_center_y + math.sin(angle) * neck_r
        vz = neck_center_z
        inner_verts.append(bm_cap.verts.new((vx, vy, vz)))
        
    for ir in range(cap_radial):
        ir_next = (ir + 1) % cap_radial
        f_under = bm_cap.faces.new((rim_row[ir], inner_verts[ir], inner_verts[ir_next], rim_row[ir_next]))
        f_under.material_index = 1
        
    bmesh.ops.recalc_face_normals(bm_cap, faces=bm_cap.faces)
    cap_mesh = bpy.data.meshes.new(f"{name}_Cap_Mesh")
    cap_obj = bpy.data.objects.new(f"{name}_Cap", cap_mesh)
    bpy.context.collection.objects.link(cap_obj)
    bm_cap.to_mesh(cap_mesh)
    bm_cap.free()
    for p in cap_mesh.polygons:
        p.use_smooth = True
    cap_mesh.update()
    cap_obj.data.materials.append(mat_shroom_cap)
    cap_obj.data.materials.append(mat_shroom_gills)

# Lateral Groups - Cleanly framing the left and right margins, 100% rooted in terrain
# Left Cluster: Main Large + 2 Companion Small
build_smooth_parasol_mushroom(-5.0, 3.2, scale=1.25, lean_dir=0.9, name="Parasol_Left_Large")
build_smooth_parasol_mushroom(-4.3, 2.4, scale=0.72, lean_dir=0.7, name="Parasol_Left_Medium")
build_smooth_parasol_mushroom(-5.5, 3.8, scale=0.45, lean_dir=0.5, name="Parasol_Left_Small")

# Right Cluster: Main Large + 2 Companion Small
build_smooth_parasol_mushroom(5.0, 3.2, scale=1.25, lean_dir=-0.9, name="Parasol_Right_Large")
build_smooth_parasol_mushroom(4.3, 2.4, scale=0.72, lean_dir=-0.7, name="Parasol_Right_Medium")
build_smooth_parasol_mushroom(5.5, 3.8, scale=0.45, lean_dir=-0.5, name="Parasol_Right_Small")

# ==============================================================================
# 5. WIDE DIRT PATH & CIRCULAR LAKE BASIN (CENTRO 100% DESOBSTRUÍDO)
# ==============================================================================
def build_terrain_and_lake_basin():
    bpy.ops.mesh.primitive_grid_add(x_subdivisions=50, y_subdivisions=50, size=34.0, location=(0.0, 0.0, 0.0))
    terrain = bpy.context.active_object
    terrain.name = "Cavern_Terrain_Grid"
    terrain.data.materials.append(mat_ground_path)
    terrain.data.materials.append(mat_ground_grass)

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
        if dist_l < lake_radius + 1.6 or (center.y < 4.2 and abs(center.x) < 2.5):
            f.material_index = 0 # Path, shore & lakebed rich brown earth
        else:
            f.material_index = 1 # Emerald grass banks

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(terrain.data)
    bm.free()
    for p in terrain.data.polygons:
        p.use_smooth = True
    terrain.data.update()

    # Solid 3D Travertine Stone Rim around Lake (radius = 5.2m)
    bm_rim = bmesh.new()
    rim_pts = 48
    rim_grid = []
    for ip in range(rim_pts + 1):
        ang = 2.0 * math.pi * ip / rim_pts
        cos_a = math.cos(ang)
        sin_a = math.sin(ang)
        
        v0 = bm_rim.verts.new((lake_center_x + cos_a * (lake_radius - 0.20), lake_center_y + sin_a * (lake_radius - 0.20), -0.15))
        v1 = bm_rim.verts.new((lake_center_x + cos_a * (lake_radius - 0.05), lake_center_y + sin_a * (lake_radius - 0.05), 0.26))
        v2 = bm_rim.verts.new((lake_center_x + cos_a * (lake_radius + 0.45), lake_center_y + sin_a * (lake_radius + 0.45), 0.24))
        v3 = bm_rim.verts.new((lake_center_x + cos_a * (lake_radius + 0.90), lake_center_y + sin_a * (lake_radius + 0.90), -0.06))
        rim_grid.append((v0, v1, v2, v3))
        
    for ip in range(rim_pts):
        r0 = rim_grid[ip]
        r1 = rim_grid[ip + 1]
        bm_rim.faces.new((r0[0], r0[1], r1[1], r1[0]))
        bm_rim.faces.new((r0[1], r0[2], r1[2], r1[1]))
        bm_rim.faces.new((r0[2], r0[3], r1[3], r1[2]))
        
    bmesh.ops.recalc_face_normals(bm_rim, faces=bm_rim.faces)
    rim_mesh = bpy.data.meshes.new("Lake_Travertine_Rim_Mesh")
    rim_obj = bpy.data.objects.new("Lake_Travertine_Rim", rim_mesh)
    bpy.context.collection.objects.link(rim_obj)
    bm_rim.to_mesh(rim_mesh)
    bm_rim.free()
    for p in rim_mesh.polygons:
        p.use_smooth = True
    rim_mesh.update()
    rim_obj.data.materials.append(mat_travertine)

build_terrain_and_lake_basin()

# ==============================================================================
# 6. SMOOTH FLATTENED RIVER STONES ALONG PATH BORDER
# ==============================================================================
def spawn_river_stone(x, y, z, size_x=0.35, size_y=0.28):
    bpy.ops.mesh.primitive_uv_sphere_add(
        segments=12, ring_count=8, radius=size_x,
        location=(x, y, z)
    )
    stone = bpy.context.active_object
    stone.scale = (1.0, size_y / size_x, 0.42)
    bpy.ops.object.transform_apply(scale=True)
    stone.data.materials.append(mat_river_pebble)
    for p in stone.data.polygons:
        p.use_smooth = True

river_pebble_coords = [
    # Left path edge
    (-2.2, 1.2, 0.08, 0.42, 0.35),
    (-2.4, 2.2, 0.10, 0.36, 0.30),
    (-2.8, 3.4, 0.12, 0.48, 0.38),
    (-3.6, 4.6, 0.14, 0.52, 0.42),
    # Right path edge
    (2.2, 1.2, 0.08, 0.42, 0.35),
    (2.4, 2.2, 0.10, 0.36, 0.30),
    (2.8, 3.4, 0.12, 0.48, 0.38),
    (3.6, 4.6, 0.14, 0.52, 0.42),
]
for (px, py, pz, psx, psy) in river_pebble_coords:
    spawn_river_stone(px, py, pz, psx, psy)

# ==============================================================================
# 7. LANCEOLATE CURVED FOLIAGE ROSETTES (FOREGROUND LATERAL FRAMING)
# ==============================================================================
def build_curved_lanceolate_rosette(base_x, base_y, base_z, num_fronds=6, scale=1.0, name="LanceolateRosette"):
    bm = bmesh.new()
    frond_segments = 10
    
    for ifr in range(num_fronds):
        angle_rad = (2.0 * math.pi * ifr / num_fronds) + (ifr * 0.15)
        frond_len = (1.2 + (ifr % 3) * 0.25) * scale
        frond_width = 0.22 * scale
        
        frond_rows = []
        for iseg in range(frond_segments + 1):
            t = iseg / frond_segments
            rad_dist = t * frond_len
            arc_drop = (t ** 2.2) * 0.45 * scale
            w = math.sin(t * math.pi) * frond_width
            
            cx = base_x + math.cos(angle_rad) * rad_dist
            cy = base_y + math.sin(angle_rad) * rad_dist
            cz = base_z + 0.15 + (t * 0.35 * scale) - arc_drop
            
            perp_x = -math.sin(angle_rad) * (w * 0.5)
            perp_y = math.cos(angle_rad) * (w * 0.5)
            
            v_left = bm.verts.new((cx - perp_x, cy - perp_y, cz))
            v_right = bm.verts.new((cx + perp_x, cy + perp_y, cz))
            frond_rows.append((v_left, v_right))
            
        for ip in range(frond_segments):
            v1 = frond_rows[ip][0]
            v2 = frond_rows[ip + 1][0]
            v3 = frond_rows[ip + 1][1]
            v4 = frond_rows[ip][1]
            bm.faces.new((v1, v2, v3, v4))
            
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_foliage_frond)

build_curved_lanceolate_rosette(-4.2, 1.4, 0.1, num_fronds=7, scale=1.1, name="Rosette_Left_Corner")
build_curved_lanceolate_rosette(4.2, 1.4, 0.1, num_fronds=7, scale=1.1, name="Rosette_Right_Corner")

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

print(f">>> [BLENDER 4.5 - PHASE 3] Successfully exported Masterwork Cavern Kit to: {output_path}")
