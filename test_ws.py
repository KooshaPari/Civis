import asyncio
import json
import websockets
import sys

async def test():
    uri = "ws://127.0.0.1:5173/ws"
    try:
        async with websockets.connect(uri) as ws:
            # Test sim.status
            await ws.send(json.dumps({"jsonrpc":"2.0","id":1,"method":"sim.status"}))
            resp = await asyncio.wait_for(ws.recv(), timeout=5)
            data = json.loads(resp)
            print("=== sim.status ===")
            print(json.dumps(data, indent=2)[:3000])
            
            # Wait for a tick broadcast
            print("\n=== Waiting for tick broadcast (3s)... ===")
            try:
                tick_msg = await asyncio.wait_for(ws.recv(), timeout=3)
                tick_data = json.loads(tick_msg)
                if "method" in tick_msg and "tick" in str(tick_data):
                    print(f"Received tick broadcast: tick={tick_data.get('params', {}).get('tick', '?')}")
                else:
                    print(f"Received: {tick_msg[:200]}")
            except asyncio.TimeoutError:
                print("No tick broadcast received in 3s")
                
    except Exception as e:
        import traceback
        traceback.print_exc()
        sys.exit(1)

asyncio.run(test())
