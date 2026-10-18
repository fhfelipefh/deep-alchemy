"""
Masterwork Procedural 3D Cavern Grotto Generator for Deep Alchemy - Phase 2
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

# Bioluminescent Mushroom Materials (HDR Emission > 1.5 for Bloom)
mat_shroom_cap = create_pbr_material("M_Mushroom_Cap_Turquoise", (0.17, 0.91, 0.85), roughness=0.20, emission=(0.17, 0.95, 0.88), emission_strength=3.8) # #2be8d8
mat_shroom_stem = create_pbr_material("M_Mushroom_Stem", (0.80, 0.90, 0.86), roughness=0.55, emission=(0.10, 0.35, 0.30), emission_strength=1.0)
mat_shroom_gills = create_pbr_material("M_Mushroom_Gills", (0.12, 0.70, 0.65), roughness=0.45, emission=(0.15, 0.80, 0.75), emission_strength=2.4)

# Waterfall & Water Materials
mat_waterfall_curtain = create_pbr_material("M_Waterfall_Curtain", (0.20, 0.95, 1.0), roughness=0.10, emission=(0.20, 0.95, 1.0), emission_strength=4.8, alpha=0.95) # #33f2ff
mat_waterfall_foam = create_pbr_material("M_Waterfall_Foam", (1.0, 1.0, 1.0), roughness=0.35, emission=(1.0, 1.0, 1.0), emission_strength=3.5) # #ffffff
mat_god_ray = create_pbr_material("M_God_Ray", (0.40, 0.95, 1.0), roughness=0.1, emission=(0.40, 0.95, 1.0), emission_strength=1.8, alpha=0.30)
mat_foliage_frond = create_pbr_material("M_Foliage_Frond", (0.18, 0.54, 0.24), roughness=0.60)

# ==============================================================================
# 1. CURVED ROCK AMPHITHEATER WALL & CEILING ARCH (FIM DO FUNDO PRETO)
# ==============================================================================
def build_cavern_amphitheater():
    bm = bmesh.new()
    segments_u = 48
    segments_v = 28
    
    # Cavern amphitheater and flanking canyon walls
    center_y = 7.5
    h_top = 18.0
    
    wall_grid = []
    random.seed(101)
    
    for iv in range(segments_v + 1):
        v_frac = iv / segments_v
        curr_z = v_frac * h_top
        
        # Inward doming / vaulted cavern ceiling as it reaches top
        overhang = 0.0
        if curr_z > 10.0:
            overhang = ((curr_z - 10.0) / 8.0) ** 1.6 * 4.2
            
        row = []
        for iu in range(segments_u + 1):
            u_frac = iu / segments_u
            # Wide wrap-around from -58 deg to 238 deg enclosing entire field of view
            angle = -math.pi * 0.32 + u_frac * math.pi * 1.64
            
            # Flanking radius expands forward to form a grand cavern hall
            forward_expansion = max(0.0, -math.sin(angle)) * 3.8
            curr_r = 9.2 + forward_expansion + math.sin(v_frac * math.pi * 2.5) * 0.45 - overhang
            
            x = math.cos(angle) * curr_r
            y = center_y + math.sin(angle) * curr_r
            z = curr_z
            
            # Organic rock facets / block displacement
            block_disp = (math.sin(x * 0.6 + z * 0.4) * math.cos(y * 0.5)) * 0.65
            x += math.cos(angle) * block_disp
            y += math.sin(angle) * block_disp
            
            # Central skylight opening cleft for waterfall drop (around X=0, high Z)
            if abs(x) < 2.4 and z > 14.0 and y > 6.0:
                y += 2.2
                z += 1.4
                
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
            
            # Natural geological strata: central skylight & cleft are lit, shelves have moss, recesses in shadow
            center = face.calc_center_median()
            is_cleft_lit = abs(center.x) < 4.2 and center.z > 5.0 and center.z < 15.0
            is_mid_strata = center.z > 6.0 and center.z < 11.0 and abs(center.x) < 7.5 and (int(center.z) % 3 != 0)
            is_moss_shelf = center.z < 3.2 and (iu % 4 != 0)
            
            if is_moss_shelf:
                face.material_index = 2 # Mossy rock
            elif is_cleft_lit or is_mid_strata:
                face.material_index = 1 # Lit rock
            else:
                face.material_index = 0 # Shadow rock


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
# 2. OVERHEAD ORGANIC STALACTITES FRAMING THE TOP OF THE CAMERA
# ==============================================================================
def spawn_organic_stalactite(x, y, z_top, height=3.2, radius=0.55, tilt=(0.0, 0.0), name="OrganicStalactite"):
    bm = bmesh.new()
    rings = 14
    radials = 16
    
    ring_verts = []
    for ir in range(rings + 1):
        t = ir / rings  # 0.0 at top (root), 1.0 at tip
        # Root flares out quadratically into ceiling rock to eliminate sharp base seam
        root_flare = ((1.0 - t) ** 4.0) * 0.45
        # Mineral bumps and organic variation along length
        mineral_bumps = math.sin(t * math.pi * 5.0) * 0.06 * radius
        curr_r = max(0.02, radius * (1.0 - t * 0.92) + root_flare + mineral_bumps)
        
        # Subtle organic tilt/sway
        curr_x = x + tilt[0] * (t ** 1.3) + math.sin(t * 7.0) * 0.03
        curr_y = y + tilt[1] * (t ** 1.3) + math.cos(t * 7.0) * 0.03
        curr_z = z_top - t * height
        
        row = []
        for ia in range(radials):
            angle = 2.0 * math.pi * ia / radials
            # Subtle radial asymmetry
            r_wobble = curr_r * (1.0 + math.sin(angle * 3.0 + t * 4.0) * 0.07)
            vx = curr_x + math.cos(angle) * r_wobble
            vy = curr_y + math.sin(angle) * r_wobble
            vz = curr_z
            vert = bm.verts.new((vx, vy, vz))
            row.append(vert)
        ring_verts.append(row)
        
    bm.verts.ensure_lookup_table()
    
    # Quad faces along shaft with multi-material blending
    for ir in range(rings):
        t = (ir + 0.5) / rings
        # Material assignment: mossy root at ceiling, lit mineral shaft, wet dark shadow tip
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
        
    # Open top ceiling root (faces facing ceiling to anchor)
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
    
    obj.data.materials.append(mat_rock_shadow)  # idx 0
    obj.data.materials.append(mat_rock_lit)     # idx 1
    obj.data.materials.append(mat_cavern_moss)  # idx 2
    return obj

# Natural irregular stalactite clusters rooted directly in the REAL cavern rock ceiling (Z >= 14.5m)
# Central waterfall corridor (X in [-4.0, 4.0]) is 100% OPEN and UNOBSTRUCTED
organic_stalactites = [
    # Left high cavern ceiling clusters (anchored at top rock wall Z >= 14.8m)
    (-7.2, 5.0, 15.5, 4.2, 0.68, (0.10, -0.05)),
    (-5.5, 6.8, 16.2, 3.8, 0.58, (-0.06, 0.08)),
    (-4.4, 8.5, 16.6, 3.4, 0.52, (0.08, -0.04)),
    (-6.0, 3.6, 14.8, 3.0, 0.46, (0.05, 0.06)),
    (-4.2, 5.2, 15.2, 2.6, 0.42, (-0.08, -0.05)),
    # Right high cavern ceiling clusters (anchored at top rock wall Z >= 14.8m)
    (7.2, 5.0, 15.5, 4.2, 0.68, (-0.10, -0.05)),
    (5.5, 6.8, 16.2, 3.8, 0.58, (0.06, 0.08)),
    (4.4, 8.5, 16.6, 3.4, 0.52, (-0.08, -0.04)),
    (6.0, 3.6, 14.8, 3.0, 0.46, (-0.05, 0.06)),
    (4.2, 5.2, 15.2, 2.6, 0.42, (0.08, -0.05)),
]
for idx, (sx, sy, sz, sh, sr, stilt) in enumerate(organic_stalactites):
    spawn_organic_stalactite(sx, sy, sz, height=sh, radius=sr, tilt=stilt, name=f"CeilingStalactite_{idx}")

# ==============================================================================
# 3. GRAND 3D VOLUMETRIC WATERFALL CURTAIN (CORTINA LARGA 8.6m + MERGULHO PERFEITO NO LAGO)
# ==============================================================================
def build_wide_waterfall_curtain():
    segments_u = 32
    segments_v = 44
    water_thickness = 0.75
    
    mesh = bpy.data.meshes.new("Waterfall_Curtain_Mesh")
    obj = bpy.data.objects.new("Waterfall_Curtain", mesh)
    bpy.context.collection.objects.link(obj)
    
    bm = bmesh.new()
    front_grid = []
    back_grid = []
    
    width_top = 5.2
    width_bottom = 8.6
    h_top = 17.5
    h_bottom = -2.0  # Deeply submerged beneath lakebed (Z = 0.0)
    fall_y = 10.8    # At back wall cleft
    
    for iv in range(segments_v + 1):
        v_frac = iv / segments_v
        curr_z = h_bottom + (1.0 - v_frac) * (h_top - h_bottom)
        
        # Natural arc forward as water accelerates into center of lake (Y = 8.5)
        curr_y = fall_y - math.sin(v_frac * math.pi * 0.5) ** 1.3 * 2.3
        curr_width = width_top + v_frac * (width_bottom - width_top)
        
        row_front = []
        row_back = []
        for iu in range(segments_u + 1):
            u_frac = (iu / segments_u) - 0.5  # [-0.5, 0.5]
            x = u_frac * curr_width
            # Shallow curved curtain arc
            arc_depth = (1.0 - (u_frac * 2.0) ** 2) * 0.45
            # Fluted vertical cascading torrent ridges
            ripples = math.sin(u_frac * math.pi * 10.0) * 0.12
            
            y_f = curr_y - arc_depth + ripples
            y_b = y_f + water_thickness
            z = curr_z
            
            vert_f = bm.verts.new((x, y_f, z))
            vert_b = bm.verts.new((x, y_b, z))
            row_front.append(vert_f)
            row_back.append(vert_b)
            
        front_grid.append(row_front)
        back_grid.append(row_back)
        
    bm.verts.ensure_lookup_table()
    
    # 1. Front faces (facing player / -Y)
    for iv in range(segments_v):
        for iu in range(segments_u):
            v1 = front_grid[iv][iu]
            v2 = front_grid[iv][iu + 1]
            v3 = front_grid[iv + 1][iu + 1]
            v4 = front_grid[iv + 1][iu]
            bm.faces.new((v1, v2, v3, v4))
            
    # 2. Back faces (facing cliff / +Y)
    for iv in range(segments_v):
        for iu in range(segments_u):
            v1 = back_grid[iv][iu + 1]
            v2 = back_grid[iv][iu]
            v3 = back_grid[iv + 1][iu]
            v4 = back_grid[iv + 1][iu + 1]
            bm.faces.new((v1, v2, v3, v4))
            
    # 3. Left side wall (iu = 0)
    for iv in range(segments_v):
        v1 = back_grid[iv][0]
        v2 = front_grid[iv][0]
        v3 = front_grid[iv + 1][0]
        v4 = back_grid[iv + 1][0]
        bm.faces.new((v1, v2, v3, v4))
        
    # 4. Right side wall (iu = segments_u)
    for iv in range(segments_v):
        v1 = front_grid[iv][segments_u]
        v2 = back_grid[iv][segments_u]
        v3 = back_grid[iv + 1][segments_u]
        v4 = front_grid[iv + 1][segments_u]
        bm.faces.new((v1, v2, v3, v4))
        
    # 5. Top cap (iv = 0)
    for iu in range(segments_u):
        v1 = front_grid[0][iu]
        v2 = back_grid[0][iu]
        v3 = back_grid[0][iu + 1]
        v4 = front_grid[0][iu + 1]
        bm.faces.new((v1, v2, v3, v4))
        
    # 6. Bottom submerged cap (iv = segments_v)
    for iu in range(segments_u):
        v1 = front_grid[segments_v][iu + 1]
        v2 = back_grid[segments_v][iu + 1]
        v3 = back_grid[segments_v][iu]
        v4 = front_grid[segments_v][iu]
        bm.faces.new((v1, v2, v3, v4))
        
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    
    # Smooth shading so NO interior Sobel lines are generated inside the water curtain
    for poly in mesh.polygons:
        poly.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_waterfall_curtain)

# Waterfall is spawned dynamically in Bevy with WaterMaterial & custom WGSL toon shader
# build_wide_waterfall_curtain()

# ==============================================================================
# 5. CLIFF WALL MOSS STRATA
# ==============================================================================
# Geological moss strata is integrated directly into Cavern Amphitheater shelves,
# avoiding thin dangling pole cylinders that create Sobel edge artifacts.

# ==============================================================================
# 6. LATERAL BIOLUMINESCENT MUSHROOM CLUSTERS (OUTSIDE X in [-4.0, 4.0])
# ==============================================================================
def build_smooth_parasol_mushroom(root_x, root_y, root_z, scale=1.0, lean_dir=1.0, name="SmoothMushroom"):
    # --- A. Smooth Curved Stem ---
    bm_stem = bmesh.new()
    stem_segments = 16
    radial_segments = 20
    h_stem = 2.4 * scale
    r_base = 0.28 * scale
    
    stem_rows = []
    for ih in range(stem_segments + 1):
        t = ih / stem_segments
        curr_z = root_z + t * h_stem
        # Gentle sinusoidal bend leaning inward into scene
        curr_x = root_x + math.sin(t * math.pi * 0.65) * 0.40 * lean_dir * scale
        curr_y = root_y + (t * 0.15) * scale
        curr_r = r_base * (1.0 - t * 0.48)
        
        row = []
        for ir in range(radial_segments):
            angle = 2.0 * math.pi * ir / radial_segments
            vx = curr_x + math.cos(angle) * curr_r
            vy = curr_y + math.sin(angle) * curr_r
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
    # 48 radial segments, 16 rings for buttery-smooth silhouette
    bm_cap = bmesh.new()
    cap_radial = 48
    cap_rings = 16
    
    cap_rows = []
    for iring in range(cap_rings + 1):
        t_ring = iring / cap_rings
        ring_r = t_ring * r_cap_max
        # Smooth cosine flattened parabolic dome
        dome_drop = (1.0 - math.cos(t_ring * math.pi * 0.5)) * 0.68 * scale
        ring_z = cap_top_z - dome_drop
        
        row = []
        for ir in range(cap_radial):
            angle = 2.0 * math.pi * ir / cap_radial
            # Gentle low-frequency wave on rim only
            rim_wave = 0.0
            if t_ring > 0.70:
                rim_wave = math.sin(angle * 6.0) * 0.04 * ((t_ring - 0.70) / 0.30) * scale
                
            vx = cap_center_x + math.cos(angle) * (ring_r + rim_wave)
            # Tilt slightly forward toward camera
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

# Lateral Groups (Moved completely outside X in [-4.0, 4.0] to frame the edges)
# Left Cluster: Main Large + 2 Companion Small
build_smooth_parasol_mushroom(-5.4, 3.6, 0.1, scale=1.25, lean_dir=1.0, name="Parasol_Left_Large")
build_smooth_parasol_mushroom(-4.5, 2.8, 0.1, scale=0.68, lean_dir=0.8, name="Parasol_Left_Medium")
build_smooth_parasol_mushroom(-5.8, 4.4, 0.1, scale=0.42, lean_dir=0.5, name="Parasol_Left_Small")

# Right Cluster: Main Large + 2 Companion Small
build_smooth_parasol_mushroom(5.4, 3.6, 0.1, scale=1.25, lean_dir=-1.0, name="Parasol_Right_Large")
build_smooth_parasol_mushroom(4.5, 2.8, 0.1, scale=0.68, lean_dir=-0.8, name="Parasol_Right_Medium")
build_smooth_parasol_mushroom(5.8, 4.4, 0.1, scale=0.42, lean_dir=-0.5, name="Parasol_Right_Small")

# Background Cliff Ledge Accent Mushrooms
build_smooth_parasol_mushroom(-7.2, 7.5, 2.5, scale=0.85, lean_dir=0.4, name="Parasol_Cliff_Left")
build_smooth_parasol_mushroom(7.2, 7.5, 2.5, scale=0.85, lean_dir=-0.4, name="Parasol_Cliff_Right")

# ==============================================================================
# 7. WIDE DIRT PATH & CIRCULAR LAKE BASIN (CENTRO 100% DESOBSTRUÍDO)
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
        x = v.co.x
        y = v.co.y
        dist_lake = math.sqrt((x - lake_center_x)**2 + (y - lake_center_y)**2)
        
        # Circular lake basin depression (radius >= 5.0m)
        if dist_lake < lake_radius:
            depth_factor = 1.0 - (dist_lake / lake_radius)
            v.co.z = -0.75 * depth_factor
        elif dist_lake < lake_radius + 2.2:
            # Smooth flat lakeshore surrounding the travertine rim
            v.co.z = 0.05
        elif y < 4.2 and abs(x) < 2.5:
            # Wide flat dirt path in center leading to lake
            v.co.z = 0.04
        else:
            # Flanking emerald grass banks gently rising on the far sides
            dist_side = max(0.0, abs(x) - 4.2)
            bank_h = min(2.2, dist_side * 0.40)
            v.co.z = 0.05 + bank_h + math.sin(x * 0.3) * 0.10

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
        
        # 4 vertices per profile slice: inner_bot, inner_top, outer_top, outer_skirt
        v0 = bm_rim.verts.new((lake_center_x + cos_a * (lake_radius - 0.20), lake_center_y + sin_a * (lake_radius - 0.20), -0.15))
        v1 = bm_rim.verts.new((lake_center_x + cos_a * (lake_radius - 0.05), lake_center_y + sin_a * (lake_radius - 0.05), 0.26))
        v2 = bm_rim.verts.new((lake_center_x + cos_a * (lake_radius + 0.45), lake_center_y + sin_a * (lake_radius + 0.45), 0.24))
        v3 = bm_rim.verts.new((lake_center_x + cos_a * (lake_radius + 0.90), lake_center_y + sin_a * (lake_radius + 0.90), -0.06))
        rim_grid.append((v0, v1, v2, v3))
        
    for ip in range(rim_pts):
        r0 = rim_grid[ip]
        r1 = rim_grid[ip + 1]
        # Inner face facing water
        bm_rim.faces.new((r0[0], r0[1], r1[1], r1[0]))
        # Top curb face (facing up +Z)
        bm_rim.faces.new((r0[1], r0[2], r1[2], r1[1]))
        # Outer face sloping into terrain (+radius, -Z)
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
# 8. SMOOTH FLATTENED RIVER STONES ALONG PATH BORDER
# ==============================================================================
def spawn_river_stone(x, y, z, size_x=0.35, size_y=0.28):
    bpy.ops.mesh.primitive_uv_sphere_add(
        segments=12, ring_count=8, radius=size_x,
        location=(x, y, z)
    )
    stone = bpy.context.active_object
    stone.scale = (1.0, size_y / size_x, 0.42) # Flattened river stone
    bpy.ops.object.transform_apply(scale=True)
    stone.data.materials.append(mat_river_pebble)

stone_coords = [
    # Left path border
    (-2.4, 0.8, 0.08, 0.32, 0.26),
    (-2.5, 1.8, 0.10, 0.38, 0.30),
    (-2.6, 2.8, 0.12, 0.42, 0.34),
    (-2.8, 3.8, 0.14, 0.36, 0.28),
    # Right path border
    (2.4, 0.8, 0.08, 0.32, 0.26),
    (2.5, 1.8, 0.10, 0.38, 0.30),
    (2.6, 2.8, 0.12, 0.42, 0.34),
    (2.8, 3.8, 0.14, 0.36, 0.28),
]
for sx, sy, sz, s_x, s_y in stone_coords:
    spawn_river_stone(sx, sy, sz, s_x, s_y)

# ==============================================================================
# 9. LANCEOLATE CURVED FOLIAGE ROSETTES (FOREGROUND LATERAL FRAMING)
# ==============================================================================
def build_curved_lanceolate_rosette(base_x, base_y, base_z, num_fronds=6, scale=1.0, name="LanceolateRosette"):
    bm = bmesh.new()
    for ifr in range(num_fronds):
        angle = (2.0 * math.pi * ifr / num_fronds) + (random.random() - 0.5) * 0.25
        # Teardrop / lanceolate curved frond
        length = 1.35 * scale
        frond_pts = 8
        frond_rows = []
        for ip in range(frond_pts + 1):
            t = ip / frond_pts
            # Tapered lanceolate width (narrow at base, wide at 0.4, sharp at tip)
            w = math.sin(t * math.pi) * 0.22 * scale
            # Downward droop curve: z -= t * t
            dz = - (t ** 1.8) * 0.45 * scale
            dist_xy = t * length
            
            cx = base_x + math.cos(angle) * dist_xy
            cy = base_y + math.sin(angle) * dist_xy
            cz = base_z + 0.25 * scale + dz
            
            # Left and right frond edge points
            perp_x = -math.sin(angle) * w
            perp_y = math.cos(angle) * w
            
            v_l = bm.verts.new((cx + perp_x, cy + perp_y, cz))
            v_r = bm.verts.new((cx - perp_x, cy - perp_y, cz))
            frond_rows.append((v_l, v_r))
            
        for ip in range(frond_pts):
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

# Placed only in the extreme bottom corners (X <= -4.2 and X >= +4.2)
build_curved_lanceolate_rosette(-4.2, 1.4, 0.1, num_fronds=7, scale=1.1, name="Rosette_Left_Corner")
build_curved_lanceolate_rosette(4.2, 1.4, 0.1, num_fronds=7, scale=1.1, name="Rosette_Right_Corner")

# ==============================================================================
# 10. EXPORT COMPLETE GLB MODEL
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

print(f">>> [BLENDER 4.5 - PHASE 2] Successfully exported Masterwork Cavern Kit to: {output_path}")
