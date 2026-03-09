#!/usr/bin/env python3
"""Extract structured contract JSON from XSD files.

Usage:
  python3 scripts/extract_xsd_contract.py \
    --template-id atmos_ec3_v1 \
    --raw-dir tests/fixtures/xsd/raw \
    --output tests/fixtures/xsd/contract.atmos_ec3_v1.json
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys
import xml.etree.ElementTree as ET


def local_name(tag: str) -> str:
    if "}" in tag:
        return tag.split("}", 1)[1]
    return tag


def normalize_qname(value: str | None) -> str | None:
    if value is None:
        return None
    if ":" in value:
        return value.split(":", 1)[1]
    return value


def parse_simple_type(node: ET.Element, name: str | None) -> dict:
    restriction = None
    for child in node:
        if local_name(child.tag) == "restriction":
            restriction = child
            break

    base = None
    enumerations: list[str] = []
    min_inclusive = None
    max_inclusive = None
    pattern = None

    if restriction is not None:
        base = normalize_qname(restriction.attrib.get("base"))
        for child in restriction:
            lname = local_name(child.tag)
            value = child.attrib.get("value")
            if lname == "enumeration" and value is not None:
                enumerations.append(value)
            elif lname == "minInclusive" and value is not None:
                min_inclusive = value
            elif lname == "maxInclusive" and value is not None:
                max_inclusive = value
            elif lname == "pattern" and value is not None:
                pattern = value

    return {
        "name": name,
        "base": base,
        "enumerations": sorted(enumerations),
        "min_inclusive": min_inclusive,
        "max_inclusive": max_inclusive,
        "pattern": pattern,
    }


def add_element(elements: dict, paths: dict, path: str, name: str, type_name: str | None, min_occurs: str, max_occurs: str) -> None:
    entry = {
        "name": name,
        "path": path,
        "type": type_name,
        "min_occurs": min_occurs,
        "max_occurs": max_occurs,
    }
    elements[path] = entry
    paths[path] = {
        "path": path,
        "element": name,
        "type": type_name,
    }


def add_attribute(attributes: dict, owner_path: str, attr_node: ET.Element) -> None:
    name = attr_node.attrib.get("name")
    if name is None:
        ref = attr_node.attrib.get("ref")
        if ref is not None:
            name = normalize_qname(ref)
    if name is None:
        return

    key = (owner_path, name)
    attributes[key] = {
        "name": name,
        "owner_path": owner_path,
        "required": attr_node.attrib.get("use") == "required",
        "type": normalize_qname(attr_node.attrib.get("type")),
    }


def walk_particle(
    node: ET.Element,
    owner_path: str,
    *,
    walk_element,
) -> None:
    for child in node:
        lname = local_name(child.tag)
        if lname in {"sequence", "choice", "all"}:
            walk_particle(child, owner_path, walk_element=walk_element)
        elif lname == "element":
            walk_element(child, owner_path)


def parse_xsd(path: Path) -> tuple[dict, dict, dict, dict]:
    tree = ET.parse(path)
    root = tree.getroot()

    complex_types: dict[str, ET.Element] = {}
    simple_types: dict[str, dict] = {}
    global_elements: list[ET.Element] = []

    for child in root:
        lname = local_name(child.tag)
        if lname == "complexType" and "name" in child.attrib:
            complex_types[child.attrib["name"]] = child
        elif lname == "simpleType" and "name" in child.attrib:
            simple_types[child.attrib["name"]] = parse_simple_type(child, child.attrib["name"])
        elif lname == "element":
            global_elements.append(child)

    elements: dict[str, dict] = {}
    attributes: dict[tuple[str, str], dict] = {}
    paths: dict[str, dict] = {}
    visited_complex: set[tuple[str, str]] = set()

    def walk_named_complex_type(type_name: str, owner_path: str) -> None:
        key = (owner_path, type_name)
        if key in visited_complex:
            return
        visited_complex.add(key)

        ct = complex_types.get(type_name)
        if ct is None:
            return
        walk_complex_type(ct, owner_path)

    def walk_complex_content(content: ET.Element, owner_path: str) -> None:
        for child in content:
            lname = local_name(child.tag)
            if lname not in {"extension", "restriction"}:
                continue

            base = normalize_qname(child.attrib.get("base"))
            if base is not None:
                walk_named_complex_type(base, owner_path)

            for grand in child:
                glname = local_name(grand.tag)
                if glname in {"sequence", "choice", "all"}:
                    walk_particle(grand, owner_path, walk_element=walk_element)
                elif glname == "attribute":
                    add_attribute(attributes, owner_path, grand)

    def walk_complex_type(node: ET.Element, owner_path: str) -> None:
        for child in node:
            lname = local_name(child.tag)
            if lname in {"sequence", "choice", "all"}:
                walk_particle(child, owner_path, walk_element=walk_element)
            elif lname == "attribute":
                add_attribute(attributes, owner_path, child)
            elif lname in {"complexContent", "simpleContent"}:
                walk_complex_content(child, owner_path)

    def walk_element(node: ET.Element, parent_path: str) -> None:
        name = node.attrib.get("name")
        if name is None:
            ref = node.attrib.get("ref")
            if ref is not None:
                name = normalize_qname(ref)
        if name is None:
            return

        path = f"{parent_path}/{name}" if parent_path else f"/{name}"
        type_name = normalize_qname(node.attrib.get("type"))
        min_occurs = node.attrib.get("minOccurs", "1")
        max_occurs = node.attrib.get("maxOccurs", "1")

        add_element(elements, paths, path, name, type_name, min_occurs, max_occurs)

        inline_simple_type = None
        inline_complex_type = None
        for child in node:
            lname = local_name(child.tag)
            if lname == "simpleType":
                inline_simple_type = child
            elif lname == "complexType":
                inline_complex_type = child

        if inline_simple_type is not None:
            anon_name = f"anon::{path}"
            simple_types[anon_name] = parse_simple_type(inline_simple_type, anon_name)

        if inline_complex_type is not None:
            walk_complex_type(inline_complex_type, path)
        elif type_name is not None:
            walk_named_complex_type(type_name, path)

    for element in global_elements:
        walk_element(element, "")

    return elements, attributes, simple_types, paths


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Extract structured JSON contract from XSD")
    parser.add_argument("--xsd", action="append", default=[], help="XSD file path. Repeatable.")
    parser.add_argument(
        "--raw-dir",
        default="tests/fixtures/xsd/raw",
        help="Directory containing *.xsd files if --xsd is not set.",
    )
    parser.add_argument("--output", required=True, help="Output JSON path")
    parser.add_argument("--template-id", required=True, help="Template id for metadata")
    parser.add_argument("--dee-version", default="unknown", help="DEE version for metadata")
    parser.add_argument("--exported-at", default="unknown", help="XSD export timestamp")
    return parser.parse_args()


def resolve_input_files(args: argparse.Namespace) -> list[Path]:
    files: list[Path] = []

    if args.xsd:
        files.extend(Path(p) for p in args.xsd)
    else:
        raw_dir = Path(args.raw_dir)
        files.extend(sorted(raw_dir.glob("*.xsd")))

    missing = [str(p) for p in files if not p.exists()]
    if missing:
        raise SystemExit(f"xsd file(s) not found: {', '.join(missing)}")

    if not files:
        raise SystemExit("no xsd inputs found; pass --xsd or place *.xsd under --raw-dir")

    return sorted(files)


def main() -> int:
    args = parse_args()
    xsd_files = resolve_input_files(args)

    merged_elements: dict[str, dict] = {}
    merged_attributes: dict[tuple[str, str], dict] = {}
    merged_simple_types: dict[str, dict] = {}
    merged_paths: dict[str, dict] = {}

    for file_path in xsd_files:
        elements, attributes, simple_types, paths = parse_xsd(file_path)
        merged_elements.update(elements)
        merged_attributes.update(attributes)
        merged_simple_types.update(simple_types)
        merged_paths.update(paths)

    contract = {
        "meta": {
            "template_id": args.template_id,
            "dee_version": args.dee_version,
            "exported_at": args.exported_at,
            "source_files": [p.name for p in xsd_files],
        },
        "elements": [merged_elements[key] for key in sorted(merged_elements.keys())],
        "attributes": [
            merged_attributes[key]
            for key in sorted(merged_attributes.keys(), key=lambda item: (item[0], item[1]))
        ],
        "simple_types": [
            merged_simple_types[key] for key in sorted(merged_simple_types.keys())
        ],
        "paths": [merged_paths[key] for key in sorted(merged_paths.keys())],
    }

    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(contract, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    return 0


if __name__ == "__main__":
    sys.exit(main())
