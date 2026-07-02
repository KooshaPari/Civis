// gen-crests.mjs — procedural heraldry for emergent factions (DNA -> form).
// Each faction shares a structural "design DNA" (escutcheon ring + emergent node sigil)
// but a distinct hue, shield silhouette, and central glyph. Keycap teal accents the frame.
// Emits 6 SVGs matching existing filenames: crest-{blue,cyan,gold,green,red,violet}.svg
import { writeFileSync, mkdirSync } from "node:fs";
import { dirname } from "node:path";

const OUT = process.argv[2] || "./svg/crests";
mkdirSync(OUT, { recursive: true });

const KEYCAP = "#7ebab5";

// Faction DNA: hue color, accent (lighter), shield style, central glyph type.
const FACTIONS = [
  { name: "blue",   c: "#5b8fc9", a: "#9cc2ec", shield: "heater", glyph: "tower"   },
  { name: "cyan",   c: "#46b6c4", a: "#8fe1ea", shield: "rounded", glyph: "wave"    },
  { name: "gold",   c: "#c9a24b", a: "#ecd089", shield: "pointed", glyph: "sun"     },
  { name: "green",  c: "#5aa86a", a: "#9bd6a6", shield: "spade",   glyph: "leaf"    },
  { name: "red",    c: "#c95b5b", a: "#ec9c9c", shield: "heater",  glyph: "flame"   },
  { name: "violet", c: "#8a6fc9", a: "#c0aeec", shield: "rounded", glyph: "crystal" },
];

// Shield silhouettes (viewBox 128, centered, ~width 78)
function shieldPath(style) {
  switch (style) {
    case "heater":  return "M64,18 L102,30 V64 Q102,98 64,112 Q26,98 26,64 V30 Z";
    case "rounded": return "M40,20 H88 Q104,20 104,40 V62 Q104,96 64,112 Q24,96 24,62 V40 Q24,20 40,20 Z";
    case "pointed": return "M64,16 L100,28 V58 Q100,86 64,114 Q28,86 28,58 V28 Z";
    case "spade":   return "M64,16 Q100,40 100,66 Q100,98 64,112 Q28,98 28,66 Q28,40 64,16 Z";
    default:        return "M64,18 L102,30 V64 Q102,98 64,112 Q26,98 26,64 V30 Z";
  }
}

// Central glyphs — each renders to inner content (stroke=accent).
function glyph(type, a) {
  const s = (b) => `stroke="${a}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" fill="none" ${b}`;
  switch (type) {
    case "tower": // settlement / order
      return `<rect x="54" y="50" width="20" height="34" ${s("")}/>
              <path ${s('d="M50,50 L64,38 L78,50"')}/>
              <line x1="64" y1="50" x2="64" y2="84" ${s("")}/>`;
    case "wave": // tide / flow
      return `<path ${s('d="M40,58 Q52,46 64,58 Q76,70 88,58"')}/>
              <path ${s('d="M40,74 Q52,62 64,74 Q76,86 88,74"')}/>`;
    case "sun": // dawn / dominion
      return `<circle cx="64" cy="64" r="13" ${s("")}/>
              ${[0,45,90,135,180,225,270,315].map(d=>{const r=d*Math.PI/180;
                const x1=64+Math.cos(r)*20,y1=64+Math.sin(r)*20,x2=64+Math.cos(r)*27,y2=64+Math.sin(r)*27;
                return `<line x1="${x1.toFixed(1)}" y1="${y1.toFixed(1)}" x2="${x2.toFixed(1)}" y2="${y2.toFixed(1)}" ${s("")}/>`}).join("")}`;
    case "leaf": // growth / kinship
      return `<path ${s('d="M64,42 Q86,56 64,86 Q42,56 64,42 Z"')}/>
              <line x1="64" y1="48" x2="64" y2="84" ${s("")}/>`;
    case "flame": // war / forge
      return `<path ${s('d="M64,40 Q78,56 70,72 Q80,66 76,82 Q64,94 52,82 Q48,66 58,72 Q50,56 64,40 Z"')}/>`;
    case "crystal": // arcana / lattice
      return `<path ${s('d="M64,40 L80,58 L72,86 L56,86 L48,58 Z"')}/>
              <line x1="64" y1="40" x2="64" y2="86" ${s("")}/>
              <line x1="48" y1="58" x2="80" y2="58" ${s("")}/>`;
    default: return "";
  }
}

function crest(f) {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128">
  <!-- Faction crest "${f.name}" — procedural heraldry (DNA->form). Shared Keycap frame, distinct hue/shield/sigil. -->
  <defs>
    <radialGradient id="amb" cx="50%" cy="46%" r="55%">
      <stop offset="0%" stop-color="${f.c}" stop-opacity="0.30"/>
      <stop offset="100%" stop-color="${f.c}" stop-opacity="0"/>
    </radialGradient>
    <linearGradient id="field" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#0e1416"/>
      <stop offset="100%" stop-color="#080b0c"/>
    </linearGradient>
    <filter id="glow" x="-30%" y="-30%" width="160%" height="160%">
      <feGaussianBlur stdDeviation="2" result="b"/><feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
  </defs>

  <circle cx="64" cy="64" r="56" fill="url(#amb)"/>
  <!-- shared Keycap teal outer ring (faction DNA) -->
  <circle cx="64" cy="64" r="58" fill="none" stroke="${KEYCAP}" stroke-width="1.4" opacity="0.35" stroke-dasharray="3 5"/>

  <!-- shield field -->
  <path d="${shieldPath(f.shield)}" fill="url(#field)" stroke="${f.c}" stroke-width="3" filter="url(#glow)"/>
  <path d="${shieldPath(f.shield)}" fill="none" stroke="${f.a}" stroke-width="1" opacity="0.5"/>

  <!-- central faction glyph -->
  <g filter="url(#glow)">${glyph(f.glyph, f.a)}</g>

  <!-- emergent node motif at crest base (shared DNA) -->
  <g>
    <line x1="50" y1="98" x2="64" y2="104" stroke="${KEYCAP}" stroke-width="1" opacity="0.5"/>
    <line x1="78" y1="98" x2="64" y2="104" stroke="${KEYCAP}" stroke-width="1" opacity="0.5"/>
    <circle cx="50" cy="98" r="2.2" fill="${KEYCAP}" opacity="0.8"/>
    <circle cx="78" cy="98" r="2.2" fill="${KEYCAP}" opacity="0.8"/>
    <circle cx="64" cy="104" r="2.6" fill="${f.a}"/>
  </g>
</svg>
`;
}

for (const f of FACTIONS) {
  const p = `${OUT}/crest-${f.name}.svg`;
  mkdirSync(dirname(p), { recursive: true });
  writeFileSync(p, crest(f));
  console.log("wrote", p);
}
