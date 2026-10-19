import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

def find_workspace_root() -> Path:

    env_root = os.environ.get("DEEP_ALCHEMY_ROOT")
    if env_root and Path(env_root).exists():
        return Path(env_root).resolve()

    current = Path(__file__).resolve()
    for parent in [current] + list(current.parents):
        if (parent / "Cargo.toml").exists() and (parent / "assets").exists():
            return parent
    return Path.cwd().resolve()

def find_game_binary(workspace_root: Optional[Path] = None) -> Optional[Path]:

    root = workspace_root or find_workspace_root()
    debug_bin = root / "target" / "debug" / "deep_alchemy.exe"
    release_bin = root / "target" / "release" / "deep_alchemy.exe"

    if release_bin.exists():
        return release_bin
    if debug_bin.exists():
        return debug_bin
    return None

def run_game(
    args: Optional[List[str]] = None,
    timeout: Optional[float] = None,
    workspace_root: Optional[Path] = None,
) -> Dict[str, Any]:

    root = workspace_root or find_workspace_root()
    binary = find_game_binary(root)

    cmd: List[str]
    if binary:
        cmd = [str(binary)] + (args or [])
    else:
        cargo = shutil.which("cargo")
        if not cargo:
            raise RuntimeError(
                "Neither deep_alchemy binary nor cargo found. Compile first with 'cargo build'."
            )
        cmd = [cargo, "run", "--"] + (args or [])

    try:
        proc = subprocess.run(
            cmd,
            cwd=str(root),
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        return {
            "success": proc.returncode == 0,
            "returncode": proc.returncode,
            "stdout": proc.stdout,
            "stderr": proc.stderr,
            "command": " ".join(cmd),
        }
    except subprocess.TimeoutExpired as exc:
        return {
            "success": False,
            "error": "Execution timed out",
            "timeout": timeout,
            "stdout": exc.stdout if isinstance(exc.stdout, str) else "",
            "stderr": exc.stderr if isinstance(exc.stderr, str) else "",
        }

def run_screenshot_test(workspace_root: Optional[Path] = None) -> Dict[str, Any]:

    root = workspace_root or find_workspace_root()
    result = run_game(args=["--screenshot-test"], timeout=45.0, workspace_root=root)

    front_frame = root / "debug_output" / "frame_check.png"
    rear_frame = root / "debug_output" / "frame_rear_check.png"

    result["front_frame_path"] = str(front_frame) if front_frame.exists() else None
    result["rear_frame_path"] = str(rear_frame) if rear_frame.exists() else None
    result["frames_captured"] = front_frame.exists() and rear_frame.exists()

    return result

def validate_frame(image_path: Path) -> Dict[str, Any]:

    if not image_path.exists():
        return {"error": f"Image file not found: {image_path}", "passed": False}

    try:
        from PIL import Image
        import numpy as np
    except ImportError:

        file_size = image_path.stat().st_size
        return {
            "passed": file_size > 10_000,
            "file_size": file_size,
            "path": str(image_path),
            "note": "Pillow/numpy not in environment; verified file existence and size",
        }

    img = Image.open(image_path).convert("RGB")
    arr = np.array(img, dtype=np.float32)
    h, w, _ = arr.shape
    total_pixels = h * w

    r, g, b = arr[:, :, 0], arr[:, :, 1], arr[:, :, 2]
    max_c = np.maximum(np.maximum(r, g), b)
    min_c = np.minimum(np.minimum(r, g), b)
    delta = max_c - min_c

    saturation = np.zeros_like(max_c)
    non_zero = max_c > 0
    saturation[non_zero] = delta[non_zero] / max_c[non_zero]

    gray_mask = (saturation < 0.08) & (arr.mean(axis=2) > 40.0) & (arr.mean(axis=2) < 220.0)
    gray_ratio = float(np.sum(gray_mask) / total_pixels)

    vibrant_mask = (saturation > 0.28) & (max_c > 65.0)
    vibrant_ratio = float(np.sum(vibrant_mask) / total_pixels)

    lum = 0.299 * r + 0.587 * g + 0.114 * b
    dx = np.diff(lum, axis=1)[:, :-1]
    dy = np.diff(lum, axis=0)[:-1, :]
    min_h = min(dx.shape[0], dy.shape[0])
    min_w = min(dx.shape[1], dy.shape[1])
    grad = np.sqrt(dx[:min_h, :min_w] ** 2 + dy[:min_h, :min_w] ** 2)

    edge_pixels = np.sum(grad > 38.0)
    edge_density = float(edge_pixels / (min_h * min_w))

    passed = (gray_ratio <= 0.30) and (edge_density >= 0.008)

    return {
        "path": str(image_path),
        "resolution": [w, h],
        "gray_ratio": round(gray_ratio, 4),
        "vibrant_ratio": round(vibrant_ratio, 4),
        "mean_saturation": round(float(np.mean(saturation)), 4),
        "edge_density": round(edge_density, 4),
        "passed": passed,
    }

def get_latest_telemetry(workspace_root: Optional[Path] = None) -> Optional[Dict[str, Any]]:

    root = workspace_root or find_workspace_root()
    telemetry_file = root / "debug_output" / "camera_frustum_dump.json"
    if not telemetry_file.exists():
        return None
    try:
        with open(telemetry_file, "r", encoding="utf-8") as f:
            return json.load(f)
    except Exception:
        return None
