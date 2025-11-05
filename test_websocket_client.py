#!/usr/bin/env python3
"""
WebSocket JSON-RPC Test Client

A simple Python client to test the WebSocket JSON-RPC endpoint.
Requires: pip install websockets asyncio

Usage:
    python test_websocket_client.py
"""

import asyncio
import json
import sys

try:
    import websockets
except ImportError:
    print("Error: websockets library not installed")
    print("Install with: pip install websockets")
    sys.exit(1)


class JsonRpcClient:
    """JSON-RPC WebSocket Client"""

    def __init__(self, url: str = "ws://127.0.0.1:3000/live"):
        self.url = url
        self.websocket = None
        self.request_id = 1

    async def connect(self):
        """Connect to the WebSocket server"""
        try:
            self.websocket = await websockets.connect(self.url)
            print(f"✓ Connected to {self.url}")
        except Exception as e:
            print(f"✗ Failed to connect: {e}")
            raise

    async def disconnect(self):
        """Disconnect from the WebSocket server"""
        if self.websocket:
            await self.websocket.close()
            print("✓ Disconnected")

    async def send_request(self, method: str, params=None, request_id=None):
        """
        Send a JSON-RPC request

        Args:
            method: The method name to call
            params: Optional parameters (dict or list)
            request_id: Optional request ID (if None, auto-increments)

        Returns:
            The response from the server
        """
        if not self.websocket:
            raise Exception("Not connected")

        # Build request
        request = {
            "jsonrpc": "2.0",
            "method": method
        }

        if params is not None:
            request["params"] = params

        if request_id is None:
            request["id"] = self.request_id
            self.request_id += 1
        else:
            request["id"] = request_id

        # Send request
        request_json = json.dumps(request)
        print(f"\n→ Sending: {request_json}")
        await self.websocket.send(request_json)

        # Receive response
        response = await self.websocket.recv()
        print(f"← Received: {response}")

        return json.loads(response)

    async def send_notification(self, method: str, params=None):
        """
        Send a JSON-RPC notification (no response expected)

        Args:
            method: The method name to call
            params: Optional parameters
        """
        if not self.websocket:
            raise Exception("Not connected")

        # Build notification (no id field)
        notification = {
            "jsonrpc": "2.0",
            "method": method
        }

        if params is not None:
            notification["params"] = params

        # Send notification
        notification_json = json.dumps(notification)
        print(f"\n→ Sending notification: {notification_json}")
        await self.websocket.send(notification_json)
        print("  (No response expected for notifications)")

    async def ping(self):
        """Test ping method"""
        print("\n" + "="*50)
        print("Testing: ping")
        print("="*50)
        response = await self.send_request("ping")
        return response

    async def echo(self, message: dict):
        """Test echo method"""
        print("\n" + "="*50)
        print("Testing: echo")
        print("="*50)
        response = await self.send_request("echo", message)
        return response

    async def add(self, a: float, b: float):
        """Test add method"""
        print("\n" + "="*50)
        print("Testing: add")
        print("="*50)
        response = await self.send_request("add", [a, b])
        return response

    async def get_server_info(self):
        """Test getServerInfo method"""
        print("\n" + "="*50)
        print("Testing: getServerInfo")
        print("="*50)
        response = await self.send_request("getServerInfo")
        return response


async def run_tests():
    """Run a comprehensive test suite"""
    client = JsonRpcClient()

    try:
        # Connect
        await client.connect()

        # Test 1: Ping
        response = await client.ping()
        assert "result" in response
        assert response["result"]["pong"] is True
        print("✓ Ping test passed")

        # Test 2: Echo
        test_data = {"message": "Hello, WebSocket!", "number": 42}
        response = await client.echo(test_data)
        assert response["result"] == test_data
        print("✓ Echo test passed")

        # Test 3: Add
        response = await client.add(15, 27)
        assert response["result"] == 42
        print("✓ Add test passed")

        # Test 4: Server Info
        response = await client.get_server_info()
        assert "result" in response
        assert response["result"]["name"] == "webboard"
        assert response["result"]["jsonrpc_version"] == "2.0"
        print("✓ Server info test passed")

        # Test 5: Method not found
        print("\n" + "="*50)
        print("Testing: Method not found error")
        print("="*50)
        response = await client.send_request("nonexistent_method")
        assert "error" in response
        assert response["error"]["code"] == -32601
        print("✓ Method not found error test passed")

        # Test 6: Invalid params
        print("\n" + "="*50)
        print("Testing: Invalid parameters error")
        print("="*50)
        response = await client.send_request("add", [1])  # Needs 2 params
        assert "error" in response
        assert response["error"]["code"] == -32602
        print("✓ Invalid params error test passed")

        # Test 7: Notification (no response)
        await client.send_notification("echo", {"notify": "test"})
        print("✓ Notification test passed")

        print("\n" + "="*50)
        print("ALL TESTS PASSED! ✓")
        print("="*50)

    except Exception as e:
        print(f"\n✗ Test failed: {e}")
        import traceback
        traceback.print_exc()

    finally:
        await client.disconnect()


async def interactive_mode():
    """Interactive mode for manual testing"""
    client = JsonRpcClient()

    try:
        await client.connect()

        print("\n" + "="*50)
        print("Interactive JSON-RPC Client")
        print("="*50)
        print("\nCommands:")
        print("  ping              - Test ping method")
        print("  echo <message>    - Test echo method")
        print("  add <a> <b>       - Test add method")
        print("  info              - Get server info")
        print("  quit              - Exit")
        print()

        while True:
            try:
                command = input("\n> ").strip()

                if not command:
                    continue

                if command == "quit":
                    break
                elif command == "ping":
                    await client.ping()
                elif command.startswith("echo "):
                    message = command[5:]
                    await client.echo({"message": message})
                elif command.startswith("add "):
                    parts = command.split()
                    if len(parts) == 3:
                        a = float(parts[1])
                        b = float(parts[2])
                        await client.add(a, b)
                    else:
                        print("Usage: add <number1> <number2>")
                elif command == "info":
                    await client.get_server_info()
                else:
                    print(f"Unknown command: {command}")

            except KeyboardInterrupt:
                print("\nInterrupted")
                break
            except Exception as e:
                print(f"Error: {e}")

    finally:
        await client.disconnect()


def main():
    """Main entry point"""
    print("WebSocket JSON-RPC Test Client")
    print("="*50)

    if len(sys.argv) > 1 and sys.argv[1] == "--interactive":
        print("Starting in interactive mode...")
        asyncio.run(interactive_mode())
    else:
        print("Running automated test suite...")
        print("(Use --interactive for interactive mode)")
        asyncio.run(run_tests())


if __name__ == "__main__":
    main()
