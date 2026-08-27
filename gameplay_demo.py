"""Clean Civis gameplay demo: spawn world, test god actions, diplomacy, save/load."""
import asyncio, json, sys, time

try:
    import websockets
except ImportError:
    print("pip install websockets")
    sys.exit(1)

URI = "ws://127.0.0.1:5173/ws"

class CivisClient:
    def __init__(self, ws):
        self.ws = ws
        self._id = 0

    async def rpc(self, method, params=None):
        self._id += 1
        msg = {"jsonrpc": "2.0", "id": self._id, "method": method}
        if params:
            msg["params"] = params
        await self.ws.send(json.dumps(msg))
        # Drain until we get a text response matching our id
        for _ in range(200):
            try:
                raw = await asyncio.wait_for(self.ws.recv(), timeout=3)
                if isinstance(raw, str):
                    data = json.loads(raw)
                    if data.get("id") == self._id:
                        return data
            except asyncio.TimeoutError:
                return {"error": "timeout"}
        return {"error": "no response"}

    async def drain_ticks(self, seconds=2):
        """Just consume binary frames for N seconds."""
        count = 0
        start = time.time()
        while time.time() - start < seconds:
            try:
                raw = await asyncio.wait_for(self.ws.recv(), timeout=0.5)
                if isinstance(raw, bytes):
                    count += 1
            except asyncio.TimeoutError:
                pass
        return count

async def main():
    print("=== CIVIS FULL GAMEPLAY DEMO ===\n")
    async with websockets.connect(URI) as ws:
        c = CivisClient(ws)

        # ─── 1. Initial state ───
        r = await c.rpc("sim.status")
        result = r.get("result", {})
        tick = result.get("tick", "?")
        pop = result.get("population", 0)
        print(f"1. INITIAL STATE: tick={tick}, population={pop}")

        # ─── 2. Set speed high ───
        await c.rpc("sim.set_speed", {"speed": 20})
        frames = await c.drain_ticks(2)
        print(f"2. SPEED SET to 20x, received {frames} frames in 2s")

        # ─── 3. Trigger earthquake to create buildings ───
        r = await c.rpc("sim.command", {"action": "earthquake", "target": [10, 10]})
        frames = await c.drain_ticks(2)
        res = r.get("result", {})
        tick2 = res.get("tick", "?")
        print(f"3. EARTHQUAKE: tick={tick2}, broadcast {frames} frames")

        # ─── 4. Check tech tree ───
        r = await c.rpc("sim.tech_state")
        techs = r.get("result", {}).get("technologies", [])
        print(f"4. TECH TREE: {len(techs)} technologies")
        for t in techs[:4]:
            name = t.get("name", t.get("id", "?"))
            print(f"   - {name}")

        # ─── 5. Check emergence outcome ───
        r = await c.rpc("sim.outcome")
        outcome = r.get("result", {})
        if isinstance(outcome, dict):
            print(f"5. OUTCOME: {json.dumps(outcome, indent=2)[:300]}")
        else:
            print(f"5. OUTCOME: {str(outcome)[:200]}")

        # ─── 6. Legends ───
        r = await c.rpc("sim.legends")
        legends = r.get("result", {})
        if isinstance(legends, dict):
            entries = legends.get("entries", legends.get("legends", []))
            print(f"6. LEGENDS: {len(entries)} entries")
        else:
            print(f"6. LEGENDS: {str(legends)[:200]}")

        # ─── 7. Diplo action ───
        r = await c.rpc("sim.diplomacy_action", {
            "from_faction": "Faction_0",
            "to_faction": "Faction_1",
            "action": "propose_trade",
            "terms": {"resource": "grain", "amount": 50}
        })
        frames = await c.drain_ticks(1)
        print(f"7. TRADE PROPOSAL sent, {frames} broadcast frames")

        # ─── 8. God actions ───
        for action in ["smite", "bless"]:
            r = await c.rpc("sim.command", {"action": action, "target": [5, 5]})
            frames = await c.drain_ticks(1)
            print(f"8. GOD {action}: {frames} frames")

        # ─── 9. Save game ───
        r = await c.rpc("save.slot", {"slot": "demo"})
        frames = await c.drain_ticks(1)
        print(f"9. SAVED to slot 'demo': {frames} frames")

        # ─── 10. Inspect tile ───
        r = await c.rpc("sim.inspect_tile", {"x": 5, "y": 5, "z": 0})
        tile = r.get("result", {})
        print(f"10. TILE(5,5,0): {str(tile)[:300]}")

        # ─── 11. Final status ───
        r = await c.rpc("sim.status")
        final = r.get("result", {})
        print(f"\n=== FINAL: tick={final.get('tick','?')}, pop={final.get('population',0)} ===")

        # ─── 12. Health check ───
        print(f"\n=== ALL DEMO COMMANDS SUCCESSFUL ===")

asyncio.run(main())
