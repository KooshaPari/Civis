"""Populate the Civis world: spawn factions, trigger diplomacy, test god-actions."""
import asyncio, json, sys, time

try:
    import websockets
except ImportError:
    print("pip install websockets")
    sys.exit(1)

URI = "ws://127.0.0.1:5173/ws"

async def send_rpc(ws, method, params=None, _id=[0]):
    _id[0] += 1
    msg = {"jsonrpc": "2.0", "id": _id[0], "method": method}
    if params:
        msg["params"] = params
    await ws.send(json.dumps(msg))
    # Drain binary frames until we get a text response
    for _ in range(50):
        try:
            raw = await asyncio.wait_for(ws.recv(), timeout=5)
            if isinstance(raw, str):
                return json.loads(raw)
            # skip binary
        except asyncio.TimeoutError:
            return {"error": "timeout"}
    return {"error": "no text response"}

async def main():
    print("Connecting to", URI)
    async with websockets.connect(URI) as ws:
        # ─── 1. Check current state ───
        r = await send_rpc(ws, "sim.status")
        tick = r.get("result", {}).get("tick", "?")
        pop = r.get("result", {}).get("population", 0)
        print(f"\n=== CURRENT STATE: tick={tick}, population={pop} ===")

        # ─── 2. Get snapshot (factions, civilians) ───
        r = await send_rpc(ws, "sim.snapshot")
        snap = r.get("result", {})
        factions = snap.get("factions", [])
        civilians = snap.get("civilians", [])
        buildings = snap.get("buildings", [])
        print(f"Factions: {len(factions)}, Civilians: {len(civilians)}, Buildings: {len(buildings)}")
        if factions:
            for f in factions[:5]:
                print(f"  - {f.get('name', 'unnamed')}: pop={f.get('population', 0)}, mood={f.get('mood', '?')}")
        if civilians:
            for c in civilians[:5]:
                print(f"  - civilian: pos={c.get('pos', '?')}, task={c.get('task', '?')}")

        # ─── 3. Try sim.command to trigger world gen / spawn ───
        print("\n=== SENDING sim.command: spawn ===")
        r = await send_rpc(ws, "sim.command", {"action": "spawn"})
        print(f"  spawn result: {json.dumps(r, indent=2)[:500]}")

        # Wait for a few ticks
        await asyncio.sleep(2)

        # ─── 4. Check state again ───
        r = await send_rpc(ws, "sim.status")
        tick2 = r.get("result", {}).get("tick", "?")
        pop2 = r.get("result", {}).get("population", 0)
        print(f"\n=== AFTER SPAWN: tick={tick2}, population={pop2} (was {pop}) ===")

        # ─── 5. Get tech tree ───
        r = await send_rpc(ws, "sim.tech_state")
        techs = r.get("result", {}).get("technologies", [])
        print(f"\n=== TECH TREE ({len(techs)} technologies) ===")
        for t in techs[:6]:
            print(f"  - {t.get('name', '?')}: researched={t.get('researched', False)}, progress={t.get('progress', 0)}")

        # ─── 6. Try diplomacy actions ───
        print("\n=== DIPLOMACY ACTIONS ===")
        r = await send_rpc(ws, "sim.diplomacy_action", {
            "from_faction": "Faction_A",
            "to_faction": "Faction_B",
            "action": "propose_trade",
            "terms": {"resource": "food", "amount": 100}
        })
        print(f"  propose_trade: {json.dumps(r, indent=2)[:300]}")

        r = await send_rpc(ws, "sim.diplomacy_action", {
            "from_faction": "Faction_A",
            "to_faction": "Faction_B",
            "action": "declare_war"
        })
        print(f"  declare_war: {json.dumps(r, indent=2)[:300]}")

        # ─── 7. Test god actions ───
        print("\n=== GOD ACTIONS ===")
        for action in ["smite", "bless", "earthquake"]:
            r = await send_rpc(ws, "sim.command", {"action": action, "target": [5, 5]})
            print(f"  {action}: {json.dumps(r, indent=2)[:300]}")

        # ─── 8. Set speed to fast ───
        r = await send_rpc(ws, "sim.set_speed", {"speed": 10})
        print(f"\n=== SET SPEED: {json.dumps(r, indent=2)[:200]} ===")

        # ─── 9. Live ticks for 5 seconds ───
        print("\n=== LIVE TICKS (5 seconds) ===")
        start = time.time()
        tick_count = 0
        while time.time() - start < 5:
            try:
                raw = await asyncio.wait_for(ws.recv(), timeout=1)
                if isinstance(raw, bytes):
                    tick_count += 1
            except asyncio.TimeoutError:
                break
        print(f"  Received {tick_count} binary frames in 5s ({tick_count/5:.1f} frames/sec)")

        # ─── 10. Final state ───
        r = await send_rpc(ws, "sim.status")
        final = r.get("result", {})
        print(f"\n=== FINAL: tick={final.get('tick','?')}, pop={final.get('population',0)} ===")

        # ─── 11. Legends ───
        r = await send_rpc(ws, "sim.legends")
        legends = r.get("result", {})
        if legends:
            entries = legends.get("entries", legends.get("legends", []))
            print(f"\n=== LEGENDS ({len(entries)} entries) ===")
            for e in entries[:5]:
                print(f"  - {e}")

        # ─── 12. Outcome metrics ───
        r = await send_rpc(ws, "sim.outcome")
        outcome = r.get("result", {})
        print(f"\n=== OUTCOME METRICS ===")
        for k, v in (outcome if isinstance(outcome, dict) else {}).items():
            print(f"  {k}: {v}")

        # ─── 13. Save game ───
        r = await send_rpc(ws, "save.slot", {"slot": "demo_save"})
        print(f"\n=== SAVE: {json.dumps(r, indent=2)[:200]} ===")

        # ─── 14. Subscribe to specific frames ───
        r = await send_rpc(ws, "sim.subscribe", {"frame_kinds": ["climate", "terrain"]})
        print(f"\n=== SUBSCRIBE: {json.dumps(r, indent=2)[:200]} ===")

        # ─── 15. Inspect a tile ───
        r = await send_rpc(ws, "sim.inspect_tile", {"x": 10, "y": 10, "z": 0})
        tile = r.get("result", {})
        print(f"\n=== TILE (10,10,0) ===")
        if isinstance(tile, dict):
            for k, v in tile.items():
                print(f"  {k}: {str(v)[:100]}")
        else:
            print(f"  {json.dumps(tile, indent=2)[:500]}")

        print("\n=== DEMO COMPLETE ===")

asyncio.run(main())
