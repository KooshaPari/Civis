# Coverage baseline 2026-06-10 (llvm-cov)

## Per-crate coverage
| Crate | Regions | Missed Regions | Region Coverage | Functions | Missed Functions | Lines | Missed Lines | Line Coverage |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
|agents|2376|116|95.12%|158|9|1901|97|94.90%|
|ai|399|399|0.00%|51|51|287|287|0.00%|
|build|550|21|96.18%|37|4|418|25|94.02%|
|civ-traffic|775|40|94.84%|54|4|442|31|92.99%|
|civis-cli|519|386|25.63%|35|25|355|275|22.54%|
|civis-mcp|3|3|0.00%|1|1|3|3|0.00%|
|civlab-sdk|292|93|68.15%|39|19|209|88|57.89%|
|diffusion|291|11|96.22%|22|0|162|10|93.83%|
|economy|1000|101|89.90%|63|3|640|66|89.69%|
|engine|7268|946|86.98%|452|59|4972|638|87.17%|
|genetics|355|14|96.06%|26|2|194|11|94.33%|
|laws|191|5|97.38%|19|1|153|2|98.69%|
|legends|1629|820|49.66%|113|61|1087|516|52.53%|
|mod-host|2915|377|87.07%|205|36|1787|246|86.23%|
|needs|419|10|97.61%|30|2|383|6|98.43%|
|planet|420|4|99.05%|33|0|286|4|98.60%|
|protocol-3d|446|34|92.38%|26|4|332|26|92.17%|
|research|502|7|98.61%|36|0|354|3|99.15%|
|save-db|405|42|89.63%|19|1|250|14|94.40%|
|server|3953|804|79.66%|284|46|2993|519|82.66%|
|species|232|0|100.00%|16|0|150|0|100.00%|
|tactics|2692|31|98.85%|196|2|1582|26|98.36%|
|voxel|3766|361|90.41%|186|15|2102|210|90.01%|
|watch|3607|1157|67.92%|296|108|2814|1045|62.86%|

## Workspace total
Regions: 35005, Missed Regions: 5782, Region coverage: 83.48%; Functions: 2397, Missed Functions: 453, Function coverage: 81.10%; Lines: 23856, Missed Lines: 4148, Line coverage: 82.61%.

## 10 worst files
| File | Regions Missed | Region Coverage | Line Coverage |
|---|---:|---:|---:|
|ai\src\registry.rs|29|0.00%|0.00%|
|civis-mcp\src\lib.rs|3|0.00%|0.00%|
|civis-cli\src\screenshot.rs|59|0.00%|0.00%|
|civis-cli\src\proc.rs|33|0.00%|0.00%|
|civis-cli\src\lib.rs|93|0.00%|0.00%|
|civis-cli\src\build.rs|27|0.00%|0.00%|
|ai\src\providers\dummy.rs|56|0.00%|0.00%|
|ai\src\provenance.rs|38|0.00%|0.00%|
|ai\src\preflight.rs|37|0.00%|0.00%|
|ai\src\pool.rs|108|0.00%|0.00%|

## Delta plan to 85%
| Stage | Action | Target metric change |
|---|---|---:|
| 1 | Expand test cases in `watch` low-coverage modules (`app.rs`, `snapshot.rs`, `saves_api.rs`, `server.rs`, `sim_worker.rs`) | +6.00pp line |
| 2 | Add focused coverage for `ai` module entry points and `civis-cli` command branches | +1.00pp lines |
| 3 | Fill in high-risk `watch/mods_api.rs` and `server/ws_bridge.rs` request/error paths | +1.00pp+ to close gap |
| 4 | Re-run llvm-cov and capture delta to <= 85.00% lines | +85.00% target |
