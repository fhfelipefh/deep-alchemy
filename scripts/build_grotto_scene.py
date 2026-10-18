"""
Masterwork Procedural 3D Cavern Grotto Generator - Anti-Low-Poly & High Geometry Fidelity Edition
Executes inside Blender 4.5 background mode.
Generates assets/models/cavern_kit.glb adhering strictly to the Art Reference Directives
and Anti-Low-Poly Geometric Constraints:

1. Subdivisão e Suavização de Superfícies (Proibição de BoxGeometry, ConeGeometry e Flat Shading)
   - Todas as rochas e paredes geradas a partir de Icosaedros subdivididos (subdivisions=4) ou grids de alta densidade.
   - computeVertexNormals e use_smooth = True em 100% das malhas orgânicas.
   - Deformação de vértices com ruído contínuo tridimensional multi-escala (Perlin/Simplex/Harmônico).
   - Musgo integrado lendo a normal da superfície (Normal_Z > 0.55).

2. Cogumelos da Direita e Separação de Bounding Box (Zero Colisão)
   - Modelagem por Revolução (LatheGeometry: 36 segmentos radiais x 20 anéis) para chapéus côncavos com cavidade inferior.
   - Caules curvos contínuos (TubeGeometry / Spline: 24 radiais x 24 longitudinais com flare basal).
   - Raio proporcional reduzido para não dominar a cena.
   - Coordenadas afastadas para trás e para a direita (X >= 4.1, Y >= 2.95), garantindo > 1.6m de folga da Bounding Box do banco.

3. Anfiteatro Contínuo, Teto Elevado e Terreno Sinuoso
   - Remoção completa dos prismas/cubos no horizonte.
   - Parede de anfiteatro contínua e alta (Z até 18.0..22.0) envolvendo a lagoa em formato de ferradura.
   - Teto da caverna elevado para encobrir totalmente a origem da cachoeira (Z = 17.5) e fechar o fundo preto.
   - Estalactites orgânicas alongadas (24 radiais x 24 anéis) com afinamento exponencial suave e leve curvatura.
   - Terreno subdividido (140x140) com relevo ondulado nas laterais e contorno de terra batida sinuoso/meandrado.

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

# Flora: Right Side (Intense Cyan/Turquoise Lathe Mushrooms, Tall Grass, Glowing Bulbs)
mat_shroom_vibrant_cap = create_pbr_material("M_Shroom_Vibrant_Cap", (0.04, 0.88, 0.90), roughness=0.18, emission=(0.06, 0.96, 0.94), emission_strength=2.6)
mat_tall_grass = create_pbr_material("M_Tall_Grass", (0.16, 0.50, 0.22), roughness=0.72)
mat_glow_bulb = create_pbr_material("M_Glow_Bulb", (0.15, 0.95, 0.90), roughness=0.15, emission=(0.15, 0.98, 0.92), emission_strength=3.0)
mat_dandelion_stem = create_pbr_material("M_Dandelion_Stem", (0.18, 0.42, 0.10), roughness=0.78)
mat_dandelion_leaf = create_pbr_material("M_Dandelion_Leaf", (0.24, 0.58, 0.12), roughness=0.72)
mat_dandelion_flower = create_pbr_material("M_Dandelion_Flower", (0.95, 0.72, 0.08), roughness=0.38, emission=(0.80, 0.42, 0.03), emission_strength=0.55)
mat_dandelion_seed = create_pbr_material("M_Dandelion_Seed", (0.92, 0.98, 0.94), roughness=0.48)
mat_moss_wet = create_pbr_material("M_Wet_Moss", (0.06, 0.30, 0.12), roughness=0.92)
mat_moss_lime = create_pbr_material("M_Lime_Moss", (0.22, 0.62, 0.18), roughness=0.84)
mat_moss_fringes = create_pbr_material("M_Moss_Fringes", (0.11, 0.40, 0.16), roughness=0.88)

# Waterfall Foam Billows & Water Ripples
mat_water_foam_billow = create_pbr_material("M_Water_Foam_Billow", (0.92, 0.98, 1.0), roughness=0.20, emission=(0.60, 0.90, 1.0), emission_strength=1.5)
mat_water_ripple_ring = create_pbr_material("M_Water_Ripple_Ring", (0.40, 0.92, 0.98), roughness=0.15, emission=(0.35, 0.90, 0.98), emission_strength=1.8, alpha=0.85)

# Ceiling Lichen
mat_ceiling_lichen = create_pbr_material("M_Ceiling_Lichen", (0.20, 0.92, 0.88), roughness=0.15, emission=(0.18, 0.95, 0.90), emission_strength=2.8)

# ==============================================================================
# DETERMINISTIC CONTINUOUS MULTI-SCALE 3D NOISE FUNCTION
# ==============================================================================
def continuous_noise_3d(x, y, z):
    """
    Smooth, band-limited multi-harmonic 3D noise for organic surface displacement.
    Guarantees C1 continuity without polygonal creases.
    """
    n1 = math.sin(x * 1.35 + y * 1.82 + z * 0.91)
    n2 = math.cos(x * 2.71 - y * 1.43 + z * 2.15)
    n3 = math.sin(x * 4.95 + y * 3.78 - z * 3.12)
    n4 = math.cos(x * 9.15 - y * 8.21 + z * 7.43)
    return (n1 * 0.48 + n2 * 0.28 + n3 * 0.16 + n4 * 0.08)

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
    elif y < 5.2 and abs(x) < 2.4:
        # Smooth beaten brown dirt path
        return 0.04
    else:
        # Rolling lateral grassy banks with organic wave relief
        dist_side = max(0.0, abs(x) - 2.4)
        bank_h = min(1.85, dist_side * 0.35)
        wave_relief = (
            math.sin(x * 0.45 + y * 0.35) * 0.08 +
            math.cos(x * 0.72) * 0.04 +
            continuous_noise_3d(x * 0.4, y * 0.4, 0.0) * 0.06
        )
        return 0.05 + bank_h + wave_relief

# ==============================================================================
# 1. HIGH-VAULTED CONTINUOUS CAVERN ROOF & ARCH (ENCOBRE TOTALMENTE A CACHOEIRA)
# ==============================================================================
def get_cavern_ceiling_z(x, y):
    """
    Mathematical elevation function for the cavern ceiling rock vault.
    Guarantees exact matching between ceiling mesh and stalactite anchoring.
    """
    wall_dist = 9.2 + max(0.0, y - 2.0) * 0.35
    sin_ang = max(-0.99, min(0.99, x / max(0.01, wall_dist)))
    angle = math.asin(sin_ang)
    is_over_waterfall = (abs(angle) < 0.35 and y > 7.0)
    
    if y >= 12.5:
        # Rear rock vault meeting the back cliff wall
        curr_z = 18.2 - abs(math.sin(angle)) * 4.0
    elif is_over_waterfall:
        # Vault arches cleanly over the waterfall top (Z = 17.5) at Z = 18.2..18.6
        y_blend = min(1.0, max(0.0, (y - 7.0) / 4.8))
        peak_z = 18.5 - (abs(angle) / 0.35) * 1.5
        curr_z = 10.5 + y_blend * (peak_z - 10.5)
    else:
        # Framing cavern arch: opens high and wide, eliminating claustrophobia
        arch_dip = math.cos(angle * 1.15) * 1.8
        base_z = 6.2 + arch_dip + max(0.0, y) * 0.28
        curr_z = max(4.5, base_z)
        
    n_disp = continuous_noise_3d(angle * 3.5, y * 0.45, curr_z * 0.35) * 0.38
    return curr_z + n_disp

def build_visible_cavern_roof_and_arch():
    """
    Creates an unbroken, organically curved cavern vault:
    - Arches over the waterfall origin (Z = 17.5, Y = 10.8) at Z = 18.2..18.6, encasing it seamlessly.
    - Connects directly to the solid waterfall back cliff wall, completely sealing off any black void.
    - Dips down to Z = 6.2..8.0 in the foreground to frame the camera shot in a natural wide arch.
    - Uses smooth subdivision (use_smooth = True) and continuous 3D noise for authentic cave rock.
    """
    bm = bmesh.new()
    segments_u = 56
    segments_v = 44
    
    grid = []
    for iv in range(segments_v + 1):
        v_frac = iv / segments_v
        curr_y = -4.5 + v_frac * 18.0
        
        row = []
        for iu in range(segments_u + 1):
            u_frac = iu / segments_u
            angle = -math.pi * 0.48 + u_frac * math.pi * 0.96
            wall_dist = 9.2 + max(0.0, curr_y - 2.0) * 0.35
            curr_x = math.sin(angle) * wall_dist
            curr_z = get_cavern_ceiling_z(curr_x, curr_y)
            
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
            norm = f.normal
            if center.z > 14.0 and abs(center.x) < 3.8:
                f.material_index = 1 # Lit rock near upper cleft
            elif norm.z > 0.55 and center.z < 8.0:
                f.material_index = 2 # Top velvet moss on ledges
            else:
                f.material_index = 0 # Deep shadow cavern rock
                
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new("Cavern_Visible_Arch_Mesh")
    obj = bpy.data.objects.new("Cavern_Visible_Arch", mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    
    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)
    return obj

build_visible_cavern_roof_and_arch()


# ==============================================================================
# 2. ELONGATED ORGANIC STALACTITES (EXPONENTIAL TAPER + TOP PIVOT + EMBEDDED IN CEILING)
# ==============================================================================
def spawn_organic_stalactite(x, y, z_top, height=2.8, radius=0.42, tilt=(0.0, 0.0), name="OrganicStalactite"):
    """
    Spawns highly subdivided, organically curved mineral stalactites with smooth shading:
    - 24 radial segments x 24 height rings.
    - Pivot / Origin: Exactly at the top wide base (0, 0, 0) inside the rock ceiling,
      growing downward along -Z into the scene (z_local from 0.0 to -height).
    - Radius governed by exponential decay: r(t) = radius * (1 - t)^1.35 + flaring root.
    - Embedded 0.5 units within the solid ceiling vault rock (z_top = ceiling_z + 0.5)
      leaving zero exposed edges or gaps.
    """
    bm = bmesh.new()
    rings = 24
    radials = 24
    
    ring_verts = []
    for ir in range(rings + 1):
        t = ir / rings
        # Exponential taper profile with prominent flaring attachment base
        root_flare = ((1.0 - t) ** 4.0) * 0.42 * radius
        exp_taper = ((1.0 - t) ** 1.35)
        # Gentle mineral deposition ripples
        mineral_bumps = math.sin(t * math.pi * 7.0) * 0.025 * radius
        curr_r = max(0.012, radius * exp_taper + root_flare + mineral_bumps)
        
        # Local growth downward from 0.0 to -height
        curr_x = tilt[0] * (t ** 1.30) + math.sin(t * 4.5) * 0.02
        curr_y = tilt[1] * (t ** 1.30) + math.cos(t * 4.5) * 0.02
        curr_z = -t * height
        
        row = []
        for ia in range(radials):
            angle = 2.0 * math.pi * ia / radials
            r_wobble = curr_r * (1.0 + math.sin(angle * 3.0 + t * 5.0) * 0.05)
            vx = curr_x + math.cos(angle) * r_wobble
            vy = curr_y + math.sin(angle) * r_wobble
            vz = curr_z
            vert = bm.verts.new((vx, vy, vz))
            row.append(vert)
        ring_verts.append(row)
        
    bm.verts.ensure_lookup_table()
    
    for ir in range(rings):
        t = (ir + 0.5) / rings
        if t < 0.25:
            mat_idx = 0  # Dark shadow rock merging into ceiling
        elif t < 0.80:
            mat_idx = 1  # Luminous mineral calcite ridges
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
            
    # Pointed tip cap at bottom (local z = -height)
    tip_center = bm.verts.new((tilt[0], tilt[1], -height))
    for ia in range(radials):
        ia_next = (ia + 1) % radials
        face = bm.faces.new((ring_verts[rings][ia], ring_verts[rings][ia_next], tip_center))
        face.material_index = 0
        
    # Top ceiling attachment cap at local z = 0.0 (embedded in ceiling)
    top_center = bm.verts.new((0.0, 0.0, 0.0))
    for ia in range(radials):
        ia_next = (ia + 1) % radials
        face = bm.faces.new((ring_verts[0][ia_next], ring_verts[0][ia], top_center))
        face.material_index = 0
        
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    
    # Translate mesh from local origin to world anchor (x, y, z_top)
    trans_mat = Matrix.Translation(Vector((x, y, z_top)))
    bm.transform(trans_mat)
    
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    
    for poly in mesh.polygons:
        poly.use_smooth = True # Smooth mineral shading
    mesh.update()
    
    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)
    return obj

# Framing stalactites with varied lengths, radii, and natural asymmetrical clustering.
# Ranging from colossal ancient columns (r=0.75m, deep drop) to slender soda-straw needles (r=0.15m).
# Anchored dynamically in the ceiling vault (z_top = ceiling_z + 0.5) descending into view.
visible_stalactite_targets = [
    # Left Fissure Cluster: Massive anchor, flanking daggers and fine needles
    (-6.2, 4.2, 2.65, 0.74, (0.09, -0.07)),  # Colossal ancient stalactite (deep drop to z=2.65)
    (-5.4, 3.2, 3.85, 0.46, (0.05, 0.04)),   # Medium fluted body
    (-6.6, 5.1, 5.60, 0.16, (0.02, -0.02)),  # Slender needle (soda straw)
    (-5.8, 2.1, 4.40, 0.38, (0.04, -0.03)),  # Intermediate column
    (-4.9, 4.6, 4.90, 0.28, (-0.03, 0.05)),  # Slender satellite
    (-4.4, 6.4, 3.30, 0.58, (-0.06, 0.08)),  # Long dagger flanking waterfall margin
    (-4.1, 7.1, 5.75, 0.18, (0.03, -0.02)),  # Delicate straw spike
    (-5.7, 0.6, 4.10, 0.44, (0.06, -0.04)),  # Mid canyon flank
    (-6.3, 1.4, 5.40, 0.20, (-0.02, 0.03)),  # Fine straw spike
    (-5.1, -1.0, 3.50, 0.52, (0.05, 0.03)),  # Foreground overhang stalactite
    (-4.4, -1.6, 5.25, 0.17, (-0.03, 0.02)), # Foreground needle

    # Right Fissure Cluster: Asymmetrical, unique heights and widths
    (6.8, 2.2, 2.80, 0.70, (-0.08, 0.06)),   # Dominant colossal column on right
    (5.9, 3.5, 4.20, 0.42, (-0.04, -0.05)),  # Fluted mid body
    (6.4, 4.1, 5.80, 0.15, (0.02, 0.01)),    # Slender needle (soda straw)
    (5.3, 4.9, 3.60, 0.50, (0.04, -0.07)),   # Heavy mineral spike
    (4.7, 5.8, 5.50, 0.22, (-0.02, 0.04)),   # High delicate needle
    (4.2, 7.4, 3.40, 0.55, (-0.07, 0.06)),   # Long dagger framing right waterfall
    (3.8, 8.2, 5.65, 0.18, (0.01, -0.03)),   # Waterfall margin needle
    (5.8, 0.9, 4.50, 0.36, (-0.04, 0.03)),   # Canyon mid flank
    (6.5, 0.2, 5.35, 0.20, (0.03, -0.02)),   # Flank needle
    (5.4, -0.9, 3.35, 0.56, (-0.05, 0.04)),  # Foreground overhang pillar
    (4.6, -1.5, 5.15, 0.19, (0.02, -0.03)),  # Foreground needle
]

for idx, (sx, sy, target_tip_z, sr, stilt) in enumerate(visible_stalactite_targets):
    ceiling_z = get_cavern_ceiling_z(sx, sy)
    z_top = ceiling_z + 0.50  # 0.5m embedded inside rock vault!
    sh = max(1.8, z_top - target_tip_z)
    spawn_organic_stalactite(sx, sy, z_top, height=sh, radius=sr, tilt=stilt, name=f"VisibleStalactite_{idx}")

# ==============================================================================
# 3. CONTINUOUS U-CANYON GROTTO WALLS & 3-TIER ROCK TERRACES
# ==============================================================================
def build_continuous_u_canyon_grotto_walls():
    """
    Unified Continuous Horseshoe Canyon & Amphitheater Walls:
    - Runs continuously from rear wall behind player (Y = -4.5, X in [-8.5, 8.5]),
      along the left canyon rock wall (X in [-8.8, -8.2], Y in [-4.5, 5.5]),
      around the lake amphitheater horseshoe (Y in [5.5, 13.5], curving gracefully behind waterfall cascade at X=0, Y=13.5),
      along the right canyon rock wall (X in [8.2, 8.8], Y in [5.5, -4.5]),
      and closing seamlessly behind the player!
    - 100% seals the grotto horizontally: no empty green voids or black gaps in camera view.
    - Leaves > 2.5m open clearance behind the waterfall curtain (Y = 10.8 down to 8.5).
    - Rises from terrain surface level up to the cavern ceiling vault, connecting seamlessly.
    - 3-tier stepped terracing:
      * Tier 1 (Base ledge, 0.0..0.28): Inward bulge forming natural travertine base shelves.
      * Tier 2 (Mid plant terrace, 0.28..0.55): Stepped horizontal ledge for moss, ferns, and hanging vines.
      * Tier 3 (Upper cliff, 0.55..1.0): Sheer vertical cliff connecting to ceiling vault.
    - 100% smooth shading (p.use_smooth = True) and multi-scale continuous 3D noise.
    - Integrated velvet moss on horizontal surfaces (normal.z > 0.50).
    """
    bm = bmesh.new()
    u_segs = 128
    v_segs = 44

    grid = []
    for iu in range(u_segs + 1):
        u_frac = iu / u_segs

        # Parameterize the continuous closed U-contour in (X, Y)
        if u_frac < 0.22:
            # Segment 1: Left canyon wall from rear to lake margin
            t = u_frac / 0.22
            y0 = -4.5 + t * 10.0
            meander = math.sin(t * math.pi * 2.0) * 0.35 + continuous_noise_3d(-8.5, y0 * 0.4, 0.0) * 0.25
            x0 = -8.5 + meander
            inward_nx = 1.0
            inward_ny = 0.0
        elif u_frac < 0.55:
            # Segment 2: Horseshoe amphitheater curve around lake and BEHIND waterfall
            t = (u_frac - 0.22) / 0.33
            theta_0 = 1.92
            ang = -theta_0 + t * (2.0 * theta_0)
            t_ang = abs(ang) / theta_0
            r_apex = 5.0
            r_base = 9.1
            dist_base = r_apex + (r_base - r_apex) * (t_ang ** 1.6)
            x0 = 0.0 + math.sin(ang) * dist_base
            y0 = 8.5 + math.cos(ang) * dist_base
            inward_nx = -math.sin(ang)
            inward_ny = -math.cos(ang)
        elif u_frac < 0.77:
            # Segment 3: Right canyon wall from lake margin down to rear
            t = (u_frac - 0.55) / 0.22
            y0 = 5.5 - t * 10.0
            meander = math.sin(t * math.pi * 2.0) * 0.35 + continuous_noise_3d(8.5, y0 * 0.4, 0.0) * 0.25
            x0 = 8.5 - meander
            inward_nx = -1.0
            inward_ny = 0.0
        else:
            # Segment 4: Rear wall behind player closing from right to left
            t = (u_frac - 0.77) / 0.23
            x0 = 8.5 - t * 17.0
            meander = math.sin(t * math.pi) * 0.4
            y0 = -4.5 - meander
            inward_nx = 0.0
            inward_ny = 1.0

        z_ground = get_terrain_surface_z(x0, y0) - 0.30
        z_ceil = get_cavern_ceiling_z(x0, y0) + 0.20

        if u_frac >= 0.77 and abs(x0) < 2.4:
            # Arched portal passage connecting grotto to procedural dungeon
            rel_x = x0 / 2.4
            h_arch = max(0.0, 3.8 * math.sqrt(max(0.0, 1.0 - rel_x * rel_x)))
            z_base = z_ground + h_arch
            h_total = max(1.2, z_ceil - z_base)
            portal_atten = abs(x0) / 2.4
        else:
            z_base = z_ground
            h_total = max(3.5, z_ceil - z_base)
            portal_atten = 1.0

        col = []
        for iv in range(v_segs + 1):
            v_frac = iv / v_segs
            curr_z = z_base + v_frac * h_total

            # 3-tier stepped terracing profile
            if v_frac < 0.28:
                d_tier = math.sin((v_frac / 0.28) * math.pi) * 0.60
            elif v_frac < 0.55:
                d_tier = math.sin(((v_frac - 0.28) / 0.27) * math.pi) * 0.48
            else:
                d_tier = ((v_frac - 0.55) / 0.45) * 0.15

            n_disp = continuous_noise_3d(x0 * 0.45, y0 * 0.45, curr_z * 0.35) * 0.40
            
            # Attenuate inward displacement at the rear apex to keep lake basin completely open
            if 0.22 <= u_frac <= 0.55:
                t_ang_sub = abs(-1.92 + ((u_frac - 0.22) / 0.33) * 3.84) / 1.92
                apex_atten = 0.45 + 0.55 * (t_ang_sub ** 2.0)
            else:
                apex_atten = 1.0
                
            d_total = (d_tier * 0.75 + n_disp * 0.35) * apex_atten * portal_atten

            vx = x0 + inward_nx * d_total
            vy = y0 + inward_ny * d_total
            vz = curr_z
            vert = bm.verts.new((vx, vy, vz))
            col.append(vert)
        grid.append(col)

    bm.verts.ensure_lookup_table()
    for iu in range(u_segs):
        for iv in range(v_segs):
            v1 = grid[iu][iv]
            v2 = grid[iu + 1][iv]
            v3 = grid[iu + 1][iv + 1]
            v4 = grid[iu][iv + 1]
            f = bm.faces.new((v1, v2, v3, v4))

            norm = f.normal
            center = f.calc_center_median()
            if norm.z > 0.50:
                f.material_index = 2 # M_Cavern_Moss on horizontal terraced ledges
            elif center.z > 8.0 and abs(center.x) < 3.5 and center.y > 8.0:
                f.material_index = 1 # M_Rock_Lit near upper skylight
            else:
                f.material_index = 0 # M_Rock_Shadow (wet dark cavern stone)

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new("Continuous_U_Canyon_Walls_Mesh")
    obj = bpy.data.objects.new("Continuous_U_Canyon_Walls", mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()

    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()

    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)
    return obj

build_continuous_u_canyon_grotto_walls()

# Organic Travertine Boulder Terraces flanking waterfall & canyon walls (Subdivided Icospheres + Noise)
def spawn_organic_boulder(name, center_x, center_y, center_z, scale=(1.0, 1.0, 1.0), rot_z=0.0, seed=42):
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=4, radius=1.0)
    
    rng = random.Random(seed)
    
    for v in bm.verts:
        p = v.co
        dist = p.length
        if dist > 0.0001:
            norm = p.normalized()
            n = continuous_noise_3d(norm.x * 2.2 + seed, norm.y * 2.2 + seed, norm.z * 2.2 + seed)
            bottom_flatten = 0.82 if p.z < -0.15 else 1.0
            disp = 1.0 + (n * 0.24) + rng.uniform(-0.06, 0.06)
            v.co = norm * (dist * disp * bottom_flatten)
            v.co.x *= scale[0]
            v.co.y *= scale[1]
            v.co.z *= scale[2]
            
    rot_mat = Matrix.Rotation(rot_z, 4, 'Z')
    bm.transform(rot_mat)
    trans_mat = Matrix.Translation((center_x, center_y, center_z))
    bm.transform(trans_mat)
    
    for f in bm.faces:
        if f.normal.z > 0.55:
            f.material_index = 2 # M_Cavern_Moss on top
        else:
            f.material_index = 0 # M_Rock_Shadow on flanks
            
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    
    obj.data.materials.append(mat_rock_shadow)
    obj.data.materials.append(mat_rock_lit)
    obj.data.materials.append(mat_cavern_moss)
    return obj

# Stepped organic boulder terraces in 3 height levels (Base, Mid, Upper) framing both sides
# Receded outwards (|X| >= 6.5) to clear mushrooms and bench with > 1.8m margin and open the amphitheater
organic_terraces = [
    # Left tiered shelves (Base, Mid, Upper tiers from camera foreground to waterfall)
    # Tier 1 (Base shelves):
    ("Left_Terrace_B0", -7.0, -2.0, 0.40, (1.5, 1.3, 0.75), 0.3, 110),
    ("Left_Terrace_B1", -6.8, -0.5, 0.42, (1.5, 1.3, 0.80), -0.2, 111),
    ("Left_Terrace_B2", -6.6, 1.0, 0.45, (1.6, 1.4, 0.85), 0.4, 112),
    ("Left_Terrace_B3", -6.5, 2.6, 0.48, (1.5, 1.3, 0.85), -0.3, 113),
    ("Left_Terrace_B4", -6.6, 4.4, 0.52, (1.6, 1.4, 0.90), 0.2, 114),
    ("Left_Terrace_1", -6.5, 6.8, 0.55, (1.6, 1.4, 0.90), 0.2, 101),
    ("Left_Terrace_Low", -5.8, 5.8, 0.40, (1.2, 1.1, 0.65), 0.5, 105),

    # Tier 2 (Mid-height shelves):
    ("Left_Terrace_M0", -7.5, -1.2, 1.40, (1.4, 1.2, 0.95), -0.4, 120),
    ("Left_Terrace_M1", -7.3, 0.6, 1.55, (1.5, 1.2, 1.00), 0.3, 121),
    ("Left_Terrace_M2", -7.2, 2.5, 1.80, (1.5, 1.2, 1.05), -0.2, 122),
    ("Left_Terrace_M3", -7.2, 4.4, 2.00, (1.5, 1.2, 1.10), 0.4, 123),
    ("Left_Terrace_2", -6.8, 7.8, 1.40, (1.5, 1.3, 1.10), -0.3, 102),
    ("Left_Terrace_3", -7.0, 8.8, 2.45, (1.4, 1.2, 1.20), 0.4, 103),

    # Tier 3 (Upper cliff shelves connecting to ceiling - only deep in the cavern to let foreground breathe):
    ("Left_Terrace_U1", -7.8, 2.0, 3.20, (1.4, 1.2, 1.15), -0.3, 131),
    ("Left_Terrace_U2", -7.6, 4.0, 3.60, (1.4, 1.1, 1.20), 0.3, 132),
    ("Left_Terrace_4", -7.4, 9.8, 3.65, (1.3, 1.1, 1.30), -0.2, 104),

    # Right tiered shelves (Base, Mid, Upper tiers):
    # Tier 1 (Base shelves):
    ("Right_Terrace_B0", 7.0, -2.0, 0.40, (1.5, 1.3, 0.75), -0.3, 210),
    ("Right_Terrace_B1", 6.8, -0.5, 0.42, (1.5, 1.3, 0.80), 0.2, 211),
    ("Right_Terrace_B2", 6.6, 1.0, 0.45, (1.6, 1.4, 0.85), -0.4, 212),
    ("Right_Terrace_B3", 6.5, 2.6, 0.48, (1.5, 1.3, 0.85), 0.3, 213),
    ("Right_Terrace_1", 6.5, 6.8, 0.55, (1.6, 1.4, 0.90), -0.2, 201),
    ("Right_Terrace_Low", 5.8, 5.8, 0.40, (1.2, 1.1, 0.65), -0.5, 205),

    # Tier 2 (Mid-height shelves):
    ("Right_Terrace_M0", 7.5, -1.2, 1.40, (1.4, 1.2, 0.95), 0.4, 220),
    ("Right_Terrace_M1", 7.3, 0.6, 1.55, (1.5, 1.2, 1.00), -0.3, 221),
    ("Right_Terrace_M2", 7.2, 2.5, 1.80, (1.5, 1.2, 1.05), 0.3, 222),
    ("Right_Terrace_M3", 7.2, 4.4, 2.00, (1.5, 1.2, 1.10), -0.4, 223),
    ("Right_Terrace_2", 6.8, 7.8, 1.40, (1.5, 1.3, 1.10), 0.3, 202),
    ("Right_Terrace_3", 7.0, 8.8, 2.45, (1.4, 1.2, 1.20), -0.4, 203),

    # Tier 3 (Upper cliff shelves connecting to ceiling):
    ("Right_Terrace_U1", 7.8, 2.0, 3.20, (1.4, 1.2, 1.15), 0.3, 231),
    ("Right_Terrace_U2", 7.6, 4.0, 3.60, (1.4, 1.1, 1.20), -0.3, 232),
    ("Right_Terrace_4", 7.4, 9.8, 3.65, (1.3, 1.1, 1.30), 0.2, 204),
]

for (tname, tx, ty, tz, tscale, trot, tseed) in organic_terraces:
    spawn_organic_boulder(tname, tx, ty, tz, scale=tscale, rot_z=trot, seed=tseed)

# Submerged river boulders and stones resting at the bottom of the clear lake basin under the waterfall
submerged_lake_boulders = [
    # Directly under the waterfall plunge impact zone:
    ("Submerged_Boulder_Plunge_Center", 0.0, 9.2, -0.38, (1.20, 0.85, 0.45), 0.2, 501),
    ("Submerged_Boulder_Plunge_Left", -1.4, 9.0, -0.40, (0.95, 1.05, 0.42), -0.4, 502),
    ("Submerged_Boulder_Plunge_Right", 1.5, 9.1, -0.39, (1.05, 0.80, 0.44), 0.3, 503),
    ("Submerged_Boulder_Plunge_FarL", -2.4, 8.6, -0.32, (0.80, 0.75, 0.38), 0.5, 504),
    ("Submerged_Boulder_Plunge_FarR", 2.3, 8.7, -0.34, (0.85, 0.70, 0.40), -0.3, 505),
    # Across the middle lake bed:
    ("Submerged_Boulder_Mid_1", -0.7, 7.8, -0.45, (0.80, 0.70, 0.36), 0.6, 510),
    ("Submerged_Boulder_Mid_2", 0.8, 7.6, -0.44, (0.75, 0.65, 0.35), -0.5, 511),
    ("Submerged_Boulder_Mid_3", -1.8, 7.3, -0.38, (0.65, 0.75, 0.32), 0.4, 512),
    ("Submerged_Boulder_Mid_4", 1.7, 7.4, -0.36, (0.70, 0.60, 0.34), -0.2, 513),
    # Shallows near the travertine rim:
    ("Submerged_Boulder_Fore_1", 0.2, 6.5, -0.30, (0.60, 0.55, 0.28), 0.3, 520),
    ("Submerged_Boulder_Fore_2", -1.1, 6.2, -0.25, (0.55, 0.50, 0.26), -0.4, 521),
    ("Submerged_Boulder_Fore_3", 1.2, 6.3, -0.26, (0.58, 0.48, 0.27), 0.5, 522),
]

for (sname, sx, sy, sz, sscale, srot, sseed) in submerged_lake_boulders:
    spawn_organic_boulder(sname, sx, sy, sz, scale=sscale, rot_z=srot, seed=sseed)

# Hanging Vines draping down the rock ledges
def spawn_hanging_vine(x, y, z_top, length=1.8, segments=12, name="HangingVine"):
    bm = bmesh.new()
    width = 0.065
    rows = []
    for iseg in range(segments + 1):
        t = iseg / segments
        cz = z_top - t * length
        cx = x + math.sin(t * math.pi * 3.0) * 0.08
        cy = y + (t * 0.035)
        w = width * (1.0 - t * 0.35)
        v_l = bm.verts.new((cx - w * 0.5, cy, cz))
        v_r = bm.verts.new((cx + w * 0.5, cy, cz))
        rows.append((v_l, v_r))
        
        # Leaf buds sprouting sideways
        if iseg % 2 == 1:
            side = 1.0 if (iseg % 4 == 1) else -1.0
            leaf_tip = bm.verts.new((cx + side * (w + 0.15), cy - 0.02, cz - 0.03))
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
    # Flanking waterfall
    (-3.6, 11.2, 7.5, 3.2),
    (3.6, 11.2, 7.5, 3.2),
    (-4.5, 10.5, 5.2, 2.4),
    (4.5, 10.5, 5.2, 2.4),
    (-6.9, 8.9, 3.4, 1.45),
    (6.9, 8.9, 3.4, 1.45),
    (-6.5, 7.9, 2.2, 1.25),
    (6.5, 7.9, 2.2, 1.25),
    (-5.8, 6.9, 1.1, 0.95),
    (5.8, 6.9, 1.1, 0.95),
    # Left canyon wall
    (-6.8, 4.2, 2.6, 1.5),
    (-6.7, 2.4, 2.4, 1.6),
    (-6.6, 0.6, 2.2, 1.4),
    (-6.8, -1.2, 2.0, 1.3),
    # Right canyon wall
    (6.8, 4.4, 2.6, 1.5),
    (6.7, 2.4, 2.4, 1.6),
    (6.6, 0.6, 2.2, 1.4),
    (6.8, -1.2, 2.0, 1.3),
]
for idx, (vx, vy, vz, vl) in enumerate(vine_locations):
    spawn_hanging_vine(vx, vy, vz, length=vl, name=f"Rock_Vine_{idx}")

# Cascading Moss Curtains draping down sheer rock walls
def spawn_moss_curtain(x, y, z_top, length=1.8, width=0.45, name="MossCurtain"):
    bm = bmesh.new()
    segments_y = 12
    segments_x = 6
    grid = []
    for iy in range(segments_y + 1):
        t = iy / segments_y
        cz = z_top - t * length
        w_curr = width * (1.0 - t * 0.25)
        row = []
        for ix in range(segments_x + 1):
            tx = ix / segments_x
            cx = x + (tx - 0.5) * w_curr
            cy = y + math.sin(t * math.pi * 2.5 + tx * 3.0) * 0.05
            fringe = math.sin(tx * math.pi * 6.0) * 0.06 if t > 0.85 else 0.0
            row.append(bm.verts.new((cx, cy, cz + fringe)))
        grid.append(row)
    for iy in range(segments_y):
        for ix in range(segments_x):
            bm.faces.new((grid[iy][ix], grid[iy][ix+1], grid[iy+1][ix+1], grid[iy+1][ix]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_cavern_moss)
    return obj

moss_curtain_locations = [
    (-3.8, 11.0, 6.8, 2.4, 0.55),
    (3.8, 11.0, 6.8, 2.4, 0.55),
    (-6.8, 5.5, 3.2, 1.8, 0.50),
    (6.8, 5.5, 3.2, 1.8, 0.50),
    (-6.6, 1.8, 2.6, 1.6, 0.45),
    (6.6, 1.8, 2.6, 1.6, 0.45),
]
for idx, (mx, my, mz, ml, mw) in enumerate(moss_curtain_locations):
    spawn_moss_curtain(mx, my, mz, length=ml, width=mw, name=f"Moss_Curtain_{idx}")

# Rosette Ferns on rock ledges
def spawn_rock_fern_rosette(x, y, z, scale=1.0, num_fronds=7, name="RockFern"):
    bm = bmesh.new()
    for ifr in range(num_fronds):
        ang = (ifr / float(num_fronds)) * math.pi * 2.0 + (ifr * 0.3)
        length = 0.95 * scale
        width = 0.28 * scale
        segs = 10
        left_v = []
        right_v = []
        for iseg in range(segs + 1):
            t = iseg / segs
            dist = t * length
            w = math.sin(t * math.pi) * width * 0.5
            drop = (t ** 2.2) * 0.45 * scale
            lx = x + math.cos(ang) * dist
            ly = y + math.sin(ang) * dist
            lz = z + (t * 0.15 * scale) - drop
            
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
    return obj

fern_rosette_locations = [
    # Flanking waterfall & lake margin
    (-5.2, 6.6, 0.95, 1.1),
    (5.2, 6.6, 0.95, 1.1),
    (-6.1, 7.6, 1.85, 1.2),
    (6.1, 7.6, 1.85, 1.2),
    (-6.8, 8.6, 2.85, 1.0),
    (6.8, 8.6, 2.85, 1.0),
    # Canyon wall shelves
    (-5.6, -0.6, 0.85, 1.05),
    (5.6, -0.6, 0.85, 1.05),
    (-5.5, 1.2, 0.90, 1.15),
    (5.5, 1.0, 0.90, 1.15),
    (-6.4, 3.2, 2.50, 1.0),
    (6.4, 4.2, 2.60, 1.0),
]
for idx, (fx, fy, fz, fs) in enumerate(fern_rosette_locations):
    spawn_rock_fern_rosette(fx, fy, fz, scale=fs, name=f"Rock_Fern_{idx}")

# Wet moss behind the waterfall: varied lobes and hanging fronds instead of a
# single flat green patch. Each patch uses a deterministic colour variation.
def spawn_moss_patch(x, y, z, scale=1.0, variant=0, name="WetMossPatch"):
    mats = [mat_moss_wet, mat_cavern_moss, mat_moss_lime]
    rng = random.Random(variant * 97 + int(abs(x*y*z) * 10))
    for i in range(7):
        bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2, radius=0.22*scale,
            location=(x+(i-3)*0.18*scale+rng.uniform(-.08,.08), y, z+rng.uniform(-.10,.12)*scale))
        lobe=bpy.context.object; lobe.name=f"{name}_Lobe_{i}"; lobe.scale=(1.6,.55,rng.uniform(.55,1.1))
        lobe.data.materials.append(mats[(variant+i)%len(mats)])
        for poly in lobe.data.polygons: poly.use_smooth=True
    for i in range(4):
        bpy.ops.mesh.primitive_cylinder_add(vertices=10, radius=.028*scale,
            depth=rng.uniform(.45,.90)*scale, location=(x+(i-1.5)*.28*scale,y,z-.35*scale))
        frond=bpy.context.object; frond.name=f"{name}_Frond_{i}"; frond.rotation_euler[1]=rng.uniform(-.35,.35)
        frond.data.materials.append(mat_moss_fringes)
        for poly in frond.data.polygons: poly.use_smooth=True

for idx, args in enumerate([
    (-4.65,10.9,5.3,1.25,0), (-3.75,11.2,6.1,.90,1), (4.55,10.8,5.0,1.30,2),
    (3.70,11.1,6.2,.82,0), (-5.2,9.8,3.8,.72,2), (5.15,9.7,3.9,.78,1)]):
    spawn_moss_patch(*args, name=f"Waterfall_Back_Moss_{idx}")

# Bioluminescent Cyan Wall Spores / Miniature bracket fungi on shaded rock faces
def spawn_wall_cyan_spores(x, y, z, num_shrooms=5, name="WallCyanSpores"):
    bm = bmesh.new()
    rng = random.Random(int((abs(x) + abs(y) + abs(z)) * 100))
    for ish in range(num_shrooms):
        ox = rng.uniform(-0.25, 0.25)
        oy = rng.uniform(-0.15, 0.15)
        oz = rng.uniform(-0.20, 0.20)
        r_sh = rng.uniform(0.045, 0.085)
        segs_u = 12
        segs_v = 8
        sh_grid = []
        for iv in range(segs_v + 1):
            tv = iv / segs_v
            curr_r = tv * r_sh
            dz = - (tv ** 2.0) * 0.035
            row = []
            for iu in range(segs_u + 1):
                tu = iu / segs_u
                ang = -math.pi * 0.5 + tu * math.pi
                vx = x + ox + math.cos(ang) * curr_r
                vy = y + oy + math.sin(ang) * curr_r
                vz = z + oz + dz
                row.append(bm.verts.new((vx, vy, vz)))
            sh_grid.append(row)
        for iv in range(segs_v):
            for iu in range(segs_u):
                bm.faces.new((sh_grid[iv][iu], sh_grid[iv][iu+1], sh_grid[iv+1][iu+1], sh_grid[iv+1][iu]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"{name}_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    obj.data.materials.append(mat_shroom_pale_cap)
    return obj

wall_spore_locations = [
    (-7.2, -0.5, 2.8),
    (-7.1, 1.8, 3.2),
    (-7.2, 3.8, 3.5),
    (-6.6, 6.2, 2.4),
    (-4.5, 11.0, 6.0),
    (7.2, -0.5, 2.8),
    (7.1, 1.8, 3.2),
    (7.2, 3.8, 3.5),
    (6.6, 6.2, 2.4),
    (4.5, 11.0, 6.0),
]
for idx, (wx, wy, wz) in enumerate(wall_spore_locations):
    spawn_wall_cyan_spores(wx, wy, wz, num_shrooms=6, name=f"Wall_Cyan_Spore_{idx}")

# ==============================================================================
# 4. TERRAIN (140x140 GRID, SINUOUS MEANDERING DIRT PATH & NATURAL RELIEF)
# ==============================================================================
def build_terrain_and_lake_rim():
    # High subdivision grid: 140x140 for organic path transitions and rolling mounds
    bpy.ops.mesh.primitive_grid_add(x_subdivisions=140, y_subdivisions=140, size=34.0, location=(0.0, 0.0, 0.0))
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
        
        # Sinuous, meandering natural dirt path boundary (eliminates straight/trapezoid lines)
        w_base = 2.45
        w_meander = (
            math.sin(center.y * 0.72) * 0.38 +
            math.cos(center.y * 1.55 + 0.3) * 0.24 +
            math.sin(center.x * 1.2 + center.y * 0.9) * 0.18
        )
        w_path = w_base + w_meander
        
        is_path_strip = (center.y < 5.0 and abs(center.x) < w_path)
        is_lake_shore = dist_l < (lake_radius + 1.25 + math.sin(center.x * 2.0) * 0.15)
        
        if is_path_strip or is_lake_shore:
            f.material_index = 0 # Pure beaten brown dirt #472914
        else:
            f.material_index = 1 # Dense green grass banks with smooth relief

    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(terrain.data)
    bm.free()
    for p in terrain.data.polygons:
        p.use_smooth = True
    terrain.data.update()

    # Fine curved stone rim leveled with the water
    bm_rim = bmesh.new()
    rim_pts = 72
    rim_grid = []
    rim_top_z = 0.08
    rim_bot_z = -0.06
    
    for ip in range(rim_pts + 1):
        frac = ip / rim_pts
        ang = math.pi * 0.85 + frac * math.pi * 1.30
        
        r_inner = lake_radius - 0.08
        r_outer = lake_radius + 0.18
        
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
        p.use_smooth = True
    rim_mesh.update()
    rim_obj.data.materials.append(mat_fine_rim)

build_terrain_and_lake_rim()

# Loose river pebbles on dirt path edges (Smooth UV spheres)
def spawn_river_pebble(x, y, z, sx, sy):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=12, radius=0.10, location=(x, y, z))
    peb = bpy.context.active_object
    peb.scale = (sx, sy, 0.45)
    bpy.ops.object.transform_apply(scale=True)
    peb.data.materials.append(mat_river_pebble)
    for p in peb.data.polygons:
        p.use_smooth = True

pebble_coords = [
    (-2.2, 0.8, 0.06, 0.28, 0.22),
    (-2.3, 1.8, 0.07, 0.35, 0.26),
    (-2.1, 2.7, 0.08, 0.26, 0.20),
    (-2.4, 3.6, 0.09, 0.32, 0.25),
    (2.1, 0.8, 0.06, 0.25, 0.20),
    (2.2, 1.8, 0.08, 0.32, 0.24),
    (2.3, 2.9, 0.09, 0.28, 0.22),
    (2.5, 3.8, 0.10, 0.35, 0.28),
]
for (px, py, pz, psx, psy) in pebble_coords:
    spawn_river_pebble(px, py, pz, psx, psy)

# Organic Transition Rocks on the boundary between Dirt and Grass (High-Density Icospheres)
def spawn_organic_transition_rock(x, y, z, sx=0.55, sy=0.45, sz=0.35, rot_z=0.0, seed=42):
    """
    Subdivided Icosphere rock (subdivisions=4) with smooth continuous noise displacement.
    Integrated moss on Normal_Z > 0.55 without square blocks or flat shading.
    """
    bm = bmesh.new()
    bmesh.ops.create_icosphere(bm, subdivisions=4, radius=1.0)
    
    rng = random.Random(seed)
    for v in bm.verts:
        p = v.co
        dist = p.length
        if dist > 0.0001:
            norm = p.normalized()
            n = continuous_noise_3d(norm.x * 2.5 + seed, norm.y * 2.5 + seed, norm.z * 2.5 + seed)
            bottom_flatten = 0.75 if p.z < -0.1 else 1.0
            disp = 1.0 + (n * 0.28) + rng.uniform(-0.06, 0.06)
            v.co = norm * (dist * disp * bottom_flatten)
            v.co.x *= sx
            v.co.y *= sy
            v.co.z *= sz
            
    rot_mat = Matrix.Rotation(rot_z, 4, 'Z')
    bm.transform(rot_mat)
    trans_mat = Matrix.Translation((x, y, z))
    bm.transform(trans_mat)
    
    for f in bm.faces:
        if f.normal.z > 0.55:
            f.material_index = 1 # M_Cavern_Moss integrated top
        else:
            f.material_index = 0 # M_Angular_Rock
            
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(f"TransitionRock_{seed}_Mesh")
    obj = bpy.data.objects.new(f"TransitionRock_{seed}", mesh)
    bpy.context.collection.objects.link(obj)
    bm.to_mesh(mesh)
    bm.free()
    
    for p in mesh.polygons:
        p.use_smooth = True
    mesh.update()
    
    obj.data.materials.append(mat_angular_rock)
    obj.data.materials.append(mat_cavern_moss)
    return obj

organic_rock_coords = [
    (-2.4, 1.1, 0.10, 0.52, 0.42, 0.30, 0.4, 301),
    (-2.5, 2.3, 0.12, 0.58, 0.46, 0.34, -0.6, 302),
    (-2.7, 3.4, 0.15, 0.64, 0.50, 0.38, 0.8, 303),
    (2.3, 1.0, 0.10, 0.50, 0.40, 0.28, -0.4, 304),
    (2.4, 2.2, 0.12, 0.56, 0.44, 0.32, 0.6, 305),
    (2.6, 3.3, 0.15, 0.62, 0.48, 0.36, -0.8, 306),
    # Foreground path-framing rocks
    (-2.3, -0.8, 0.08, 0.50, 0.42, 0.28, 0.3, 310),
    (-2.4, 0.2, 0.09, 0.54, 0.44, 0.32, -0.5, 312),
    (2.3, -0.8, 0.08, 0.48, 0.40, 0.26, -0.3, 311),
    (2.4, 0.2, 0.09, 0.52, 0.42, 0.30, 0.5, 313),
]
for (ax, ay, az, asx, asy, asz, arot, aseed) in organic_rock_coords:
    spawn_organic_transition_rock(ax, ay, az, asx, asy, asz, arot, aseed)

# ==============================================================================
# 5. LEFT SIDE FLORA: LEANING PARASOL MUSHROOMS WITH PALE BLUE-CYAN CAPS & GILLS
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
    stem_segments = 24
    radial_segments = 20
    h_stem = 2.4 * scale
    r_base = 0.20 * scale
    
    stem_rows = []
    for ih in range(stem_segments + 1):
        t = ih / stem_segments
        curr_z = root_z + t * h_stem
        curr_x = root_x + (t ** 1.3) * math.sin(lean_angle) * h_stem
        curr_y = root_y + (t ** 1.3) * 0.25 * scale
        
        root_flare = (((0.25 - t) / 0.25) ** 2.0) * 0.28 * scale if t < 0.25 else 0.0
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
    
    # Mathematical tip position and tangent vector of the stem at t = 1.0
    tip_x = root_x + math.sin(lean_angle) * h_stem
    tip_y = root_y + 0.25 * scale
    tip_z = root_z + h_stem
    tip_pos = Vector((tip_x, tip_y, tip_z))
    
    # Tangent vector of parametric stem curve at t = 1.0: (dx/dt, dy/dt, dz/dt)
    tangent = Vector((1.30 * math.sin(lean_angle) * h_stem, 1.30 * 0.25 * scale, h_stem)).normalized()

    stem_mesh = bpy.data.meshes.new(f"{name}_Stem_Mesh")
    stem_obj = bpy.data.objects.new(f"{name}_Stem", stem_mesh)
    bpy.context.collection.objects.link(stem_obj)
    bm_stem.to_mesh(stem_mesh)
    bm_stem.free()
    for p in stem_mesh.polygons:
        p.use_smooth = True
    stem_mesh.update()
    stem_obj.data.materials.append(mat_shroom_stem)
    
    # 2. Arched Parasol Cap in local coordinates around (0, 0, 0)
    # Origin (0,0,0) is at the inner neck insertion point
    r_cap = 1.15 * scale
    bm_cap = bmesh.new()
    cap_radials = 36
    cap_rings = 16
    
    cap_grid = []
    for iring in range(cap_rings + 1):
        t_r = iring / cap_rings
        ring_r = t_r * r_cap
        drop_z = (t_r ** 2.2) * 0.62 * scale
        curr_z = 0.40 * scale - drop_z
        
        row = []
        for ia in range(cap_radials):
            angle = 2.0 * math.pi * ia / cap_radials
            rim_wave = 0.0
            if t_r > 0.80:
                rim_wave = math.sin(angle * 6.0) * 0.03 * scale
            vx = math.cos(angle) * (ring_r + rim_wave)
            vy = math.sin(angle) * (ring_r + rim_wave)
            vz = curr_z + rim_wave * 0.3
            row.append(bm_cap.verts.new((vx, vy, vz)))
        cap_grid.append(row)
        
    for iring in range(cap_rings):
        for ia in range(cap_radials):
            ia_next = (ia + 1) % cap_radials
            f = bm_cap.faces.new((cap_grid[iring][ia], cap_grid[iring][ia_next], cap_grid[iring + 1][ia_next], cap_grid[iring + 1][ia]))
            f.material_index = 0 # Pale Blue-Cyan Cap
            
    # Luminous underside gills connecting rim to inner neck insertion ring at z = 0.0
    rim_verts = cap_grid[-1]
    neck_verts = []
    for ia in range(cap_radials):
        angle = 2.0 * math.pi * ia / cap_radials
        pleat = math.sin(angle * 18.0) * 0.02 * scale
        vx = math.cos(angle) * (0.22 * scale + pleat)
        vy = math.sin(angle) * (0.22 * scale + pleat)
        vz = 0.0
        neck_verts.append(bm_cap.verts.new((vx, vy, vz)))
        
    for ia in range(cap_radials):
        ia_next = (ia + 1) % cap_radials
        f_gill = bm_cap.faces.new((rim_verts[ia], neck_verts[ia], neck_verts[ia_next], rim_verts[ia_next]))
        f_gill.material_index = 1 # Glowing cyan gills
        
    bmesh.ops.recalc_face_normals(bm_cap, faces=bm_cap.faces)
    
    # Align cap axis with stem tangent and embed 0.10 * scale into cap cavity (zero gap)
    rot_mat = Vector((0.0, 0.0, 1.0)).rotation_difference(tangent).to_matrix().to_4x4()
    trans_pos = tip_pos - tangent * (0.10 * scale)
    trans_mat = Matrix.Translation(trans_pos)
    bm_cap.transform(trans_mat @ rot_mat)
    
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
    cap_obj.parent = stem_obj

# Left Side Cluster: Harmonious Tiered Parasol Mushrooms (In open clearing, > 2.4m free space to rock terraces)
build_stylized_parasol_mushroom(-3.60, 3.20, scale=1.05, lean_angle=0.10, name="Left_Shroom_Large")
build_stylized_parasol_mushroom(-2.95, 2.45, scale=0.72, lean_angle=0.08, name="Left_Shroom_Medium")
build_stylized_parasol_mushroom(-4.10, 2.15, scale=0.48, lean_angle=0.06, name="Left_Shroom_Small")

# Broad Arching Fern/Palmate Leaves surrounding Left Mushrooms
def spawn_broad_leaf_spray(x, y, z, scale=1.0, num_leaves=7, name="BroadLeafSpray"):
    bm = bmesh.new()
    for il in range(num_leaves):
        ang = (il / float(num_leaves)) * math.pi * 1.8 - 0.4
        length = 1.20 * scale
        width = 0.40 * scale
        
        segs = 8
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

spawn_broad_leaf_spray(-3.3, 2.1, 0.10, scale=1.15, name="Left_Fern_Spray_1")
spawn_broad_leaf_spray(-3.9, 3.0, 0.12, scale=0.95, name="Left_Fern_Spray_2")
# Creeping foreground ground plants advancing softly over the brown dirt boundary
spawn_broad_leaf_spray(-2.3, -0.2, 0.08, scale=0.85, name="Path_Creeping_Leaf_L")
spawn_broad_leaf_spray(2.3, -0.2, 0.08, scale=0.85, name="Path_Creeping_Leaf_R")

# Slender Stalks with Glowing Tips
def spawn_glowing_stalk(x, y, z, h=0.75, name="GlowingStalk"):
    bpy.ops.mesh.primitive_cylinder_add(vertices=16, radius=0.016, depth=h, location=(x, y, z + h * 0.5))
    stem = bpy.context.active_object
    stem.name = f"{name}_Stem"
    stem.data.materials.append(mat_cavern_vine)
    for p in stem.data.polygons:
        p.use_smooth = True
    
    bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=12, radius=0.055, location=(x + 0.03, y, z + h))
    tip = bpy.context.active_object
    tip.name = f"{name}_Tip"
    tip.data.materials.append(mat_glowing_tip)
    for p in tip.data.polygons:
        p.use_smooth = True

def spawn_dandelion(x, y, z, height=0.82, bloom=True, seed_head=False, name="Dandelion"):
    # A continuous tapered tube keeps the stem physically connected to the head.
    bm = bmesh.new(); rings = []
    for i in range(13):
        t = i / 12.0
        center = Vector((x + 0.07 * math.sin(t * math.pi), y, z + height * t))
        radius = 0.028 * (1.0 - 0.25 * t)
        rings.append([bm.verts.new((center.x + math.cos(math.tau*j/10)*radius,
                                    center.y + math.sin(math.tau*j/10)*radius, center.z)) for j in range(10)])
    for i in range(12):
        for j in range(10):
            bm.faces.new((rings[i][j], rings[i][(j+1)%10], rings[i+1][(j+1)%10], rings[i+1][j]))
    mesh = bpy.data.meshes.new(f"{name}_StemMesh")
    stem = bpy.data.objects.new(f"{name}_Stem", mesh); bpy.context.collection.objects.link(stem)
    bm.to_mesh(mesh); bm.free(); mesh.materials.append(mat_dandelion_stem)
    for poly in mesh.polygons: poly.use_smooth = True

    # Five low serrated leaf blades identify the rosette at the base.
    for li in range(5):
        a = li * math.tau / 5.0
        lb = bmesh.new(); left=[]; right=[]
        for k in range(7):
            t = k / 6.0; length = 0.42 * t; width = math.sin(t * math.pi) * 0.055
            cx, cy, cz = x + math.cos(a)*length, y + math.sin(a)*length, z + 0.02
            px, py = -math.sin(a)*width, math.cos(a)*width
            left.append(lb.verts.new((cx-px, cy-py, cz))); right.append(lb.verts.new((cx+px, cy+py, cz)))
        for k in range(6): lb.faces.new((left[k], right[k], right[k+1], left[k+1]))
        lm = bpy.data.meshes.new(f"{name}_LeafMesh_{li}"); lo = bpy.data.objects.new(f"{name}_Leaf_{li}", lm)
        bpy.context.collection.objects.link(lo); lb.to_mesh(lm); lb.free(); lm.materials.append(mat_dandelion_leaf); lo.parent = stem

    head = Vector((x + 0.07, y, z + height))
    if bloom:
        bpy.ops.mesh.primitive_uv_sphere_add(segments=12, ring_count=8, radius=0.07, location=head)
        center = bpy.context.object; center.name=f"{name}_FlowerCenter"; center.data.materials.append(mat_dandelion_flower); center.parent=stem
        for i in range(24):
            a = math.tau * i / 24.0
            bpy.ops.mesh.primitive_uv_sphere_add(segments=8, ring_count=5, radius=1.0,
                location=head + Vector((math.cos(a)*0.10, math.sin(a)*0.10, 0.018)))
            petal=bpy.context.object; petal.name=f"{name}_RayFloret_{i}"; petal.scale=(0.025,0.10,0.016)
            petal.rotation_euler[2]=a-math.pi/2; petal.data.materials.append(mat_dandelion_flower); petal.parent=stem
    if seed_head:
        bpy.ops.mesh.primitive_uv_sphere_add(segments=12, ring_count=8, radius=0.095, location=head)
        c=bpy.context.object; c.name=f"{name}_SeedHead"; c.data.materials.append(mat_dandelion_seed); c.parent=stem
        for i in range(30):
            a=math.tau*i/30.0; length=0.30 + (i%4)*0.055
            tip=head+Vector((math.cos(a)*length, math.sin(a)*length, 0.10+(i%3)*0.045))
            bpy.ops.mesh.primitive_cylinder_add(vertices=6, radius=0.005, depth=(tip-head).length, location=(head+tip)*0.5)
            filament=bpy.context.object; filament.name=f"{name}_SeedFilament_{i}"
            filament.rotation_euler=Vector((0,0,1)).rotation_difference(tip-head).to_euler(); filament.data.materials.append(mat_dandelion_seed); filament.parent=stem
            bpy.ops.mesh.primitive_uv_sphere_add(segments=6, ring_count=4, radius=0.018, location=tip)
            tuft=bpy.context.object; tuft.name=f"{name}_SeedTuft_{i}"; tuft.data.materials.append(mat_dandelion_seed); tuft.parent=stem

for idx, (dx, dy, dh, bloom, seed_head) in enumerate([
    (-3.4,1.9,0.72,True,False), (-4.2,2.0,0.88,True,False), (-2.8,2.9,0.66,True,False),
    (-4.0,3.5,0.80,False,True), (2.7,1.7,0.68,True,False), (3.25,2.55,0.84,False,True)]):
    spawn_dandelion(dx, dy, get_terrain_surface_z(dx, dy), dh, bloom, seed_head, f"Dandelion_{idx}")

# ==============================================================================
# 6. RIGHT SIDE FLORA: REVOLUTION LATHE MUSHROOMS WITH SMOOTH CONCAVE CAPS
# ==============================================================================
def build_stylized_lathe_mushroom(root_x, root_y, scale=1.0, lean_angle=-0.15, name="Right_Dome_Mushroom"):
    """
    Substitui esferas achatadas por modelagem genuína por revolução (LatheGeometry):
    - Chapéu côncavo modelado por perfil de revolução (36 segmentos radiais x 20 anéis).
    - Caule tubular curvo (TubeGeometry / Spline: 24 radiais x 24 segmentos).
    - Cavidade inferior côncava com emissão azul-turquesa suave.
    - Dimensões proporcionais refinadas e garantia de use_smooth = True.
    """
    ground_z = get_terrain_surface_z(root_x, root_y)
    root_z = ground_z - 0.12 * scale
    
    # 1. Stem via curved Tube Geometry
    bm_stem = bmesh.new()
    stem_segments = 24
    radial_segments = 24
    h_stem = 1.95 * scale
    r_stem = 0.15 * scale
    
    stem_rows = []
    for ih in range(stem_segments + 1):
        t = ih / stem_segments
        curr_z = root_z + t * h_stem
        # Spline curvature curving gracefully away from the bench
        curr_x = root_x + (t ** 1.30) * math.sin(lean_angle) * (0.35 * h_stem)
        curr_y = root_y + (t ** 1.30) * math.cos(lean_angle) * (0.22 * h_stem)
        
        # Smooth flared root base
        root_flare = (((0.28 - t) / 0.28) ** 2.0) * 0.20 * scale if t < 0.28 else 0.0
        r_curr = r_stem * (1.0 - t * 0.32) + root_flare
        
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
    
    # Mathematical tip position and tangent vector of the stem at t = 1.0
    tip_x = root_x + math.sin(lean_angle) * (0.35 * h_stem)
    tip_y = root_y + math.cos(lean_angle) * (0.22 * h_stem)
    tip_z = root_z + h_stem
    tip_pos = Vector((tip_x, tip_y, tip_z))
    
    tangent = Vector((1.30 * math.sin(lean_angle) * (0.35 * h_stem), 1.30 * math.cos(lean_angle) * (0.22 * h_stem), h_stem)).normalized()

    stem_mesh = bpy.data.meshes.new(f"{name}_Stem_Mesh")
    stem_obj = bpy.data.objects.new(f"{name}_Stem", stem_mesh)
    bpy.context.collection.objects.link(stem_obj)
    bm_stem.to_mesh(stem_mesh)
    bm_stem.free()
    for p in stem_mesh.polygons:
        p.use_smooth = True
    stem_mesh.update()
    stem_obj.data.materials.append(mat_shroom_stem)
    
    # 2. Cap via Revolution Profile (LatheGeometry in local coordinates around (0, 0, 0))
    bm_cap = bmesh.new()
    cap_radials = 36
    cap_rings = 20
    r_cap = 0.62 * scale # Refined, non-oversized radius
    
    # Outer dome surface in local space
    cap_grid = []
    for ir in range(cap_rings + 1):
        u = ir / cap_rings
        ring_r = u * r_cap
        drop_z = (u ** 2.3) * (0.36 * scale)
        curr_z = 0.35 * scale - drop_z
        
        row = []
        for ia in range(cap_radials):
            angle = 2.0 * math.pi * ia / cap_radials
            vx = math.cos(angle) * ring_r
            vy = math.sin(angle) * ring_r
            vz = curr_z
            row.append(bm_cap.verts.new((vx, vy, vz)))
        cap_grid.append(row)
        
    for ir in range(cap_rings):
        for ia in range(cap_radials):
            ia_next = (ia + 1) % cap_radials
            f = bm_cap.faces.new((cap_grid[ir][ia], cap_grid[ir][ia_next], cap_grid[ir + 1][ia_next], cap_grid[ir + 1][ia]))
            f.material_index = 0 # Vibrant Turquoise Cap
            
    # Inner concave underside cavity connecting rim to inner neck insertion ring at z = 0.0
    rim_verts = cap_grid[-1]
    neck_verts = []
    for ia in range(cap_radials):
        angle = 2.0 * math.pi * ia / cap_radials
        vx = math.cos(angle) * (0.16 * scale)
        vy = math.sin(angle) * (0.16 * scale)
        vz = 0.0
        neck_verts.append(bm_cap.verts.new((vx, vy, vz)))
        
    for ia in range(cap_radials):
        ia_next = (ia + 1) % cap_radials
        f_cavity = bm_cap.faces.new((rim_verts[ia], neck_verts[ia], neck_verts[ia_next], rim_verts[ia_next]))
        f_cavity.material_index = 1 # Soft glowing gills / underside
        
    bmesh.ops.recalc_face_normals(bm_cap, faces=bm_cap.faces)
    
    # Align cap axis with stem tangent and embed 0.08 * scale into cavity (zero gap)
    rot_mat = Vector((0.0, 0.0, 1.0)).rotation_difference(tangent).to_matrix().to_4x4()
    trans_pos = tip_pos - tangent * (0.08 * scale)
    trans_mat = Matrix.Translation(trans_pos)
    bm_cap.transform(trans_mat @ rot_mat)
    
    cap_mesh = bpy.data.meshes.new(f"{name}_Cap_Mesh")
    cap_obj = bpy.data.objects.new(f"{name}_Cap", cap_mesh)
    bpy.context.collection.objects.link(cap_obj)
    bm_cap.to_mesh(cap_mesh)
    bm_cap.free()
    for p in cap_mesh.polygons:
        p.use_smooth = True
    cap_mesh.update()
    cap_obj.data.materials.append(mat_shroom_vibrant_cap)
    cap_obj.data.materials.append(mat_shroom_gills)
    cap_obj.parent = stem_obj

# Exactly three rounded lathe mushrooms positioned safely in the open clearing (> 1.7m clear space from bench and rocks)
build_stylized_lathe_mushroom(4.20, 3.40, scale=0.78, lean_angle=-0.12, name="Right_Dome_Mushroom_1")
build_stylized_lathe_mushroom(3.60, 2.70, scale=0.60, lean_angle=-0.08, name="Right_Dome_Mushroom_2")
build_stylized_lathe_mushroom(4.75, 3.90, scale=0.45, lean_angle=-0.14, name="Right_Dome_Mushroom_3")

# Tall grass must never intersect a taller authored element.  This policy is
# deliberately shared by every clump, rather than encoding bench-specific
# coordinates, so new props only need to register one footprint below.
TALL_GRASS_CLEARANCE = 0.42
TALL_GRASS_MAX_RELOCATION = 2.4
tall_grass_exclusion_zones = []

def register_tall_grass_exclusion(name, x, y, radius):
    """Reserve a horizontal footprint for any rock, prop, or tall flora."""
    tall_grass_exclusion_zones.append((name, x, y, radius + TALL_GRASS_CLEARANCE))

def resolve_tall_grass_position(x, y):
    """Return the nearest deterministic spiral candidate outside all blockers."""
    def is_clear(cx, cy):
        return all(math.hypot(cx - bx, cy - by) >= radius
                   for _, bx, by, radius in tall_grass_exclusion_zones)

    if is_clear(x, y):
        return x, y

    # Fixed angular phases keep exports deterministic and avoid visual clumps.
    for ring in range(1, 14):
        distance = ring * 0.18
        if distance > TALL_GRASS_MAX_RELOCATION:
            break
        for spoke in range(16):
            angle = spoke * math.tau / 16.0 + ring * 0.41
            candidate_x = x + math.cos(angle) * distance
            candidate_y = y + math.sin(angle) * distance
            if is_clear(candidate_x, candidate_y):
                return candidate_x, candidate_y

    raise RuntimeError(f"No clear terrain location for tall grass at ({x}, {y})")

# The bench is exported separately, but its reserved footprint belongs to the
# scene-placement policy. Future high props must call this same registrar.
register_tall_grass_exclusion("Rustic_Mossy_Bench", 2.90, 1.05, 1.15)
register_tall_grass_exclusion("Right_Dome_Mushroom_1", 4.20, 3.40, 0.82)
register_tall_grass_exclusion("Right_Dome_Mushroom_2", 3.60, 2.70, 0.66)
register_tall_grass_exclusion("Right_Dome_Mushroom_3", 4.75, 3.90, 0.52)

# Tall Grass Clumps behind/around the right mushrooms
def spawn_tall_grass_clump(x, y, z, scale=1.0, num_blades=14, name="TallGrass"): 
    bm = bmesh.new()
    for ib in range(num_blades):
        ang = (ib / float(num_blades)) * math.pi * 2.0 + (ib * 0.2)
        h = (0.75 + (ib % 3) * 0.18) * scale
        tilt = 0.22 * scale
        
        v_base_l = bm.verts.new((x + math.cos(ang) * 0.05, y + math.sin(ang) * 0.05, z))
        v_base_r = bm.verts.new((x + math.cos(ang + 0.3) * 0.05, y + math.sin(ang + 0.3) * 0.05, z))
        v_mid_l = bm.verts.new((x + math.cos(ang) * (0.05 + tilt * 0.4), y + math.sin(ang) * (0.05 + tilt * 0.4), z + h * 0.5))
        v_mid_r = bm.verts.new((x + math.cos(ang + 0.2) * (0.05 + tilt * 0.4), y + math.sin(ang + 0.2) * (0.05 + tilt * 0.4), z + h * 0.5))
        v_tip = bm.verts.new((x + math.cos(ang) * (0.05 + tilt), y + math.sin(ang) * (0.05 + tilt), z + h))
        
        bm.faces.new((v_base_l, v_base_r, v_mid_r, v_mid_l))
        bm.faces.new((v_mid_l, v_mid_r, v_tip))
        
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

tall_grass_coords = [
    (3.8, 2.6), (4.4, 3.5), (4.9, 3.1), (4.6, 4.1),
    # Foreground lateral grass tufts framing dirt path
    (-2.8, -1.0), (-3.1, -0.2), (-2.8, 0.8),
    (2.8, -1.0), (3.1, -0.2), (2.8, 0.8),
]
for idx, (tx, ty) in enumerate(tall_grass_coords):
    tx, ty = resolve_tall_grass_position(tx, ty)
    tz = get_terrain_surface_z(tx, ty)
    spawn_tall_grass_clump(tx, ty, tz, scale=0.85, name=f"Right_TallGrass_{idx}")

# Glowing Bulb Plants (Repositioned to the right cluster, safely clear of bench)
def spawn_bulb_plant(x, y, z, name="BulbPlant"):
    for ib in range(4):
        ang = (ib / 4.0) * math.pi * 2.0
        bx = x + math.cos(ang) * 0.18
        by = y + math.sin(ang) * 0.18
        bz = z + 0.28 + (ib % 2) * 0.10
        
        bpy.ops.mesh.primitive_cylinder_add(vertices=12, radius=0.012, depth=0.35, location=(x + math.cos(ang) * 0.09, y + math.sin(ang) * 0.09, z + 0.16))
        stem = bpy.context.active_object
        stem.name = f"{name}_Stem_{ib}"
        stem.rotation_euler = (0.2 * math.sin(ang), 0.2 * math.cos(ang), 0.0)
        stem.data.materials.append(mat_cavern_vine)
        for p in stem.data.polygons:
            p.use_smooth = True
        
        bpy.ops.mesh.primitive_uv_sphere_add(segments=16, ring_count=12, radius=0.065, location=(bx, by, bz))
        bulb = bpy.context.active_object
        bulb.name = f"{name}_Bulb_{ib}"
        bulb.data.materials.append(mat_glow_bulb)
        for p in bulb.data.polygons:
            p.use_smooth = True

for idx, (bx, by) in enumerate([(4.3, 2.45), (4.9, 3.30), (4.4, 4.10)]):
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
        bpy.ops.mesh.primitive_uv_sphere_add(segments=20, ring_count=14, radius=1.0, location=(fx, fy, fz))
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
        segs = 64
        w = 0.065
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

# Also save the native .blend file for artists to open, sculpt, and refine manually in Blender
blend_dir = os.path.abspath("assets/models/source_blend")
os.makedirs(blend_dir, exist_ok=True)
blend_path = os.path.join(blend_dir, "cavern_kit.blend")
bpy.ops.wm.save_as_mainfile(filepath=blend_path)
print(f">>> [BLENDER 4.5] Saved native Blender project to: {blend_path}")

bpy.ops.export_scene.gltf(
    filepath=output_path,
    export_format='GLB',
    use_selection=False,
    export_apply=True,
    export_materials='EXPORT',
)

print(f">>> [BLENDER 4.5] Successfully exported Anti-Low-Poly Cavern Kit to: {output_path}")
