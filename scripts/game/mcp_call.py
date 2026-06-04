"""Tiny FastMCP HTTP client to drive the running dinoforge-mcp server.

Usage: python mcp_call.py <tool_name> <args_json_file>
The args file contains a JSON object of kwargs (or omit for no args).
"""
import asyncio
import json
import sys

from fastmcp import Client

URL = "http://127.0.0.1:8765/mcp/"


async def main():
    tool = sys.argv[1]
    args = {}
    if len(sys.argv) > 2:
        with open(sys.argv[2], "r", encoding="utf-8") as f:
            args = json.load(f)
    async with Client(URL) as client:
        result = await client.call_tool(tool, args)
        out = getattr(result, "data", None)
        if out is None:
            out = [getattr(c, "text", str(c)) for c in getattr(result, "content", [])]
        print(json.dumps(out, default=str))


asyncio.run(main())
