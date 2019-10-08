import struct
import sys
import argparse

import bpy
from bpy import data as D
from bpy import context as C
from mathutils import *
from math import *


def convert(objects, *, include, exclude):
    polygons = []
    triggers = {}

    for obj in objects:
        if obj.type != 'MESH':
            continue
        if include is not None and obj.name not in include:
            continue
        if obj.name in exclude:
            continue

        mat = obj.matrix_world

        bounds = [mat @ Vector(c) for c in obj.bound_box]
        min_x = min(c[0] for c in bounds)
        max_x = max(c[0] for c in bounds)
        min_z = min(c[2] for c in bounds)
        max_z = max(c[2] for c in bounds)

        if obj.name.startswith('Trigger_'):
            _, s = obj.name.split('_', 1)
            trigger_id = int(s, 0)
            if trigger_id in triggers:
                raise RuntimeError('Duplicate trigger: %s' % trigger_id)
            triggers[trigger_id] = (min_x, max_x, min_z, max_z)

        mesh = obj.to_mesh()

        for poly in mesh.polygons:
            print("Polygon index: %d, length: %d" %
                  (poly.index, poly.loop_total))
            vertices = []

            for loop_index in poly.loop_indices:
                v = mat @ mesh.vertices[mesh.loops[loop_index].vertex_index].co
                vertices.append(v)

            polygons.append(vertices)

    return polygons, triggers


def write_usize(out, value):
    # bincode uses 64 bit values for vector length, hence the filler high dword
    out.write(struct.pack('<II', value, 0))


def export(polygons, triggers, out):
    write_usize(out, len(polygons))
    for polygon in polygons:
        vertices = polygon
        write_usize(out, len(vertices))
        for v in vertices:
            out.write(struct.pack('ff', v[0], v[2]))

    write_usize(out, len(triggers))
    for trigger_id, bounds in triggers.items():
        out.write(struct.pack('<Iffff', trigger_id, *bounds))


def main(args):
    parser = argparse.ArgumentParser(
        description='Convert meshes'
    )
    parser.add_argument(
        'out',
        help='Save mesh data to this file'
    )
    parser.add_argument(
        '--exclude', action='append',
        help='Exclude object with this name'
    )
    parser.add_argument(
        '--include', action='append', default=None,
        help='Include only objects with these names'
    )
    args = parser.parse_args(args=args)

    polygons, triggers = convert(C.scene.objects, exclude=args.exclude or [],
                                 include=args.include)

    with open(args.out, 'wb') as out:
        print('Writing', args.out)
        export(polygons, triggers, out)


i = sys.argv.index('--')
main(sys.argv[i + 1:])
