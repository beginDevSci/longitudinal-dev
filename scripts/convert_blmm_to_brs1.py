#!/usr/bin/env python3
"""
Convert BLMM data files for the brain viewer.

Converts:
1. Statistics: .dat files to BRS1 format (.bin.gz)
2. Geometry: FreeSurfer .pial files to BRG1 format (.bin.gz)
3. Metadata: generates JSON metadata files

The viewer expects:
- Statistics: {base_path}/{hemi}_{analysis}_{stat}.bin.gz
- Geometry: {base_path}/{hemi}_geometry.bin.gz
- Metadata: {base_path}/{hemi}_{analysis}_{stat}.json
"""

import json
import struct
import gzip
import numpy as np
from pathlib import Path

# Constants for fsaverage5
N_VERTICES = 10242

# Map from our statistic names to expected volumes
STATISTICS = {
    "conT": 5,      # 5 contrasts
    "conTlp": 5,    # 5 contrasts
    "beta": 23,     # 23 fixed effects
    "sigma2": 1,    # 1 residual variance
}

# Contrast labels for conT and conTlp (5 contrasts)
CONTRAST_LABELS = [
    "Intercept",
    "Sex",
    "CrossSectionalAge",
    "LongitudinalTime",
    "NIHScore",
]

# Beta coefficient labels (23 fixed effects)
BETA_LABELS = [
    "Intercept",
    "Sex",
    "CrossSectionalAge",
    "LongitudinalTime",
    "NIHScore",
    # Additional covariates (placeholders - adjust as needed)
    *[f"Covariate_{i}" for i in range(6, 24)]
]

# Statistic metadata definitions
STAT_META = {
    "conT": {
        "display_name": "T-Statistics",
        "description": "T-statistics for testing whether contrasts differ from zero",
        "colormap": "coolwarm",
        "symmetric": True,
        "suggested_threshold": 2.0,
        "labels": CONTRAST_LABELS,
    },
    "conTlp": {
        "display_name": "-log10(p-values)",
        "description": "Negative log10 of p-values (values > 1.3 correspond to p < 0.05)",
        "colormap": "plasma",
        "symmetric": False,
        "suggested_threshold": 1.3,
        "labels": CONTRAST_LABELS,
    },
    "beta": {
        "display_name": "Beta Coefficients",
        "description": "Raw fixed effect coefficient estimates",
        "colormap": "coolwarm",
        "symmetric": True,
        "suggested_threshold": None,
        "labels": BETA_LABELS,
    },
    "sigma2": {
        "display_name": "Residual Variance",
        "description": "Residual variance at each vertex",
        "colormap": "viridis",
        "symmetric": False,
        "suggested_threshold": None,
        "labels": ["Residual Variance"],
    },
}


def generate_metadata_json(hemi: str, analysis: str, stat: str, output_path: Path):
    """Generate metadata JSON file for a statistic."""
    meta = STAT_META[stat]

    hemisphere_full = "left" if hemi == "lh" else "right"

    metadata = {
        "name": f"{hemi}_{analysis}_{stat}",
        "display_name": f"{meta['display_name']} ({analysis.upper()})",
        "description": meta["description"],
        "hemisphere": hemisphere_full,
        "analysis": analysis,
        "colormap": meta["colormap"],
        "symmetric": meta["symmetric"],
        "suggested_threshold": meta["suggested_threshold"],
        "nan_handling": "transparent",
        "volumes": [
            {"index": i, "label": label}
            for i, label in enumerate(meta["labels"])
        ],
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "w") as f:
        json.dump(metadata, f, indent=2)

    print(f"  Generated: {output_path.name}")


def read_freesurfer_surface(pial_path: Path):
    """Read a FreeSurfer surface file (.pial, .white, etc.)."""
    with open(pial_path, "rb") as f:
        # Read magic bytes (3 bytes, big-endian)
        magic = f.read(3)
        if magic != b"\xff\xff\xfe":
            raise ValueError(f"Invalid FreeSurfer magic: {magic.hex()}")

        # Read comment (terminated by double newline \n\n)
        comment = b""
        while True:
            char = f.read(1)
            if not char:
                raise ValueError("Unexpected EOF reading comment")
            comment += char
            if comment.endswith(b"\n\n"):
                break

        # Read vertex and face counts (big-endian 32-bit ints)
        n_vertices = struct.unpack(">I", f.read(4))[0]
        n_faces = struct.unpack(">I", f.read(4))[0]

        # Read vertices (big-endian floats)
        vertices = np.zeros((n_vertices, 3), dtype=np.float32)
        for i in range(n_vertices):
            x = struct.unpack(">f", f.read(4))[0]
            y = struct.unpack(">f", f.read(4))[0]
            z = struct.unpack(">f", f.read(4))[0]
            vertices[i] = [x, y, z]

        # Read faces (big-endian ints)
        faces = np.zeros((n_faces, 3), dtype=np.uint32)
        for i in range(n_faces):
            v0 = struct.unpack(">I", f.read(4))[0]
            v1 = struct.unpack(">I", f.read(4))[0]
            v2 = struct.unpack(">I", f.read(4))[0]
            faces[i] = [v0, v1, v2]

    return vertices, faces


def compute_vertex_normals(vertices, faces):
    """Compute per-vertex normals by averaging face normals."""
    normals = np.zeros_like(vertices)

    for face in faces:
        v0, v1, v2 = vertices[face[0]], vertices[face[1]], vertices[face[2]]
        edge1 = v1 - v0
        edge2 = v2 - v0
        face_normal = np.cross(edge1, edge2)
        norm = np.linalg.norm(face_normal)
        if norm > 1e-10:
            face_normal /= norm

        normals[face[0]] += face_normal
        normals[face[1]] += face_normal
        normals[face[2]] += face_normal

    # Normalize
    norms = np.linalg.norm(normals, axis=1, keepdims=True)
    norms[norms < 1e-10] = 1.0
    normals /= norms

    return normals.astype(np.float32)


def convert_pial_to_brg1(pial_path: Path, output_path: Path):
    """Convert a FreeSurfer .pial file to BRG1 format."""
    vertices, faces = read_freesurfer_surface(pial_path)
    normals = compute_vertex_normals(vertices, faces)

    n_vertices = len(vertices)
    n_faces = len(faces)

    # Build BRG1 binary
    buf = bytearray()

    # Header
    buf.extend(b"BRG1")                              # Magic
    buf.extend(struct.pack("<I", 1))                 # Version
    buf.extend(struct.pack("<I", 0))                 # Flags
    buf.extend(struct.pack("<I", n_vertices))        # n_vertices
    buf.extend(struct.pack("<I", n_faces))           # n_faces

    # Vertices (little-endian floats)
    for v in vertices:
        buf.extend(struct.pack("<fff", v[0], v[1], v[2]))

    # Normals (little-endian floats)
    for n in normals:
        buf.extend(struct.pack("<fff", n[0], n[1], n[2]))

    # Faces (little-endian uint32)
    for f in faces:
        buf.extend(struct.pack("<III", f[0], f[1], f[2]))

    # Write gzip-compressed
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with gzip.open(output_path, "wb") as f:
        f.write(bytes(buf))

    print(f"  Converted: {pial_path.name} -> {output_path.name}")
    print(f"    {n_vertices} vertices, {n_faces} faces")

def convert_dat_to_brs1(dat_path: Path, output_path: Path, expected_volumes: int):
    """Convert a .dat file to BRS1 format."""
    # Read raw floats
    raw_data = np.fromfile(dat_path, dtype=np.float32)

    n_values = len(raw_data)
    n_vertices = N_VERTICES
    n_volumes = n_values // n_vertices

    if n_volumes != expected_volumes:
        print(f"  Warning: Expected {expected_volumes} volumes, got {n_volumes}")

    if n_values != n_vertices * n_volumes:
        raise ValueError(f"Data size mismatch: {n_values} values for {n_vertices}x{n_volumes}")

    # Reshape to (n_volumes, n_vertices) for per-volume stats
    data = raw_data.reshape((n_volumes, n_vertices))

    # Compute statistics
    nan_mask = np.isnan(data)
    nan_count = np.sum(nan_mask)

    valid_data = data[~nan_mask] if nan_count > 0 else data.flatten()
    global_min = float(np.min(valid_data)) if len(valid_data) > 0 else 0.0
    global_max = float(np.max(valid_data)) if len(valid_data) > 0 else 0.0

    # Per-volume ranges
    volume_ranges = []
    for v in range(n_volumes):
        vol_data = data[v]
        vol_valid = vol_data[~np.isnan(vol_data)]
        if len(vol_valid) > 0:
            volume_ranges.append((float(np.min(vol_valid)), float(np.max(vol_valid))))
        else:
            volume_ranges.append((0.0, 0.0))

    # Build BRS1 binary
    buf = bytearray()

    # Header
    buf.extend(b"BRS1")                              # Magic
    buf.extend(struct.pack("<I", 1))                 # Version
    buf.extend(struct.pack("<I", 0))                 # Flags
    buf.extend(struct.pack("<I", n_vertices))        # n_vertices
    buf.extend(struct.pack("<I", n_volumes))         # n_volumes
    buf.extend(struct.pack("<f", global_min))        # global_min
    buf.extend(struct.pack("<f", global_max))        # global_max
    buf.extend(struct.pack("<I", int(nan_count)))    # nan_count

    # Volume ranges
    for vmin, vmax in volume_ranges:
        buf.extend(struct.pack("<ff", vmin, vmax))

    # Data values (already in correct order: volume-major)
    buf.extend(raw_data.tobytes())

    # Write gzip-compressed
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with gzip.open(output_path, "wb") as f:
        f.write(bytes(buf))

    print(f"  Converted: {dat_path.name} -> {output_path.name}")
    print(f"    {n_vertices} vertices, {n_volumes} volumes")
    print(f"    Range: [{global_min:.3f}, {global_max:.3f}], NaN count: {nan_count}")

def main():
    base_dir = Path(__file__).parent.parent / "public" / "data" / "blmm"

    hemispheres = ["lh", "rh"]
    analyses = ["des1"]  # Only des1 is available

    # Convert geometry (surfaces)
    print("Converting geometry files...")
    for hemi in hemispheres:
        pial_file = base_dir / f"{hemi}.pial"
        if pial_file.exists():
            out_file = base_dir / f"{hemi}_geometry.bin.gz"
            convert_pial_to_brg1(pial_file, out_file)
        else:
            print(f"  Skipping {hemi}.pial (not found)")

    # Convert statistics
    for hemi in hemispheres:
        for ana in analyses:
            src_dir = base_dir / f"results_{hemi}_{ana}"

            if not src_dir.exists():
                print(f"Skipping {src_dir} (not found)")
                continue

            print(f"\nProcessing {src_dir}...")

            for stat, expected_vols in STATISTICS.items():
                dat_file = src_dir / f"blmm_vox_{stat}.dat"
                if not dat_file.exists():
                    print(f"  Skipping {stat} (not found)")
                    continue

                # Output to flat structure expected by viewer
                out_file = base_dir / f"{hemi}_{ana}_{stat}.bin.gz"
                convert_dat_to_brs1(dat_file, out_file, expected_vols)

                # Generate metadata JSON
                json_file = base_dir / f"{hemi}_{ana}_{stat}.json"
                generate_metadata_json(hemi, ana, stat, json_file)

    print("\nDone! Files ready for the brain viewer.")

if __name__ == "__main__":
    main()
