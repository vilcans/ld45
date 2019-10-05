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

    for obj in objects:
        if obj.type != 'MESH':
            continue
        if include is not None and obj.name not in include:
            continue
        if obj.name in exclude:
            continue

        mat = obj.matrix_world
        mesh = obj.to_mesh()

        for poly in mesh.polygons:
            print("Polygon index: %d, length: %d" %
                  (poly.index, poly.loop_total))
            vertices = []

            for loop_index in poly.loop_indices:
                v = mat @ mesh.vertices[mesh.loops[loop_index].vertex_index].co
                vertices.append(v)

            polygons.append(vertices)

    return polygons


def export(polygons, out):
    # bincode uses 64 bit values for vector length
    out.write(struct.pack('<II', len(polygons), 0))
    for polygon in polygons:
        vertices = polygon
        out.write(struct.pack('<II', len(vertices), 0))
        for v in vertices:
            out.write(struct.pack('ff', v[0], v[2]))


def main(args):
    parser = argparse.ArgumentParser(
        description='Convert meshes'
    )
    parser.add_argument(
        'out',
        help='Save mesh data to this file'
    )
    parser.add_argument(
        '--exclude', nargs='*', default=[],
        help='Exclude object with this name'
    )
    parser.add_argument(
        '--include', nargs='*', default=None,
        help='Include only objects with these names'
    )
    args = parser.parse_args(args=args)

    polygons = convert(C.scene.objects, exclude=args.exclude,
                       include=args.include)

    with open(args.out, 'wb') as out:
        print('Writing', args.out)
        export(polygons, out)


i = sys.argv.index('--')
main(sys.argv[i + 1:])
