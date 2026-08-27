import asyncio
import json
import websockets
import sys

URI = "ws://127.0.0.1:5173/ws"

async def recv_rpc(ws, target_id, max_attempts=100):
    for _ in range(max_attempts):
        msg = await asyncio.wait_for(ws.recv(), timeout=10)
        if isinstance(msg, bytes):
            prefix = msg[:4] if len(msg) >= 4 else b''
            print(f"  [binary: {len(msg)} bytes, prefix={prefix}]")
            continue
        data = json.loads(msg)
        if "id" in data and data["id"] == target_id:
            return data
    return None

async def main():
    print(f"Connecting to {URI}...")
    async with websockets.connect(URI) as ws:
        print("Connected!\n")
        
        print("=" * 60)
        print("1. SIM.STATUS")
        print("=" * 60)
        await ws.send(json.dumps({"jsonrpc":"2.0","id":1,"method":"sim.status","params":{}}))
        resp = await recv_rpc(ws, 1)
        if resp:
            print(json.dumps(resp.get("result", {}), indent=2))
        
        print("\n" + "=" * 60)
        print("2. SIM.SNAPSHOT")
        print("=" * 60)
        await ws.send(json.dumps({"jsonrpc":"2.0","id":2,"method":"sim.snapshot","params":{}}))
        resp = await recv_rpc(ws, 2)
        if resp:
            result = resp.get("result", {})
            print(f"Tick: {result.get('tick', '?')}")
            factions = result.get('factions', [])
            civilians = result.get('civilians', [])
            buildings = result.get('buildings', [])
            print(f"Factions: {len(factions)}")
            print(f"Civilians: {len(civilians)}")
            print(f"Buildings: {len(buildings)}")
            for f in factions[:5]:
                print(f"  Faction: {f.get('name', '?')} pop={f.get('population', '?')} treasury={f.get('treasury', '?')}")
        
        print("\n" + "=" * 60)
        print("3. SIM.TECH_STATE")
        print("=" * 60)
        await ws.send(json.dumps({"jsonrpc":"2.0","id":3,"method":"sim.tech_state","params":{}}))
        resp = await recv_rpc(ws, 3)
        if resp:
            print(json.dumps(resp.get("result", {}), indent=2)[:1500])
        
        print("\n" + "=" * 60)
        print("4. SIM.GET_SPEED")
        print("=" * 60)
        await ws.send(json.dumps({"jsonrpc":"2.0","id":4,"method":"sim.get_speed","params":{}}))
        resp = await recv_rpc(ws, 4)
        if resp:
            print(json.dumps(resp, indent=2))
        
        print("\n" + "=" * 60)
        print("5. SIM.OUTCOME (Emergence Metrics)")
        print("=" * 60)
        await ws.send(json.dumps({"jsonrpc":"2.0","id":5,"method":"sim.outcome","params":{}}))
        resp = await recv_rpc(ws, 5)
        if resp:
            print(json.dumps(resp.get("result", {}), indent=2)[:2000])
        
        print("\n" + "=" * 60)
        print("6. SIM.LEGENDS")
        print("=" * 60)
        await ws.send(json.dumps({"jsonrpc":"2.0","id":6,"method":"sim.legends","params":{}}))
        resp = await recv_rpc(ws, 6)
        if resp:
            result = resp.get("result", {})
            legends = result.get("legends", [])
            print(f"Total legends: {len(legends)}")
            for l in legends[:5]:
                print(f"  - {l.get('title', '?')} ({l.get('kind', '?')})")
        
        print("\n" + "=" * 60)
        print("7. LIVE TICK BROADCASTS (5 ticks)")
        print("=" * 60)
        tick_count = 0
        while tick_count < 5:
            msg = await asyncio.wait_for(ws.recv(), timeout=10)
            if isinstance(msg, bytes):
                print(f"  [binary frame: {len(msg)} bytes]")
                continue
            data = json.loads(msg)
            if "method" in data and "tick" in str(data.get("params", {})):
                tick = data.get("params", {}).get("tick", "?")
                print(f"  Tick {tick} (JSON text)")
                tick_count += 1
        
        print("\n" + "=" * 60)
        print("CIVIS SERVER DEMO COMPLETE")
        print("=" * 60)

if __name__ == "__main__":
    asyncio.run(main())
