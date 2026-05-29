#!/usr/bin/env bash
set -euo pipefail

input_dir="${1:-}"
output_dir="${2:-}"
sizes_arg="${3:-256}"

if [[ -z "$input_dir" || -z "$output_dir" ]]; then
  echo "Usage: $0 <input-dir> <output-dir> [sizes]" >&2
  echo "Example: $0 packs/<mod>/assets/svg packs/<mod>/assets/ui 16,32,48,256" >&2
  exit 1
fi

parse_sizes() {
  local raw="$1"
  local -a out=()
  local part
  IFS=',; ' read -r -a parts <<< "$raw"
  for part in "${parts[@]}"; do
    [[ -z "$part" ]] && continue
    if [[ ! "$part" =~ ^[0-9]+$ ]] || [[ "$part" -le 0 ]]; then
      echo "Invalid size value '$part'. Sizes must be positive integers separated by commas, semicolons, or spaces." >&2
      exit 1
    fi
    if [[ ! " ${out[*]} " =~ " $part " ]]; then
      out+=("$part")
    fi
  done
  if [[ ${#out[@]} -eq 0 ]]; then
    echo "No valid sizes provided." >&2
    exit 1
  fi
  printf '%s\n' "${out[@]}"
}

detect_tool() {
  if command -v inkscape >/dev/null 2>&1; then
    echo "inkscape"
  elif command -v resvg >/dev/null 2>&1; then
    echo "resvg"
  elif command -v rsvg-convert >/dev/null 2>&1; then
    echo "rsvg-convert"
  elif command -v magick >/dev/null 2>&1; then
    echo "magick"
  else
    return 1
  fi
}

mapfile -t sizes < <(parse_sizes "$sizes_arg")
multi_size=0
if [[ ${#sizes[@]} -gt 1 ]]; then
  multi_size=1
fi

tool=""
if ! tool="$(detect_tool)"; then
  echo "[rasterize-svg] No SVG rasterizer found." >&2
  echo "[rasterize-svg] Install hints: winget install Inkscape.Inkscape | choco install inkscape | apt install inkscape" >&2
  echo "[rasterize-svg] Alternative install hints: winget install linebender.resvg | choco install resvg | apt install librsvg2-bin" >&2
  exit 1
fi

mkdir -p "$output_dir"
echo "[rasterize-svg] Using $tool"
echo "[rasterize-svg] Input: $input_dir"
echo "[rasterize-svg] Output: $output_dir"
echo "[rasterize-svg] Sizes: ${sizes[*]}"

shopt -s nullglob
mapfile -t svg_files < <(find "$input_dir" -type f -name '*.svg' | sort)
if [[ ${#svg_files[@]} -eq 0 ]]; then
  echo "[rasterize-svg] No SVG files found under '$input_dir'." >&2
  exit 0
fi

for svg in "${svg_files[@]}"; do
  rel="${svg#"$input_dir"/}"
  rel_parent="$(dirname "$rel")"
  base="$(basename "$svg" .svg)"
  dest_dir="$output_dir"
  if [[ "$rel_parent" != "." ]]; then
    dest_dir="$output_dir/$rel_parent"
  fi
  mkdir -p "$dest_dir"

  for size in "${sizes[@]}"; do
    suffix=""
    if [[ $multi_size -eq 1 ]]; then
      suffix="-$size"
    fi
    out="$dest_dir/$base$suffix.png"

    case "$tool" in
      inkscape)
        inkscape "$svg" --export-type=png --export-area-page --export-width="$size" --export-filename="$out"
        ;;
      resvg)
        resvg "$svg" "$out" -w "$size"
        ;;
      rsvg-convert)
        rsvg-convert -w "$size" "$svg" -o "$out"
        ;;
      magick)
        magick "$svg" -background none -alpha set -resize "${size}x${size}" "$out"
        ;;
    esac

    echo "[rasterize-svg] Wrote $out"
  done
done
