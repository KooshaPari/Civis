"""RPC framing regressions using real websocat and a local RFC6455 fixture.

These are transport tests, not Civis game E2E. WEBSOCAT_BIN must identify the
pinned websocat 1.13.0 executable; a missing tool is an error, never a skip.
"""
import base64
import hashlib
import json
import os
from pathlib import Path
import signal
import shutil
import socket
import struct
import subprocess
import tempfile
import threading
import unittest

import test_playthrough_rpc as rpc_tests


def encode_json(value):
    return json.dumps(value, separators=(',', ':')).encode()


def websocket_frame(opcode, payload, final=True):
    size = len(payload)
    if size < 126:
        length = bytes([size])
    elif size <= 65535:
        length = b'\x7e' + struct.pack('!H', size)
    else:
        length = b'\x7f' + struct.pack('!Q', size)
    return bytes([(0x80 if final else 0) | opcode]) + length + payload


def receive_exact(connection, size):
    result = b''
    while len(result) < size:
        chunk = connection.recv(size - len(result))
        if not chunk:
            raise EOFError('client disconnected during request')
        result += chunk
    return result


class WebsocketFixture:
    def __init__(self, frames):
        self.frames = frames
        self.request = None
        self.error = None
        self.connection = None
        self.listener = socket.socket()
        self.listener.bind(('127.0.0.1', 0))
        self.listener.listen(1)
        self.listener.settimeout(2)
        self.port = self.listener.getsockname()[1]
        self.thread = threading.Thread(target=self.serve)

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *_):
        if self.connection is not None:
            try:
                self.connection.shutdown(socket.SHUT_RDWR)
            except OSError:
                pass
            self.connection.close()
        self.listener.close()
        self.thread.join(2)
        if self.thread.is_alive():
            raise RuntimeError('WebSocket fixture did not terminate')

    def serve(self):
        try:
            self.connection, _ = self.listener.accept()
            with self.connection as connection:
                connection.settimeout(2)
                self.handshake(connection)
                self.request = self.read_request(connection)
                try:
                    for frame in self.frames:
                        connection.sendall(websocket_frame(*frame))
                    # Wait for actual RPC cleanup rather than closing after send.
                    while connection.recv(4096):
                        pass
                except (socket.timeout, ConnectionResetError, BrokenPipeError):
                    pass
        except Exception as error:
            self.error = error

    def handshake(self, connection):
        headers = b''
        while b'\r\n\r\n' not in headers:
            headers += receive_exact(connection, 1)
            if len(headers) > 8192:
                raise ValueError('oversized upgrade request')
        fields = dict(line.split(b':', 1) for line in headers.split(b'\r\n')[1:] if b':' in line)
        key = next(value.strip() for name, value in fields.items()
                   if name.lower() == b'sec-websocket-key')
        accept = base64.b64encode(hashlib.sha1(
            key + b'258EAFA5-E914-47DA-95CA-C5AB0DC85B11').digest())
        connection.sendall(
            b'HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n'
            b'Connection: Upgrade\r\nSec-WebSocket-Accept: ' + accept + b'\r\n\r\n')

    def read_request(self, connection):
        first, second = receive_exact(connection, 2)
        if first != 0x81 or not second & 0x80:
            raise ValueError('expected one masked text request')
        size = second & 0x7f
        if size == 126:
            size = struct.unpack('!H', receive_exact(connection, 2))[0]
        elif size == 127:
            size = struct.unpack('!Q', receive_exact(connection, 8))[0]
        if size > 8192:
            raise ValueError('oversized fixture request')
        mask = receive_exact(connection, 4)
        payload = receive_exact(connection, size)
        return json.loads(bytes(value ^ mask[index % 4]
                                for index, value in enumerate(payload)))


class PlaythroughRpcWireTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        executable = os.environ.get('WEBSOCAT_BIN') or shutil.which('websocat')
        if not executable or not Path(executable).is_file():
            raise RuntimeError('websocat 1.13.0 is required; set WEBSOCAT_BIN')
        cls.websocat = Path(executable).resolve()
        version = subprocess.check_output([str(cls.websocat), '--version'], text=True, timeout=5)
        if version.strip() != 'websocat 1.13.0':
            raise RuntimeError('wire regressions require pinned websocat 1.13.0')

    def call(self, frames, *, message_limit=None):
        with tempfile.TemporaryDirectory(prefix='playthrough-wire-') as directory:
            (Path(directory) / 'websocat').symlink_to(self.websocat)
            with WebsocketFixture(frames) as fixture:
                env = os.environ | {'PATH': directory + ':' + os.environ['PATH'],
                                    'RPC_TIMEOUT_SECONDS': '2'}
                if message_limit is not None:
                    env['RPC_MAX_MESSAGE_BYTES'] = str(message_limit)
                shell = f'''set -u
set -o pipefail
CIV_WS_HOST=127.0.0.1
CIV_SERVER_PORT={fixture.port}
CIV_WS_PATH=/ws
fail() {{ printf '%s\\n' "$*" >&2; }}
''' + rpc_tests.FUNCTIONS + '''
body="$(rpc 2 sim.status '{}')"
status=$?
printf '%s\\n' "$status"
printf '%s' "$body"
'''
                process = subprocess.Popen(['bash', '-c', shell], env=env,
                                           stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                           text=True, start_new_session=True)
                try:
                    stdout, stderr = process.communicate(timeout=5)
                finally:
                    if process.poll() is None:
                        os.killpg(process.pid, signal.SIGKILL)
                        process.communicate(timeout=2)
                self.assertEqual(process.returncode, 0, stderr[:500])
                self.assertIsNone(fixture.error, repr(fixture.error))
                self.assertEqual(fixture.request,
                                 {'jsonrpc': '2.0', 'id': 2, 'method': 'sim.status', 'params': {}})
                status, _, body = stdout.partition('\n')
                return int(status), body, stderr

    def assert_success(self, frames, expected, *, message_limit=None):
        status, body, errors = self.call(frames, message_limit=message_limit)
        self.assertEqual(status, 0, errors[:500])
        self.assertEqual(json.loads(body), expected)
        self.assertNotIn('jq:', errors)
        self.assertNotIn('unrecognized transport framing', errors)

    def test_small_matching_reply(self):
        response = {'jsonrpc': '2.0', 'id': 2, 'result': {'ok': True}}
        payload = encode_json(response)
        for limit in (None, len(payload)):
            with self.subTest(exact_limit=limit):
                self.assert_success([(1, payload)], response, message_limit=limit)

    def test_large_matching_result(self):
        response = {'jsonrpc': '2.0', 'id': 2, 'result': {'padding': 'x' * 200000}}
        self.assert_success([(1, encode_json(response))], response)

    def test_large_notification_before_matching_reply(self):
        notice = {'jsonrpc': '2.0', 'method': 'scene.reset', 'params': {'padding': 'x' * 200000}}
        response = {'jsonrpc': '2.0', 'id': 2, 'result': {'ok': True}}
        self.assert_success([(1, encode_json(notice)), (1, encode_json(response))], response)

    def test_large_binary_before_matching_reply(self):
        response = {'jsonrpc': '2.0', 'id': 2, 'result': {'ok': True}}
        binary = b'F3D0\x00\n\xff' + b'x' * 200000
        self.assert_success([(2, binary), (1, encode_json(response))], response)

    def test_large_matching_error_is_preserved(self):
        response = {'jsonrpc': '2.0', 'id': 2,
                    'error': {'code': -32602, 'message': 'bad params', 'data': 'x' * 200000}}
        status, body, errors = self.call([(1, encode_json(response))])
        self.assertNotEqual(status, 0)
        self.assertTrue(body, errors[:500])
        self.assertEqual(json.loads(body), response)
        self.assertIn('server returned an error', errors)
        self.assertNotIn('jq:', errors)

    def test_oversize_frame_fails_before_json_parsing(self):
        response = {'jsonrpc': '2.0', 'id': 2, 'result': {'padding': 'x' * 200000}}
        payload = encode_json(response)
        fragments = [(1 if offset == 0 else 0, payload[offset:offset + 50000],
                      offset + 50000 >= len(payload))
                     for offset in range(0, len(payload), 50000)]
        small = encode_json({'jsonrpc': '2.0', 'id': 2, 'result': {'ok': True}})
        for limit, frames in ((65536, [(1, payload)]), (65536, fragments),
                              (len(small) - 1, [(1, small)])):
            with self.subTest(limit=limit, fragmented=len(frames) > 1):
                status, body, errors = self.call(frames, message_limit=limit)
                self.assertNotEqual(status, 0)
                self.assertEqual(body, '')
                self.assertTrue('transport ended without a matching response' in errors, errors[:500])
                self.assertNotIn('jq:', errors)
                self.assertNotIn('unrecognized transport framing', errors)


if __name__ == '__main__':
    unittest.main()
