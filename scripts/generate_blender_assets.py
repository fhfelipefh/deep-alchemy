"""
Script de automação para o Blender 4.x / 5.x.
Gera os modelos 3D autorais para a IFPF Game Jam (Alquimia e Bichinhos):
- Alquimista Explorador (Chapéu estilo Indiana Jones, mochila grande, pochete utilitária, lanterna de cabeça)
- Estátua de Ouro (Golden Idol)
- Slime Fofo (Bichinho)
- Pequeno Lobo da Caverna (Bichinho)
- Frascos de Poções Alquímicas
- Cogumelos Bioluminescentes
"""

import sys
import os

try:
    import bpy
    import math
except ImportError:
    print("Este script deve ser executado no ambiente Python do Blender.")
    sys.exit(0)

OUTPUT_DIR = os.path.join(os.path.dirname(__file__), "..", "public", "models")
os.makedirs(OUTPUT_DIR, exist_ok=True)

def clear_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)

def create_toon_material(name, color_rgba, emission_strength=0.0):
    mat = bpy.data.materials.new(name=name)
    mat.use_nodes = True
    nodes = mat.node_tree.nodes
    links = mat.node_tree.links
    nodes.clear()

    output = nodes.new(type="ShaderNodeOutputMaterial")
    bsdf = nodes.new(type="ShaderNodeBsdfPrincipled")
    bsdf.inputs['Base Color'].default_value = color_rgba
    bsdf.inputs['Roughness'].default_value = 0.35
    if emission_strength > 0:
        bsdf.inputs['Emission Color'].default_value = color_rgba
        bsdf.inputs['Emission Strength'].default_value = emission_strength
    links.new(bsdf.outputs['BSDF'], output.inputs['Surface'])
    return mat

def build_alchemist():
    clear_scene()
    mat_clothes = create_toon_material("AlchemistClothes", (0.55, 0.42, 0.28, 1.0))
    mat_hat = create_toon_material("AlchemistHat", (0.35, 0.22, 0.12, 1.0))
    mat_backpack = create_toon_material("AlchemistPack", (0.28, 0.35, 0.22, 1.0))
    mat_skin = create_toon_material("AlchemistSkin", (0.95, 0.78, 0.65, 1.0))
    mat_gold = create_toon_material("AlchemistBrass", (0.9, 0.75, 0.2, 1.0))

    # Torso (1.80m de altura total)
    bpy.ops.mesh.primitive_cylinder_add(radius=0.25, depth=0.7, location=(0, 0, 1.05))
    bpy.context.active_object.data.materials.append(mat_clothes)

    # Cabeça
    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.18, location=(0, 0, 1.55))
    bpy.context.active_object.data.materials.append(mat_skin)

    # Chapéu Indiana Jones
    bpy.ops.mesh.primitive_cylinder_add(radius=0.35, depth=0.03, location=(0, 0, 1.68))
    bpy.context.active_object.data.materials.append(mat_hat)

    bpy.ops.mesh.primitive_cylinder_add(radius=0.19, depth=0.14, location=(0, 0, 1.76))
    bpy.context.active_object.data.materials.append(mat_hat)

    # Lanterna de cabeça
    bpy.ops.mesh.primitive_cube_add(size=0.08, location=(0, 0.19, 1.66))
    bpy.context.active_object.data.materials.append(mat_gold)

    # Mochila grande
    bpy.ops.mesh.primitive_cube_add(size=0.32, location=(0, -0.22, 1.15))
    pack = bpy.context.active_object
    pack.scale = (0.9, 0.7, 1.3)
    pack.data.materials.append(mat_backpack)

    # Pochete frontal
    bpy.ops.mesh.primitive_cube_add(size=0.14, location=(0.12, 0.22, 0.75))
    pouch = bpy.context.active_object
    pouch.scale = (1.4, 0.6, 0.8)
    pouch.data.materials.append(mat_hat)

    # Pernas
    bpy.ops.mesh.primitive_cylinder_add(radius=0.1, depth=0.7, location=(-0.14, 0, 0.35))
    bpy.context.active_object.data.materials.append(mat_clothes)

    bpy.ops.mesh.primitive_cylinder_add(radius=0.1, depth=0.7, location=(0.14, 0, 0.35))
    bpy.context.active_object.data.materials.append(mat_clothes)

    filepath = os.path.join(OUTPUT_DIR, "alchemist.glb")
    bpy.ops.export_scene.gltf(filepath=filepath, export_format='GLB')
    print(f"Alquimista exportado: {filepath}")

def build_golden_idol():
    clear_scene()
    mat_gold = create_toon_material("PureGold", (1.0, 0.82, 0.1, 1.0))

    bpy.ops.mesh.primitive_cylinder_add(radius=0.15, depth=0.05, location=(0, 0, 0.025))
    bpy.context.active_object.data.materials.append(mat_gold)

    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.12, location=(0, 0, 0.2))
    bpy.context.active_object.data.materials.append(mat_gold)

    filepath = os.path.join(OUTPUT_DIR, "golden_idol.glb")
    bpy.ops.export_scene.gltf(filepath=filepath, export_format='GLB')
    print(f"Estátua dourada exportada: {filepath}")

def build_slime():
    clear_scene()
    mat_slime = create_toon_material("SlimeJelly", (0.2, 0.85, 0.45, 0.85))
    mat_eye = create_toon_material("AnimeEye", (0.05, 0.05, 0.05, 1.0))
    mat_eye_white = create_toon_material("EyeWhite", (1.0, 1.0, 1.0, 1.0))

    # Corpo esférico gelatinoso
    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.45, location=(0, 0, 0.45))
    body = bpy.context.active_object
    body.scale = (1.0, 1.0, 0.85)
    body.data.materials.append(mat_slime)

    # Olho esquerdo
    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.07, location=(-0.16, 0.38, 0.48))
    bpy.context.active_object.data.materials.append(mat_eye)
    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.025, location=(-0.14, 0.43, 0.51))
    bpy.context.active_object.data.materials.append(mat_eye_white)

    # Olho direito
    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.07, location=(0.16, 0.38, 0.48))
    bpy.context.active_object.data.materials.append(mat_eye)
    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.025, location=(0.18, 0.43, 0.51))
    bpy.context.active_object.data.materials.append(mat_eye_white)

    filepath = os.path.join(OUTPUT_DIR, "slime.glb")
    bpy.ops.export_scene.gltf(filepath=filepath, export_format='GLB')
    print(f"Slime exportado: {filepath}")

def build_cave_wolf():
    clear_scene()
    mat_fur = create_toon_material("WolfFur", (0.35, 0.38, 0.45, 1.0))
    mat_belly = create_toon_material("WolfBelly", (0.75, 0.78, 0.82, 1.0))
    mat_nose = create_toon_material("WolfNose", (0.1, 0.1, 0.12, 1.0))

    # Tronco
    bpy.ops.mesh.primitive_cylinder_add(radius=0.22, depth=0.6, location=(0, 0, 0.45))
    body = bpy.context.active_object
    body.rotation_euler = (math.radians(90), 0, 0)
    body.data.materials.append(mat_fur)

    # Cabeça
    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.2, location=(0, 0.4, 0.6))
    bpy.context.active_object.data.materials.append(mat_fur)

    # Focinho fofo
    bpy.ops.mesh.primitive_cube_add(size=0.12, location=(0, 0.58, 0.56))
    bpy.context.active_object.data.materials.append(mat_belly)

    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.03, location=(0, 0.65, 0.59))
    bpy.context.active_object.data.materials.append(mat_nose)

    # Orelhas fofas
    bpy.ops.mesh.primitive_cone_add(radius1=0.07, depth=0.15, location=(-0.12, 0.38, 0.82))
    bpy.context.active_object.data.materials.append(mat_fur)

    bpy.ops.mesh.primitive_cone_add(radius1=0.07, depth=0.15, location=(0.12, 0.38, 0.82))
    bpy.context.active_object.data.materials.append(mat_fur)

    # Cauda
    bpy.ops.mesh.primitive_cylinder_add(radius=0.06, depth=0.35, location=(0, -0.4, 0.55))
    tail = bpy.context.active_object
    tail.rotation_euler = (math.radians(-45), 0, 0)
    tail.data.materials.append(mat_fur)

    filepath = os.path.join(OUTPUT_DIR, "cave_wolf.glb")
    bpy.ops.export_scene.gltf(filepath=filepath, export_format='GLB')
    print(f"Cave Wolf exportado: {filepath}")

def build_potion_flask():
    clear_scene()
    mat_glass = create_toon_material("FlaskGlass", (0.85, 0.95, 1.0, 0.6))
    mat_cork = create_toon_material("FlaskCork", (0.45, 0.3, 0.15, 1.0))

    # Base esférica do frasco
    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.15, location=(0, 0, 0.15))
    bpy.context.active_object.data.materials.append(mat_glass)

    # Gargalo cilíndrico
    bpy.ops.mesh.primitive_cylinder_add(radius=0.05, depth=0.12, location=(0, 0, 0.32))
    bpy.context.active_object.data.materials.append(mat_glass)

    # Rolha de cortiça
    bpy.ops.mesh.primitive_cylinder_add(radius=0.055, depth=0.06, location=(0, 0, 0.4))
    bpy.context.active_object.data.materials.append(mat_cork)

    filepath = os.path.join(OUTPUT_DIR, "potion_flask.glb")
    bpy.ops.export_scene.gltf(filepath=filepath, export_format='GLB')
    print(f"Frasco de poção exportado: {filepath}")

def build_bioluminescent_mushroom():
    clear_scene()
    mat_stem = create_toon_material("MushroomStem", (0.85, 0.85, 0.75, 1.0))
    mat_cap = create_toon_material("MushroomCapGlow", (0.1, 0.9, 0.95, 1.0), emission_strength=2.5)

    # Caule curvo
    bpy.ops.mesh.primitive_cylinder_add(radius=0.05, depth=0.4, location=(0, 0, 0.2))
    bpy.context.active_object.data.materials.append(mat_stem)

    # Chapéu do cogumelo emissor
    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.22, location=(0, 0, 0.4))
    cap = bpy.context.active_object
    cap.scale = (1.0, 1.0, 0.5)
    cap.data.materials.append(mat_cap)

    filepath = os.path.join(OUTPUT_DIR, "glowing_mushroom.glb")
    bpy.ops.export_scene.gltf(filepath=filepath, export_format='GLB')
    print(f"Cogumelo bioluminescente exportado: {filepath}")

if __name__ == "__main__":
    build_alchemist()
    build_golden_idol()
    build_slime()
    build_cave_wolf()
    build_potion_flask()
    build_bioluminescent_mushroom()
