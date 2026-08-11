#!/usr/bin/env python3

from __future__ import annotations

import argparse
import struct
import sys
from pathlib import Path
from typing import Sequence


NRO_HEADER_SIZE = 0x80
NRO_MAGIC_OFFSET = 0x10
NRO_SIZE_OFFSET = 0x18
NRO_SEGMENT_OFFSETS = (0x20, 0x28, 0x30)
NRO_BUILD_ID_OFFSET = 0x40
NRO_BUILD_ID_SIZE = 0x20


class VerificationError(RuntimeError):
  pass


def read_u32(data: bytes, offset: int) -> int:
  return struct.unpack_from("<I", data, offset)[0]


def verify_nro(path: Path) -> None:
  try:
    data = path.read_bytes()
  except OSError as error:
    raise VerificationError(f"cannot read {path}: {error}") from error

  if len(data) < NRO_HEADER_SIZE:
    raise VerificationError(
      f"file is too small for an NRO header: {len(data)} bytes"
    )
  if data[NRO_MAGIC_OFFSET : NRO_MAGIC_OFFSET + 4] != b"NRO0":
    raise VerificationError("missing NRO0 header magic")

  declared_size = read_u32(data, NRO_SIZE_OFFSET)
  if declared_size != len(data):
    raise VerificationError(
      f"declared NRO size {declared_size} does not match file size {len(data)}"
    )

  previous_end = 0
  segment_names = ("text", "rodata", "data")
  for name, offset in zip(segment_names, NRO_SEGMENT_OFFSETS):
    segment_offset, segment_size = struct.unpack_from("<II", data, offset)
    segment_end = segment_offset + segment_size
    if segment_size == 0:
      raise VerificationError(f"{name} segment is empty")
    if segment_offset < previous_end:
      raise VerificationError(f"{name} segment overlaps the previous segment")
    if segment_end > declared_size:
      raise VerificationError(f"{name} segment extends past the NRO file")
    previous_end = segment_end

  mod_offset = read_u32(data, 0x04)
  if mod_offset + 4 > declared_size or data[mod_offset : mod_offset + 4] != b"MOD0":
    raise VerificationError("MOD0 header is missing or outside the NRO")

  build_id = data[NRO_BUILD_ID_OFFSET : NRO_BUILD_ID_OFFSET + NRO_BUILD_ID_SIZE]
  if not any(build_id):
    raise VerificationError("NRO build ID is empty")

  print(
    f"verified {path}: {declared_size} bytes, "
    f"build ID {build_id.hex().upper()}"
  )


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
  parser = argparse.ArgumentParser(description="Validate a Skyline NRO artifact.")
  parser.add_argument("nro", type=Path, help="path to the NRO file")
  return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
  args = parse_args(argv)
  try:
    verify_nro(args.nro)
  except VerificationError as error:
    print(f"NRO verification failed: {error}", file=sys.stderr)
    return 1
  return 0


if __name__ == "__main__":
  raise SystemExit(main())
