"""
Procedural Faceted Rock Generator for Deep Alchemy (Blender 4.5+ Headless)
Follows Technical Artist Concept-Art-Fidelity Protocol:
1. Strict prohibition of pure Cuboid / Box / Cylinder primitives for natural rocks.
2. Generates angular rocks from subdivided icosahedrons displaced with 3D noise.
3. Sets flat shading (poly.use_smooth = False) to give crisp cel-shaded faceting.
4. Assigns top-surface moss (M_RockMoss #246e34) when Normal_Z > 0.6, preserving
   dark rock (M_RockBase #26333a) on vertical and underside faces.
5. Exports modular rock kit to assets/models/faceted_rocks_kit.glb.
"""
import bpy
import bmesh
import math
import random
import os
from mathutils import Vector, Matrix, Euler

def reset_scene():
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj, do_unlink=True)
    for mesh in list(bpy.data.meshes):
        bpy.data.meshes.remove(mesh)
    for mat in list(bpy.data.materials):
        bpy.data.materials.remove(mat)

def create_pbr_material(name, base_color, roughness=0.85, metallic=0.0):
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
    return mat

def get_rock_materials():
    # Palette extracted from Concept Art Blueprint
    # Dark rock base (#26333a) -> sRGB ~ (0.15, 0.20, 0.23)
    mat_rock = bpy.data.materials.get("M_FacetedRock_Base")
    if not mat_rock:
        mat_rock = create_pbr_material("M_FacetedRock_Base", (0.15, 0.20, 0.23), roughness=0.88)
        
    # Lush velvet top moss (#246e34) -> sRGB ~ (0.14, 0.43, 0.20)
    mat_moss = bpy.data.materials.get("M_FacetedRock_Moss")
    if not mat_moss:
        mat_moss = create_pbr_material("M_FacetedRock_Moss", (0.14, 0.43, 0.20), roughness=0.92)
        
    return mat_rock, mat_moss

def noise3d_coord(x, y, z):
    # Deterministic multi-harmonic pseudo-noise
    n1 = math.sin(x * 1.7 + y * 2.3 + z * 1.1)
    n2 = math.cos(x * 3.1 - y * 1.4 + z * 2.7)
    n3 = math.sin(x * 5.2 + y * 4.1 - z * 3.3)
    return (n1 * 0.5 + n2 * 0.35 + n3 * 0.15)

def generate_angular_rock(name, radius=1.0, scale=(1.0, 1.0, 1.0), seed=42, loc=(0.0, 0.0, 0.0), rot=(0.0, 0.0, 0.0)):
    """
    Creates an angular faceted rock mesh via subdivided icosahedron + noise displacement.
    """
    mat_rock, mat_moss = get_rock_materials()
    
    mesh = bpy.data.meshes.new(name + "_Mesh")
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    
    bm = bmesh.new()
    # Base icosahedron with 2 subdivisions for crisp crystalline faceting
    bmesh.ops.create_icosphere(bm, subdivisions=2, radius=radius)
    
    rng = random.Random(seed)
    
    # Displace vertices with 3D noise and non-uniform compression
    for v in bm.verts:
        p = v.co
        dist = p.length
        if dist > 0.0001:
            normal = p.normalized()
            # 3D noise along normal
            n = noise3d_coord(p.x * 2.0 + seed, p.y * 2.0 + seed, p.z * 2.0 + seed)
            # Flatten bottom slightly to sit naturally on ground
            bottom_flatten = 0.85 if p.z < -0.1 else 1.0
            
            displacement = 1.0 + (n * 0.28) + (rng.uniform(-0.12, 0.12))
            v.co = normal * (dist * displacement * bottom_flatten)
            
            # Apply non-uniform scaling per axis
            v.co.x *= scale[0]
            v.co.y *= scale[1]
            v.co.z *= scale[2]
            
    # Optional planar bisect or planar decimation for sharp cliff facets
    bmesh.ops.planar_faces(bm, faces=bm.faces, iterations=1)
    
    # Enforce flat shading across all faces for crisp cel-shaded edges
    for f in bm.faces:
        f.smooth = False
        
    bm.to_mesh(mesh)
    bm.free()
    
    # Assign materials: Slot 0 = Rock Base, Slot 1 = Top Moss
    obj.data.materials.append(mat_rock)
    obj.data.materials.append(mat_moss)
    
    # Assign Slot 1 (Moss) to faces where normal_z > 0.60 per Technical Artist rules
    for poly in obj.data.polygons:
        if poly.normal.z > 0.60:
            poly.material_index = 1
        else:
            poly.material_index = 0
            
    obj.location = loc
    obj.rotation_euler = rot
    return obj

def generate_rock_cluster(name, count=5, spread=2.0, base_radius=0.8, loc=(0.0, 0.0, 0.0), seed=100):
    """
    Creates an organic cluster of overlapping angular rocks.
    """
    rng = random.Random(seed)
    cluster_objs = []
    
    for i in range(count):
        sub_name = f"{name}_Boulder_{i}"
        rad = base_radius * rng.uniform(0.65, 1.35)
        sx = rng.uniform(0.85, 1.3)
        sy = rng.uniform(0.85, 1.3)
        sz = rng.uniform(0.65, 1.1)
        
        angle = (i / float(count)) * 2.0 * math.pi + rng.uniform(-0.4, 0.4)
        dist = rng.uniform(0.2, spread * 0.5)
        ox = loc[0] + math.cos(angle) * dist
        oy = loc[1] + math.sin(angle) * dist
        oz = loc[2] + rng.uniform(-0.1, 0.2)
        
        rz = rng.uniform(0.0, 2.0 * math.pi)
        rx = rng.uniform(-0.25, 0.25)
        ry = rng.uniform(-0.25, 0.25)
        
        rock = generate_angular_rock(
            sub_name,
            radius=rad,
            scale=(sx, sy, sz),
            seed=seed + i * 17,
            loc=(ox, oy, oz),
            rot=(rx, ry, rz)
        )
        cluster_objs.append(rock)
        
    return cluster_objs

def generate_transition_boulders(name, start_pos, end_pos, count=8, seed=200):
    """
    Creates a row of small angular transition stones separating dirt path from grass.
    """
    rng = random.Random(seed)
    boulders = []
    
    p0 = Vector(start_pos)
    p1 = Vector(end_pos)
    
    for i in range(count):
        t = i / max(1, count - 1)
        pos = p0.lerp(p1, t)
        
        # Add slight serpentine jitter
        pos.x += rng.uniform(-0.18, 0.18)
        pos.y += rng.uniform(-0.18, 0.18)
        
        rad = rng.uniform(0.18, 0.38)
        sx = rng.uniform(0.8, 1.4)
        sy = rng.uniform(0.8, 1.4)
        sz = rng.uniform(0.5, 0.9)
        
        rock = generate_angular_rock(
            f"{name}_Pebble_{i}",
            radius=rad,
            scale=(sx, sy, sz),
            seed=seed + i * 13,
            loc=(pos.x, pos.y, pos.z),
            rot=(rng.uniform(-0.3, 0.3), rng.uniform(-0.3, 0.3), rng.uniform(0, 6.28))
        )
        boulders.append(rock)
        
    return boulders

def generate_hanging_rock_shelf(name, width=4.0, depth=2.0, height=0.8, loc=(0.0, 0.0, 0.0), seed=300):
    """
    Creates a stratified horizontal rock shelf (cave ledge or hanging rock overhang).
    """
    return generate_angular_rock(
        name,
        radius=1.5,
        scale=(width / 3.0, depth / 3.0, height / 3.0),
        seed=seed,
        loc=loc
    )

def build_and_export_rock_kit(output_path="assets/models/faceted_rocks_kit.glb"):
    reset_scene()
    print("Building Procedural Faceted Rock Kit...")
    
    # 1. Individual Master Boulders
    generate_angular_rock("Rock_Angular_Large", radius=1.4, scale=(1.2, 1.0, 0.8), seed=10, loc=(0.0, 0.0, 0.0))
    generate_angular_rock("Rock_Angular_Medium", radius=0.9, scale=(1.0, 1.1, 0.9), seed=20, loc=(4.0, 0.0, 0.0))
    generate_angular_rock("Rock_Angular_Small", radius=0.5, scale=(0.9, 0.9, 0.7), seed=30, loc=(7.0, 0.0, 0.0))
    
    # 2. Rock Clusters
    generate_rock_cluster("Cluster_Cavern", count=5, spread=2.5, base_radius=0.7, loc=(0.0, 6.0, 0.0), seed=40)
    
    # 3. Transition Stones Row (Dirt to Grass border)
    generate_transition_boulders("Transition_Border", (-3.0, -4.0, 0.0), (-3.0, 2.0, 0.0), count=7, seed=50)
    
    # 4. Overhanging Cave Shelf
    generate_hanging_rock_shelf("Shelf_Cavern_Ledge", width=5.0, depth=2.5, height=1.1, loc=(0.0, -6.0, 0.0), seed=60)
    
    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    
    # Select all created objects for glTF export
    bpy.ops.object.select_all(action='SELECT')
    
    bpy.ops.export_scene.gltf(
        filepath=output_path,
        export_format='GLB',
        use_selection=True,
        export_apply=True
    )
    print(f"Faceted Rock Kit successfully exported to: {output_path}")

if __name__ == "__main__":
    build_and_export_rock_kit()
